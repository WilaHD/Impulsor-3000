pub const TITLE:&str = r#"
.___                         .__                        ________  _______   _______   _______   
|   |  _____  ______   __ __ |  |    ______ ____ _______\_____  \ \   _  \  \   _  \  \   _  \  
|   | /     \ \____ \ |  |  \|  |   /  ___//  _ \\_  __ \ _(__  < /  /_\  \ /  /_\  \ /  /_\  \ 
|   ||  Y Y  \|  |_> >|  |  /|  |__ \___ \(  <_> )|  | \//       \\  \_/   \\  \_/   \\  \_/   \
|___||__|_|  /|   __/ |____/ |____//____  >\____/ |__|  /______  / \_____  / \_____  / \_____  /
           \/ |__|                      \/                     \/        \/        \/        \/ 
"#;

// pub struct Impuls<'a> {
//     document_pdf: PdfDocument<'a>,
//     file_path: String,
// }

// impl<'a> Impuls<'a> {
//     pub fn build(file_path: String, pdfium: &'a Pdfium) -> Result<Impuls,PdfiumError> {
        
//         match pdfium.load_pdf_from_file(&file_path, None) {
//             Ok(document_pdf) => {
//                 Ok(Impuls {
//                     document_pdf: document_pdf,
//                     file_path,
//                 })
//             }
//             Err(e) => return Err(e),
//         }
//     }
// }

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
