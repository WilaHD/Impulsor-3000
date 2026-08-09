use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    path::Path,
};

use pdfium_render::prelude::{
    PdfDocument, PdfFormFieldCommon, PdfPageRenderRotation, PdfRenderConfig, Pdfium, PdfiumError,
};

const PDF_FORM_FIELD_NAME_TEXT: &str = "Text Tagesimpuls";
const PDF_FORM_FIELD_NAME_LOSUNG: &str = "Losung";
const PDF_FORM_FIELD_NAME_AUTOR: &str = "Autor";

pub const IMPULS_FORM_FIELD_NAMES: [&str; 3] = [
    PDF_FORM_FIELD_NAME_TEXT,
    PDF_FORM_FIELD_NAME_LOSUNG,
    PDF_FORM_FIELD_NAME_AUTOR,
];

#[derive(Debug)]
pub struct ImpulsModel {
    pub state_html: ImpulsConvertingState,
    pub state_image: ImpulsConvertingState,
    pub file_name: String,
    pub file_path: String,
}

impl ImpulsModel {
    pub fn build_from_path_buf(file: &std::path::PathBuf) -> ImpulsModel {
        return ImpulsModel {
            file_name: file.file_name().unwrap().to_str().unwrap().to_string(),
            file_path: file.as_os_str().to_str().unwrap().to_string(),
            state_html: ImpulsConvertingState::Default,
            state_image: ImpulsConvertingState::Default,
        };
    }
}

pub struct Impuls<'a> {
    document_pdf: PdfDocument<'a>,
}

#[derive(Debug, Clone)]
pub enum ImpulsConvertingState {
    Default,
    Success,
    Failure(String),
}

impl<'a> Impuls<'a> {
    pub fn build_from_model(
        impuls_model: &ImpulsModel,
        pdfium: &'a Pdfium,
    ) -> Result<Impuls<'a>, PdfiumError> {
        let im = impuls_model;

        match pdfium.load_pdf_from_file(&im.file_path, None) {
            Ok(document_pdf) => Ok(Impuls {
                document_pdf: document_pdf,
            }),
            Err(e) => return Err(e),
        }
    }

    pub fn save_as_jpg(
        &self,
        impuls_model: &ImpulsModel,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = impuls_model.file_path.replace(".pdf", ".jpg");

        let render_config = PdfRenderConfig::new()
            .set_target_width(2000)
            .set_maximum_height(2000)
            .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

        let page_one = self.document_pdf.pages().get(0)?;

        page_one
            .render_with_config(&render_config)?
            .as_image()
            .into_rgb8()
            .save_with_format(file_path, image::ImageFormat::Jpeg)?;

        Ok(())
    }

    pub fn save_as_txt(
        &self,
        impuls_model: &ImpulsModel,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let form_values = match self.read_form_field_values() {
            Ok(ok) => ok,
            Err(err) => {
                let mut es = String::new();
                for e in err {
                    es = format!("{}\n{}", es, e.to_string());
                }
                return Err(es.into());
            }
        };

        let wordpress_txt = form_values.get_wordpress_string();

        let file_path = impuls_model.file_path.replace(".pdf", ".txt");
        let mut file = File::create(file_path)?;

        std::io::Write::write_all(&mut file, wordpress_txt.as_bytes())?;

        Ok(())
    }

    pub fn test_pdf_form_fields(&self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        return self.read_form_field_values().map(|_| ()).map_err(|err| err);
    }

    pub fn test_pdf_form_fields_as_str(&self) -> Result<(), Vec<String>> {
        return self
            .test_pdf_form_fields()
            .map_err(|errs| errs.iter().map(|err| err.to_string()).collect());
    }

    fn read_form_field_values(&self) -> Result<PdfFormValues, Vec<Box<dyn std::error::Error>>> {
        let mut collected_errors = vec![];

        let map_pdf = match self.read_pdf_file_to_map() {
            Ok(map) => map,
            Err(err) => {
                collected_errors.push(err);
                return Err(collected_errors);
            }
        };

        let pdf_text = get_form_map_value_by_key(&map_pdf, PDF_FORM_FIELD_NAME_TEXT)
            .map_err(|e| collected_errors.push(e))
            .unwrap_or_default();
        let pdf_autor = get_form_map_value_by_key(&map_pdf, PDF_FORM_FIELD_NAME_AUTOR)
            .map_err(|e| collected_errors.push(e))
            .unwrap_or_default();
        let pdf_losung = match get_form_map_value_by_key(&map_pdf, PDF_FORM_FIELD_NAME_LOSUNG) {
            Ok(o) => o,
            Err(e) => {
                collected_errors.push(e);
                return Err(collected_errors);
            }
        };

        let losung = match try_to_parse_losung(&pdf_losung) {
            Ok(o) => o,
            Err(e) => {
                collected_errors.push(e);
                return Err(collected_errors);
            }
        };

        return Ok(PdfFormValues {
            losung_at: losung.0,
            losung_nt: losung.1,
            text: pdf_text,
            autor: pdf_autor,
        });
    }

    fn read_pdf_file_to_map(&self) -> Result<HashMap<String, Option<String>>, Box<dyn Error>> {
        let mut map_pdf = HashMap::new();
        let Some(form) = self.document_pdf.form() else {
            return Err(Box::new(PdfiumError::UnknownFormType));
        };
        for (key, value) in form.field_values(self.document_pdf.pages()) {
            if value.is_some() {
                map_pdf.insert(key, value);
            }
        }
        return Ok(map_pdf);
    }
}

/// Copies the three Impuls text fields from `source_path` into a fresh instance of
/// `template_path` and saves it at `destination_path`.
///
/// The source document is never modified by this function. Callers can therefore
/// safely write to a temporary path before replacing or moving files.
pub fn copy_impuls_fields_to_template(
    source_path: &Path,
    template_path: &Path,
    destination_path: &Path,
    pdfium: &Pdfium,
) -> Result<(), String> {
    let source_document = pdfium
        .load_pdf_from_file(source_path, None)
        .map_err(|error| format!("Quell-PDF konnte nicht geöffnet werden: {error}"))?;
    let source_values = read_named_impuls_form_values(&source_document)?;

    let template_document = pdfium
        .load_pdf_from_file(template_path, None)
        .map_err(|error| format!("Neue Vorlage konnte nicht geöffnet werden: {error}"))?;
    let mut copied_fields = HashSet::new();

    for page in template_document.pages().iter() {
        for mut annotation in page.annotations().iter() {
            let Some(field) = annotation.as_form_field_mut() else {
                continue;
            };

            let name = field.name().unwrap_or_default();
            let Some(value) = source_values.get(name.as_str()) else {
                continue;
            };
            let Some(text_field) = field.as_text_field_mut() else {
                return Err(format!(
                    "Feld '{name}' in der neuen Vorlage ist kein Textfeld."
                ));
            };

            text_field
                .set_value(value)
                .map_err(|error| format!("Feld '{name}' konnte nicht gesetzt werden: {error}"))?;
            copied_fields.insert(name);
        }
    }

    for name in IMPULS_FORM_FIELD_NAMES {
        if !copied_fields.contains(name) {
            return Err(format!(
                "Feld '{name}' wurde in der neuen Vorlage nicht gefunden."
            ));
        }
    }

    template_document
        .save_to_file(destination_path)
        .map_err(|error| format!("Neue PDF konnte nicht gespeichert werden: {error}"))
}

fn read_named_impuls_form_values(
    document: &PdfDocument<'_>,
) -> Result<HashMap<String, String>, String> {
    if document.form().is_none() {
        return Err("Quell-PDF enthält kein PDF-Formular.".to_string());
    }

    let mut result = HashMap::new();

    for page in document.pages().iter() {
        for annotation in page.annotations().iter() {
            let Some(field) = annotation.as_form_field() else {
                continue;
            };
            let name = field.name().unwrap_or_default();
            if !IMPULS_FORM_FIELD_NAMES.contains(&name.as_str()) {
                continue;
            }

            let Some(text_field) = field.as_text_field() else {
                return Err(format!("Feld '{name}' in der Quell-PDF ist kein Textfeld."));
            };
            result.insert(name, text_field.value().unwrap_or_default());
        }
    }

    for name in IMPULS_FORM_FIELD_NAMES {
        if !result.contains_key(name) {
            return Err(format!(
                "Feld '{name}' wurde in der Quell-PDF nicht gefunden."
            ));
        }
    }

    Ok(result)
}

fn get_form_map_value_by_key(
    map: &HashMap<String, Option<String>>,
    key: &str,
) -> Result<String, Box<dyn Error>> {
    let Some(Some(value)) = map.get(key) else {
        return Err(format!("PDF field '{key}' not found").into());
    };
    return Ok(value.trim().to_string());
}

fn try_to_parse_losung(pdf_losung: &str) -> Result<(String, String), Box<dyn Error>> {
    let pdf_losung = normalize_line_breaks(pdf_losung);
    let lines = pdf_losung.lines().collect::<Vec<&str>>();
    let Some(separator_index) = lines.iter().position(|line| line.trim().is_empty()) else {
        return Err("PDF field 'Losung' could not be parsed correct.".into());
    };

    let pdf_losung_at = lines[..separator_index].join("\n").trim().to_string();
    let pdf_losung_nt = lines[separator_index + 1..].join("\n").trim().to_string();

    return Ok((pdf_losung_at, pdf_losung_nt));
}

fn normalize_line_breaks(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(char) = chars.next() {
        match char {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                normalized.push('\n');
            }
            '\n' | '\u{2028}' => normalized.push('\n'),
            _ => normalized.push(char),
        }
    }

    normalized
}

struct PdfFormValues {
    pub losung_at: String,
    pub losung_nt: String,
    pub text: String,
    pub autor: String,
}

impl PdfFormValues {
    fn get_wordpress_string(&self) -> String {
        return format!(
            r#"<strong>Losung</strong>
{}

<strong>Lehrtext</strong>
{}

<strong>Impuls für den Tag</strong>
{}

{}

[audio mp3 = "https://URL[...].mp3"]"#,
            self.losung_at, self.losung_nt, self.text, self.autor
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_losung_split_by_blank_line_independent_of_line_break_style() {
        for separator in [
            "\n\n",
            "\r\n\r\n",
            "\r\r",
            "\n\r",
            "\u{2028}\u{2028}",
            "\r\u{2028}",
        ] {
            let parsed = try_to_parse_losung(&format!("AT{separator}NT")).unwrap();

            assert_eq!(parsed, ("AT".to_string(), "NT".to_string()));
        }
    }

    #[test]
    fn trims_parsed_losung_parts() {
        let parsed = try_to_parse_losung("  AT  \r\n\r\n  NT  ").unwrap();

        assert_eq!(parsed, ("AT".to_string(), "NT".to_string()));
    }

    #[test]
    fn allows_whitespace_in_blank_line_separator() {
        let parsed = try_to_parse_losung("AT\n \t \nNT").unwrap();

        assert_eq!(parsed, ("AT".to_string(), "NT".to_string()));
    }

    #[test]
    fn rejects_losung_without_blank_line_separator() {
        assert!(try_to_parse_losung("AT\nNT").is_err());
    }
}
