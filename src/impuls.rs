use std::{collections::HashMap, error::Error, fs::File, path::Path};

use crate::pdf_engine::PdfEngine;
use pdf_oxide::{
    editor::{DocumentEditor, EditableDocument, FormFieldValue},
    extractors::forms::FieldType,
    PdfDocument,
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
    path: std::path::PathBuf,
    engine: &'a PdfEngine,
}

#[derive(Debug, Clone)]
pub enum ImpulsConvertingState {
    Default,
    Success,
    Warning(String),
    Failure(String),
}

impl<'a> Impuls<'a> {
    pub fn build_from_model(
        model: &ImpulsModel,
        engine: &'a PdfEngine,
    ) -> Result<Self, Box<dyn Error>> {
        PdfDocument::open(&model.file_path)?;
        Ok(Self {
            path: model.file_path.clone().into(),
            engine,
        })
    }

    pub fn save_as_jpg(&self, model: &ImpulsModel) -> Result<Vec<String>, Box<dyn Error>> {
        let mut output = std::path::PathBuf::from(&model.file_path);
        output.set_extension("jpg");
        self.engine.render(&self.path, &output)
    }

    pub fn save_as_txt(&self, model: &ImpulsModel) -> Result<(), Box<dyn Error>> {
        let values = self.read_form_field_values().map_err(|errors| {
            errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let mut output = std::path::PathBuf::from(&model.file_path);
        output.set_extension("txt");
        let mut file = File::create(output)?;
        std::io::Write::write_all(&mut file, values.get_wordpress_string().as_bytes())?;
        Ok(())
    }

    pub fn test_pdf_form_fields(&self) -> Result<(), Vec<Box<dyn Error>>> {
        self.read_form_field_values().map(|_| ())
    }

    pub fn test_pdf_form_fields_as_str(&self) -> Result<(), Vec<String>> {
        self.test_pdf_form_fields()
            .map_err(|errors| errors.iter().map(|error| error.to_string()).collect())
    }

    fn read_form_field_values(&self) -> Result<PdfFormValues, Vec<Box<dyn Error>>> {
        let mut collected_errors = vec![];
        let map_pdf = self.read_pdf_file_to_map().map_err(|error| vec![error])?;
        let pdf_text = get_form_map_value_by_key(&map_pdf, PDF_FORM_FIELD_NAME_TEXT)
            .map_err(|e| collected_errors.push(e))
            .unwrap_or_default();
        let pdf_autor = get_form_map_value_by_key(&map_pdf, PDF_FORM_FIELD_NAME_AUTOR)
            .map_err(|e| collected_errors.push(e))
            .unwrap_or_default();
        let pdf_losung =
            get_form_map_value_by_key(&map_pdf, PDF_FORM_FIELD_NAME_LOSUNG).map_err(|e| {
                collected_errors.push(e);
                std::mem::take(&mut collected_errors)
            })?;
        let losung = try_to_parse_losung(&pdf_losung).map_err(|e| {
            collected_errors.push(e);
            std::mem::take(&mut collected_errors)
        })?;
        Ok(PdfFormValues {
            losung_at: losung.0,
            losung_nt: losung.1,
            text: pdf_text,
            autor: pdf_autor,
        })
    }

    fn read_pdf_file_to_map(&self) -> Result<HashMap<String, Option<String>>, Box<dyn Error>> {
        let mut editor = DocumentEditor::open(&self.path)?;
        let mut values = HashMap::new();
        for field in editor.get_form_fields()? {
            if let FormFieldValue::Text(value) = field.value() {
                values.insert(field.name().to_string(), Some(value));
            }
        }
        Ok(values)
    }
}

/// Copy the Impuls fields into a new editable template.
pub fn copy_impuls_fields_to_template(
    source_path: &Path,
    template_path: &Path,
    destination_path: &Path,
    _engine: &PdfEngine,
) -> Result<(), String> {
    let mut source = DocumentEditor::open(source_path)
        .map_err(|error| format!("Quell-PDF konnte nicht geöffnet werden: {error}"))?;
    let source_values = read_named_impuls_form_values(&mut source)?;
    let mut template = DocumentEditor::open(template_path)
        .map_err(|error| format!("Neue Vorlage konnte nicht geöffnet werden: {error}"))?;
    let fields = template
        .get_form_fields()
        .map_err(|error| error.to_string())?;
    for name in IMPULS_FORM_FIELD_NAMES {
        let field = fields
            .iter()
            .find(|field| field.name() == name)
            .ok_or_else(|| format!("Feld '{name}' wurde in der neuen Vorlage nicht gefunden."))?;
        if field.field_type() != Some(&FieldType::Text) {
            return Err(format!(
                "Feld '{name}' in der neuen Vorlage ist kein Textfeld."
            ));
        }
        template
            .set_form_field_value(name, FormFieldValue::Text(source_values[name].clone()))
            .map_err(|error| format!("Feld '{name}' konnte nicht gesetzt werden: {error}"))?;
    }
    template
        .save(destination_path)
        .map_err(|error| format!("Neue PDF konnte nicht gespeichert werden: {error}"))
}

fn read_named_impuls_form_values(
    document: &mut DocumentEditor,
) -> Result<HashMap<String, String>, String> {
    let mut result = HashMap::new();
    for field in document
        .get_form_fields()
        .map_err(|error| error.to_string())?
    {
        let name = field.name().to_string();
        if !IMPULS_FORM_FIELD_NAMES.contains(&name.as_str()) {
            continue;
        }
        if field.field_type() != Some(&FieldType::Text) {
            return Err(format!("Feld '{name}' in der Quell-PDF ist kein Textfeld."));
        }
        let value = match field.value() {
            FormFieldValue::Text(value) => value,
            FormFieldValue::None => String::new(),
            _ => return Err(format!("Feld '{name}' enthält keinen Text.")),
        };
        result.insert(name, value);
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

    #[test]
    fn reads_and_copies_editable_form_values() {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdf/form.pdf");
        let output =
            std::env::temp_dir().join(format!("impulsor-pdf-copy-{}.pdf", std::process::id()));
        let engine = PdfEngine::new();
        let model = ImpulsModel::build_from_path_buf(&input);
        let impuls = Impuls::build_from_model(&model, &engine).unwrap();
        let values = impuls.read_form_field_values().unwrap();
        assert_eq!(values.losung_at, "Güte und Hoffnung");
        assert_eq!(values.losung_nt, "Friede für alle");
        assert_eq!(values.autor, "Änne Müller");
        copy_impuls_fields_to_template(&input, &input, &output, &engine).unwrap();
        let mut copied = DocumentEditor::open(&output).unwrap();
        let values = read_named_impuls_form_values(&mut copied).unwrap();
        assert_eq!(values["Autor"], "Änne Müller");
        assert_eq!(
            values["Text Tagesimpuls"],
            "Grüße aus der Gemeinde.\nEin zweiter Absatz."
        );
        std::fs::remove_file(output).unwrap();
    }
}
