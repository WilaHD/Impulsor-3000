use pdfium_render::prelude::* ;

use impulsor3000::*;

fn main() {

    println!("{}", crate::TITLE);

    let files = rfd::FileDialog::new()
        .set_title("Impuls PDF-Datei(en) auswählen")
        .add_filter("Impuls.pdf", &["pdf"])
        .set_directory("~")
        .pick_files();
    
    let mut file_paths = vec![];
    for file in files.unwrap() {
        file_paths.push(file.into_os_string().to_str().unwrap().to_string())
    }
    let file_paths = file_paths;

    let _ = convert(file_paths);
    
}

fn convert(file_paths: Vec<String>) -> Result<(), ()> {
    let mut everything_ok = true;

    let pdfium_path = choose_pdfium_by_os_arch().unwrap();

    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(
            Pdfium::pdfium_platform_library_name_at_path(&pdfium_path)).unwrap()
    );

    for file_path in file_paths {
        print!("Datei: ");
        let i = match Impuls::build(file_path, &pdfium) {
            Ok(i) => {
                print!("{} \t", i.get_file_path());
                i
            },
            Err(e) => {
                println!("❌");
                println!("\t Fehler: {}", e.to_string());
                everything_ok = false;
                continue;
            },
        };

        print!("Bild ");
        match i.save_as_jpg() {
            Ok(_) => {
                print!("✅ \t");
            },
            Err(e) => {
                println!("❌");
                println!("\t Fehler: {}", e.to_string());
                everything_ok = false;
            },
        }
        
        print!("Text ");
        match i.save_as_txt() {
            Ok(_) => {
                print!("✅ \n");
            },
            Err(e) => {
                println!("❌");
                println!("\t Fehler: {}", e.to_string());
                everything_ok = false;
            },
        }
    }

    return match everything_ok {
        true => Ok(()),
        false => Err(()),
    };
}


#[cfg(test)]
mod tests {
    use crate::convert;

    #[test]
    fn it_works_for_windows_pdfs() {
        let file_paths = vec![
            "test/impulse/impuls.pdf".to_string(),
            "test/impulse/impuls2.pdf".to_string(),
        ];

        assert!(convert(file_paths).is_ok());
    }

    #[test]
    fn it_works_for_mac_pdfs() {
        let file_paths = vec![
            "test/impulse/impuls_mac.pdf".to_string(),
            "test/impulse/impuls_mac2.pdf".to_string(),
        ];

        assert!(convert(file_paths).is_ok());
    }
}