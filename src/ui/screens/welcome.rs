use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, text, vertical_space},
    Alignment, Element, Fill,
};

use super::super::{file_banner, PdfiumLibState};

pub struct State;

#[derive(Debug, Clone)]
pub enum Message {
    OpenConversion,
    OpenSettings,
    Exit,
}

pub enum Action {
    OpenConversion,
    OpenSettings,
    Exit,
}

impl State {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::OpenConversion => Action::OpenConversion,
            Message::OpenSettings => Action::OpenSettings,
            Message::Exit => Action::Exit,
        }
    }

    pub fn view(&self, pdfium: &PdfiumLibState) -> Element<'_, Message> {
        let title = file_banner::banner();

        let content = match pdfium {
            PdfiumLibState::Ok(_) => container(
                column![
                    vertical_space().height(100),
                    button(
                        container(text("Impuls-PDF-Datei(en) auswählen"))
                            .center_x(Fill)
                            .center_y(Fill)
                    )
                    .on_press(Message::OpenConversion)
                    .height(100)
                    .width(500),
                    button(text("Einstellungen").center())
                        .on_press(Message::OpenSettings)
                        .width(500)
                        .style(button::secondary),
                    button(text("Beenden").center())
                        .on_press(Message::Exit)
                        .width(500)
                        .style(button::secondary),
                ]
                .spacing(20),
            ),
            PdfiumLibState::NotFound(error_msg) => container(text(error_msg.clone())),
        };

        column![
            title,
            container(content)
                .width(Fill)
                .height(Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        ]
        .align_x(Alignment::Center)
        .into()
    }
}
