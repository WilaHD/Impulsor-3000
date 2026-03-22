use iced::{
    widget::{button, column, horizontal_rule, row, scrollable, text},
    Alignment::Center,
    Element, Task,
};
use rfd::FileHandle;

use crate::{impuls::Impuls, ui::PdfiumLibState};

use super::super::{file_background, file_banner};

#[derive(Debug, Default)]
pub struct State {
    is_selecting_templates: bool,
    mode: Mode,
}

#[derive(Debug, Default)]
enum Mode {
    #[default]
    Default,
    ShowTemplateValidationResults(Vec<TemplateValidationResult>),
}

#[derive(Debug, PartialEq)]
pub struct TemplateValidationResult {
    template_file: String,
    result: Result<(), Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    GoToWelcome,
    ShowOverview,
    TestTemplateFileDialog,
    TemplatesPicked(Option<Vec<FileHandle>>),
}

pub enum Action {
    None,
    Run(Task<Message>),
    OpenWelcome,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: Message, pdfium: &PdfiumLibState) -> Action {
        match message {
            Message::GoToWelcome => Action::OpenWelcome,
            Message::ShowOverview => {
                self.mode = Mode::Default;
                Action::None
            }
            Message::TestTemplateFileDialog => Action::Run(self.pick_templates()),
            Message::TemplatesPicked(picked_templates) => {
                self.is_selecting_templates = false;

                let Some(picked_templates) = picked_templates else {
                    self.mode = Mode::Default;
                    return Action::None;
                };

                self.mode = Mode::ShowTemplateValidationResults(validate_templates(
                    picked_templates,
                    pdfium,
                ));

                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.is_selecting_templates {
            return file_background::file_plus().into();
        }

        let title = file_banner::banner();

        match &self.mode {
            Mode::Default => column![
                title,
                iced::widget::container(
                    column![
                        text("Einstellungen").size(24),
                        horizontal_rule(1),
                        row![
                            text("Impuls-PDF-Vorlage(n) testen").width(300),
                            button(text(">")).on_press(Message::TestTemplateFileDialog)
                        ]
                        .spacing(20)
                        .align_y(Center),
                        horizontal_rule(1),
                        button("zurück")
                            .style(button::secondary)
                            .on_press(Message::GoToWelcome)
                    ]
                    .spacing(20)
                    .padding(50)
                    .align_x(Center)
                )
            ]
            .align_x(Center)
            .into(),
            Mode::ShowTemplateValidationResults(tvrs) => {
                let mut content = iced::widget::Column::new().spacing(20);

                for (index, tvr) in tvrs.iter().enumerate() {
                    content = content.push(text(format!("{}\n", tvr.template_file)));
                    content = match &tvr.result {
                        Ok(_) => content.push(text("BESTANDEN")),
                        Err(err) => content.push(text(format!(
                            "FEHLER\n{}",
                            err.iter()
                                .map(|message| format!(" - {message}\n"))
                                .collect::<Vec<String>>()
                                .join("")
                        ))),
                    };

                    if index + 1 < tvrs.len() {
                        content = content.push(horizontal_rule(1));
                    }
                }

                column![
                    title,
                    text("Testergebnisse").size(24),
                    horizontal_rule(1),
                    scrollable(content),
                    button(text("zurück").center())
                        .style(button::secondary)
                        .on_press(Message::ShowOverview)
                ]
                .spacing(20)
                .padding(50)
                .align_x(Center)
                .into()
            }
        }
    }

    fn pick_templates(&mut self) -> Task<Message> {
        self.is_selecting_templates = true;

        let extensions = vec!["pdf"];
        let picked_files_future = rfd::AsyncFileDialog::new()
            .set_title("Impuls-PDF-Vorlage(n) auswählen")
            .add_filter("Impuls (.pdf)", &extensions);

        Task::perform(picked_files_future.pick_files(), Message::TemplatesPicked)
    }
}

fn validate_templates(
    picked_templates: Vec<FileHandle>,
    pdfium: &PdfiumLibState,
) -> Vec<TemplateValidationResult> {
    let mut results = vec![];

    for template in picked_templates {
        let path = template.path().to_path_buf();
        let template_file = path.to_string_lossy().to_string();

        let result = match path.extension().and_then(|extension| extension.to_str()) {
            Some("pdf") => {
                let impuls_template_model = crate::impuls::ImpulsModel::build_from_path_buf(&path);

                match pdfium {
                    PdfiumLibState::Ok(pdfium) => {
                        Impuls::build_from_model(&impuls_template_model, pdfium)
                            .map_err(|error| vec![error.to_string()])
                            .and_then(|impuls_template| {
                                impuls_template.test_pdf_form_fields_as_str()
                            })
                    }
                    PdfiumLibState::NotFound(error) => Err(vec![error.to_string()]),
                }
            }
            _ => Err(vec![format!(
                "filetype of file {} is unknown",
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown file")
            )]),
        };

        results.push(TemplateValidationResult {
            template_file,
            result,
        });
    }

    results
}
