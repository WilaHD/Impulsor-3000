#![windows_subsystem = "windows"]

pub mod core;
pub mod impuls;
pub mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(ui::main()?)
}
