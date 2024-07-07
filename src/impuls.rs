use std::{collections::HashMap, fs::File};

use pdfium_render::{document::PdfDocument, error::PdfiumError, page::PdfPageRenderRotation, pdfium::Pdfium, render_config::PdfRenderConfig};


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
    document_pdf: PdfDocument<'a>
}

#[derive(Debug, Clone)]
pub enum ImpulsConvertingState {
    Default,
    Success,
    Failure(String),
}

impl<'a> Impuls<'a> {
    pub fn build_from_model(impuls_model: &ImpulsModel, pdfium: &'a Pdfium) -> Result<Impuls<'a>, PdfiumError> {
        let im = impuls_model;
        
        match pdfium.load_pdf_from_file(&im.file_path, None) {
            Ok(document_pdf) => {
                Ok(Impuls{
                    document_pdf: document_pdf
                })
            }
            Err(e) => return Err(e),
        }
    }

    pub fn save_as_jpg(&self, impuls_model: &ImpulsModel) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = impuls_model.file_path.replace(".pdf", ".jpg");

        let render_config = PdfRenderConfig::new()
            .set_target_width(2000)
            .set_maximum_height(2000)
            .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

        let page_one = self.document_pdf.pages().get(0)?;
    
        page_one.render_with_config(&render_config)?
            .as_image()
            .as_rgba8().ok_or(PdfiumError::ImageError)?
            .save_with_format(file_path, image::ImageFormat::Jpeg)?;

        Ok(())
    }

    pub fn save_as_txt(&self, impuls_model: &ImpulsModel) -> Result<(),Box<dyn std::error::Error>> {

        let mut map_pdf = HashMap::new();

        if let Some(form) = self.document_pdf.form() {
            for (key, value) in form.field_values(self.document_pdf.pages()) {
                if value.is_some() {
                    map_pdf.insert(key, value);
                }
            }
        } else {
            return Err(Box::new(PdfiumError::UnknownFormType));
        }

        let map_pdf = map_pdf;

        let Some(Some(pdf_losung)) = map_pdf.get("Losung") else {
            return Err("PDF field 'Losung' not found".into())
        };
        
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
            return Err("PDF field 'Losung' could not be parsed correct. There is a problem with the biblical text (AT) and the indication of the passage.".into());
        };

        let Some(pdf_losung_nt) = pdf_losung_vec.get(1) else {
            return Err("PDF field 'Losung' could not be parsed correct. There is a problem with the biblical text (NT) and the indication of the passage.".into());
        };

        let pdf_losung_at = pdf_losung_at.trim();
        let pdf_losung_nt = pdf_losung_nt.trim();

        
        // let Some(Some(pdf_wochentag)) = map_pdf.get("Wochentag") else {
        //     panic!("PDF field 'Wochentag' not found");
        // };

        let Some(Some(pdf_text)) = map_pdf.get("Text Tagesimpuls") else {
            return Err("PDF field 'Text Tagesimpuls' not found".into());
        };
        let pdf_text = pdf_text.trim();

        let Some(Some(pdf_autor)) = map_pdf.get("Autor") else {
            return Err("PDF field 'Autor' not found".into());
        };
        let pdf_autor = pdf_autor.trim();

        let wordpress_txt = format!(
r#"<strong>Losung</strong>
{}

<strong>Lehrtext</strong>
{}

<strong>Impuls für den Tag</strong>
{}

{}

[audio mp3 = "https://URL[...].mp3"]"#,
        pdf_losung_at, pdf_losung_nt, pdf_text, pdf_autor);

        let file_path = impuls_model.file_path.replace(".pdf", ".txt");
        let mut file = File::create(file_path)?;
        
        std::io::Write::write_all(&mut file, wordpress_txt.as_bytes())?;

        Ok(())
    }
}