pub mod ui;
pub mod impuls;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(ui::main()?)
}