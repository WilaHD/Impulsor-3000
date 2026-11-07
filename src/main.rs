#![windows_subsystem = "windows"]

pub mod core;
pub mod impuls;
pub mod pdf_engine;
pub mod ui;
mod update_check;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(ui::main()?)
}
