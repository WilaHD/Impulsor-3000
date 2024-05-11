use std::{collections::HashMap, fs::File, io::Write};

use image::ImageFormat;
use pdfium_render::prelude::*;

pub const TITLE:&str = r#"
.___                         .__                        ________  _______   _______   _______   
|   |  _____  ______   __ __ |  |    ______ ____ _______\_____  \ \   _  \  \   _  \  \   _  \  
|   | /     \ \____ \ |  |  \|  |   /  ___//  _ \\_  __ \ _(__  < /  /_\  \ /  /_\  \ /  /_\  \ 
|   ||  Y Y  \|  |_> >|  |  /|  |__ \___ \(  <_> )|  | \//       \\  \_/   \\  \_/   \\  \_/   \
|___||__|_|  /|   __/ |____/ |____//____  >\____/ |__|  /______  / \_____  / \_____  / \_____  /
           \/ |__|                      \/                     \/        \/        \/        \/ 
"#;

pub struct Impuls<'a> {
    document_pdf: PdfDocument<'a>,
    file_path: String,
}

impl<'a> Impuls<'a> {
    pub fn build(file_path: String, pdfium: &'a Pdfium) -> Result<Impuls,PdfiumError> {
        
        match pdfium.load_pdf_from_file(&file_path, None) {
            Ok(document_pdf) => {
                Ok(Impuls {
                    document_pdf: document_pdf,
                    file_path,
                })
            }
            Err(e) => return Err(e),
        }
    }

    pub fn save_as_jpg(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.file_path.replace(".pdf", ".jpg");

        let render_config = PdfRenderConfig::new()
            .set_target_width(2000)
            .set_maximum_height(2000)
            .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

        let page_one = self.document_pdf.pages().get(0)?;
    
        page_one.render_with_config(&render_config)?
            .as_image()
            .as_rgba8().ok_or(PdfiumError::ImageError)?
            .save_with_format(file_path, ImageFormat::Jpeg)?;

        Ok(())
    }

    pub fn save_as_txt(&self) -> Result<(),Box<dyn std::error::Error>> {

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
            panic!("PDF field 'Losung' not found");
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

        let pdf_losung_at = pdf_losung_vec[0].trim();
        let pdf_losung_nt = pdf_losung_vec[1].trim();

        
        // let Some(Some(pdf_wochentag)) = map_pdf.get("Wochentag") else {
        //     panic!("PDF field 'Wochentag' not found");
        // };

        let Some(Some(pdf_text)) = map_pdf.get("Text Tagesimpuls") else {
            panic!("PDF field 'Text Tagesimpuls' not found");
        };
        let pdf_text = pdf_text.trim();

        let Some(Some(pdf_autor)) = map_pdf.get("Autor") else {
            panic!("PDF field 'Autor' not found");
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

        let file_path = self.file_path.replace(".pdf", ".txt");
        let mut file = File::create(file_path)?;
        
        file.write_all(wordpress_txt.as_bytes())?;

        Ok(())
    }

    pub fn get_file_path(&self) -> &str {
        return &self.file_path
    }
}

pub fn choose_pdfium_by_os_arch() -> Result<String, String> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let mut path = "./pdfium/".to_string();

    match os {
        "linux" => {
            path += "linux-";
            match arch {
                "x86_64" => {
                    path += "x64"
                },
                "aarch64" => {
                    path += "arm64"
                },
                _ => {
                    return Err(format!("Architecture {arch} is not supported"));
                }
            }
        }
        "windows" => {
            path += "win-";
            match arch {
                "x86_64" => {
                    path += "x64"
                },
                "aarch64" => {
                    path += "arm64"
                },
                _ => {
                    return Err(format!("Architecture {arch} is not supported"));
                }
            }
        }
        "macos" => {
            path += "mac-univ";
        }
        _ => {
            return Err(format!("Platform {os} is not supported"));
        }
    }
    path += "/";
    Ok(path)
}
