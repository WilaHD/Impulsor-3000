use std::{collections::HashMap, error::Error, fs::File};

use pdfium_render::prelude::{
    PdfDocument, PdfPageRenderRotation, PdfRenderConfig, Pdfium, PdfiumError,
};

const PDF_FORM_FIELD_NAME_TEXT: &str = "Text Tagesimpuls";
const PDF_FORM_FIELD_NAME_LOSUNG: &str = "Losung";
const PDF_FORM_FIELD_NAME_AUTOR: &str = "Autor";

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
    let pdf_losung_vec = if pdf_losung.contains("\r\n") {
        pdf_losung.split("\r\n\r\n").collect::<Vec<&str>>()
    } else if pdf_losung.contains("\r\r") {
        pdf_losung.split("\r\r").collect::<Vec<&str>>()
    } else if pdf_losung.contains("\n\r") {
        pdf_losung.split("\n\r").collect::<Vec<&str>>()
    } else {
        pdf_losung.split("\n\n").collect::<Vec<&str>>()
    };

    let Some(pdf_losung_at) = pdf_losung_vec.get(0) else {
        return Err("PDF field 'Losung' could not be parsed correct.".into());
    };

    let Some(pdf_losung_nt) = pdf_losung_vec.get(1) else {
        return Err("PDF field 'Losung' could not be parsed correct.".into());
    };

    let pdf_losung_at = pdf_losung_at.trim().to_string();
    let pdf_losung_nt = pdf_losung_nt.trim().to_string();

    return Ok((pdf_losung_at, pdf_losung_nt));
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
