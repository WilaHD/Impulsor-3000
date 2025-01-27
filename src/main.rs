#![windows_subsystem = "windows"]

pub mod ui;
pub mod impuls;
pub mod core;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(ui::main()?)
}