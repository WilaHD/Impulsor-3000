use iced::{
    widget::{horizontal_space, row, text, Row, Text},
    Alignment, Fill, Font,
};

pub const FONT_BYTES: &[u8] =
    include_bytes!("../../fonts/MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf");
pub const FONT: Font = Font::with_name("Material Symbols Rounded");

pub mod icon {
    pub const ADD: char = '\u{e145}';
    pub const ARROW_BACK: char = '\u{e5c4}';
    pub const CLOSE: char = '\u{e5cd}';
    pub const DELETE: char = '\u{e872}';
    pub const INFO: char = '\u{e88e}';
    pub const SETTINGS: char = '\u{e8b8}';
    pub const REDO: char = '\u{e15a}';
}

pub fn symbol(codepoint: char) -> Text<'static> {
    sized(codepoint, 20.0)
}

pub fn sized(codepoint: char, size: f32) -> Text<'static> {
    text(codepoint.to_string()).font(FONT).size(size)
}

pub fn label<'a, Message>(codepoint: char, label: &'a str) -> Row<'a, Message> {
    row![symbol(codepoint), text(label)]
        .spacing(8)
        .align_y(Alignment::Center)
}

pub fn centered_label<'a, Message: 'a>(codepoint: char, label: &'a str) -> Row<'a, Message> {
    row![
        horizontal_space(),
        self::label(codepoint, label),
        horizontal_space()
    ]
    .width(Fill)
    .align_y(Alignment::Center)
}
