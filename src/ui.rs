use iced::{
    alignment::Horizontal,
    widget::{
        button, column, container, horizontal_rule, progress_bar, row, scrollable, text, tooltip,
        vertical_space, Column,
    },
    window,
    Alignment::{self, Center},
    Element, Fill, Task, Theme,
};
use pdfium_render::prelude::*;

pub mod file_assets;
pub mod file_background;
pub mod file_banner;
pub mod file_icons;

use crate::core::impuls_file::ImpulsFileType;
use crate::{
    core::impuls_file::audio::{AudioConvertingState, AudioModel, SUPPORTED_AUDIO_TYPES},
    impuls::{Impuls, ImpulsConvertingState, ImpulsModel},
};
use file_icons::*;
use impulsor3000::choose_pdfium_by_os_arch;
use rfd::FileHandle;

pub enum PdfiumLibState {
    Ok(Pdfium),
    NotFound(String),
}

pub enum CurrentMode {
    Start,
    Locked,
    Default,
    Converting,
    Settings(CurrentModeSettings),
}

#[derive(Debug, PartialEq)]
pub struct TemplateValidationResult {
    template_file: String,
    result: Result<(), Vec<String>>,
}

pub enum CurrentModeSettings {
    None,
    ShowTemplateValidationResults(Vec<TemplateValidationResult>),
}

#[derive(Debug, Clone)]
pub enum Message {
    GoToWelcome,
    Exit,
    ConvertFileDialog,
    ConvertFilesStart(Option<Vec<FileHandle>>),
    ConvertNext,
    ConvertDone,
    FindFile(String),
    Settings(MessageSettings),
}

#[derive(Debug, Clone)]
pub enum MessageSettings {
    None,
    TestTemplateFileDialog,
    TestTemplateValidate(Option<Vec<FileHandle>>),
}

struct MainView {
    file_queue: Vec<ImpulsFileType>,
    current_mode: CurrentMode,
    progress: usize,
    pdfium: PdfiumLibState,
}

impl MainView {
    fn process_file(&mut self) -> Task<Message> {
        if let PdfiumLibState::Ok(pdfium) = &self.pdfium {
            if self.progress < self.file_queue.len() {
                match &mut self.file_queue[self.progress] {
                    ImpulsFileType::Audio(impuls_audio_model) => {
                        println!(
                            "Convert Audio: {}",
                            &impuls_audio_model.get_path_input_str()
                        );
                        impuls_audio_model.convert();
                    }
                    ImpulsFileType::Pdf(impuls_model) => {
                        println!("Load Impuls: {}", &impuls_model.file_path);
                        let impuls_loaded = Impuls::build_from_model(impuls_model, &pdfium);
                        let impuls_loaded = impuls_loaded.unwrap();

                        println!("Build HTML");
                        impuls_model.state_html = match impuls_loaded.save_as_txt(impuls_model) {
                            Ok(_) => ImpulsConvertingState::Success,
                            Err(e) => ImpulsConvertingState::Failure(e.to_string()),
                        };

                        println!("Build Image");
                        impuls_model.state_image = match impuls_loaded.save_as_jpg(impuls_model) {
                            Ok(_) => ImpulsConvertingState::Success,
                            Err(e) => ImpulsConvertingState::Failure(e.to_string()),
                        };
                    }
                    ImpulsFileType::Unknown(file) => {
                        println!("Filetype for file '{file}' is not supported");
                    }
                }

                self.progress += 1;
                return Task::perform(async { () }, |_| Message::ConvertNext);
            } else {
                self.progress += 1;
                return Task::perform(async { () }, |_| Message::ConvertDone);
            }
        } else {
            return Task::perform(async { () }, |_| Message::ConvertDone);
        };
    }

    fn new() -> (MainView, Task<Message>) {
        let pdfium_lib_state = match choose_pdfium_by_os_arch() {
            Ok(pdfium_path) => {
                match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                    &pdfium_path,
                )) {
                    Ok(pdfium) => {
                        let pdfium = Pdfium::new(pdfium);
                        PdfiumLibState::Ok(pdfium)
                    }
                    Err(e) => {
                        panic!("Pdfium library not found at {pdfium_path} \n Error: {e:?}")
                    }
                }
            }
            Err(e) => PdfiumLibState::NotFound(e),
        };

        (
            Self {
                file_queue: vec![],
                current_mode: CurrentMode::Start,
                progress: 0,
                pdfium: pdfium_lib_state,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Impulsor 3000")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GoToWelcome => {
                self.current_mode = CurrentMode::Start;
            }
            Message::ConvertFileDialog => {
                self.current_mode = CurrentMode::Locked;
                let mut extensions = vec!["pdf"];
                extensions.extend(SUPPORTED_AUDIO_TYPES.iter());

                let picked_files_future = rfd::AsyncFileDialog::new()
                    .set_title("Impuls PDF-Datei(en) auswählen")
                    .add_filter("Impuls (.pdf) / Audio (.m4a .ogg .mp3)", &extensions);

                return Task::perform(picked_files_future.pick_files(), Message::ConvertFilesStart);
            }
            Message::ConvertFilesStart(picked_files) => {
                let Some(picked_files) = picked_files else {
                    self.current_mode = CurrentMode::Start;
                    return Task::none();
                };

                self.current_mode = CurrentMode::Converting;
                self.file_queue = vec![];
                for file in picked_files {
                    match file.path().extension().unwrap().to_str().unwrap() {
                        "pdf" => {
                            let impuls = ImpulsModel::build_from_path_buf(&file.into());
                            self.file_queue.push(ImpulsFileType::Pdf(impuls));
                        }
                        x if SUPPORTED_AUDIO_TYPES.contains(&x) => {
                            let impuls_audio = AudioModel::build(file.into());
                            self.file_queue.push(ImpulsFileType::Audio(impuls_audio));
                        }
                        _ => {
                            self.file_queue.push(ImpulsFileType::Unknown(
                                file.path()
                                    .extension()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string(),
                            ));
                        }
                    }
                }

                self.progress = 0;
                return Task::perform(async { () }, |_| Message::ConvertNext);
            }
            Message::ConvertNext => {
                return self.process_file();
            }
            Message::ConvertDone => {
                self.current_mode = CurrentMode::Default;
            }
            Message::Exit => return window::get_latest().and_then(window::close),
            Message::FindFile(file_path) => {
                let _ = opener::reveal(file_path);
            }
            Message::Settings(message) => match &message {
                MessageSettings::None => {
                    self.current_mode = CurrentMode::Settings(CurrentModeSettings::None);
                }
                MessageSettings::TestTemplateFileDialog => {
                    self.current_mode = CurrentMode::Locked;
                    let extensions = vec!["pdf"];

                    let picked_files_future = rfd::AsyncFileDialog::new()
                        .set_title("Impuls-PDF-Vorlage(n) auswählen")
                        .add_filter("Impuls (.pdf)", &extensions);

                    return Task::perform(picked_files_future.pick_files(), |picked_templates| {
                        Message::Settings(MessageSettings::TestTemplateValidate(picked_templates))
                    });
                }
                MessageSettings::TestTemplateValidate(picked_templates) => {
                    let Some(picked_templates) = picked_templates else {
                        self.current_mode = CurrentMode::Settings(CurrentModeSettings::None);
                        return Task::none();
                    };

                    let mut results = vec![];

                    for template in picked_templates {
                        let mut tvr: TemplateValidationResult = TemplateValidationResult {
                            template_file: "unknown file".to_string(),
                            result: Err(vec!["unknown file".to_string()]),
                        };

                        match template.path().extension().unwrap().to_str().unwrap() {
                            "pdf" => {
                                let impuls_template_model =
                                    ImpulsModel::build_from_path_buf(&template.into());

                                match &self.pdfium {
                                    PdfiumLibState::Ok(pdfium) => {
                                        if let Ok(impuls_template) =
                                            Impuls::build_from_model(&impuls_template_model, pdfium)
                                        {
                                            let template_test_result =
                                                impuls_template.test_pdf_form_fields_as_str();

                                            tvr = TemplateValidationResult {
                                                template_file: impuls_template_model.file_path,
                                                result: template_test_result,
                                            };
                                        }
                                    }
                                    PdfiumLibState::NotFound(err) => {
                                        tvr = TemplateValidationResult {
                                            template_file: impuls_template_model.file_path,
                                            result: Err(vec![err.to_string()]),
                                        };
                                    }
                                }
                            }
                            _ => {
                                let file_name = template
                                    .path()
                                    .extension()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                                tvr = TemplateValidationResult {
                                    template_file: file_name.clone(),
                                    result: Err(vec![format!(
                                        "filetype of file {file_name} is unknown"
                                    )]),
                                };
                            }
                        }

                        results.push(tvr);
                    }

                    self.current_mode = CurrentMode::Settings(
                        CurrentModeSettings::ShowTemplateValidationResults(results),
                    )
                }
            },
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let title = file_banner::banner();

        match &self.current_mode {
            CurrentMode::Start => {
                let content = match &self.pdfium {
                    PdfiumLibState::Ok(_) => container(
                        column![
                            vertical_space().height(100),
                            button(
                                container(text("Impuls-PDF-Datei(en) auswählen"))
                                    .center_x(Fill)
                                    .center_y(Fill)
                            )
                            .on_press(Message::ConvertFileDialog)
                            .height(100)
                            .width(500),
                            button(text("Einstellungen").center())
                                .on_press(Message::Settings(MessageSettings::None))
                                .width(500)
                                .style(button::secondary)
                        ]
                        .spacing(20),
                    ),
                    PdfiumLibState::NotFound(error_msg) => container(text(error_msg)),
                };

                column![
                    title,
                    container(content)
                        .width(Fill)
                        .height(Fill)
                        .align_x(Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                ]
                .align_x(Alignment::Center)
                .into()
            }

            CurrentMode::Default | CurrentMode::Converting => {
                let mut content = Column::new().align_x(Alignment::Center).spacing(2);

                for ift in &self.file_queue {
                    match ift {
                        ImpulsFileType::Audio(audio_model) => {
                            let impuls_audio_name = text(audio_model.get_file_name())
                                .align_x(Horizontal::Left)
                                .width(Fill);
                            let impuls_audio_tip = tooltip(
                                impuls_audio_name,
                                text(audio_model.get_path_input_str()),
                                tooltip::Position::FollowCursor,
                            );

                            let impuls_audio_state = match &audio_model.state {
                                AudioConvertingState::Default => build_icon_default(),
                                AudioConvertingState::Success => build_icon_audio_success(),
                                AudioConvertingState::Failure(msg) => build_icon_audio_error(msg),
                            };

                            let find_file = container(
                                button(build_icon_file_search())
                                    .on_press(Message::FindFile(audio_model.get_path_input_str()))
                                    .style(button::secondary)
                                    .width(25)
                                    .height(25)
                                    .padding(0),
                            )
                            .align_x(Horizontal::Right);

                            let r_i = row![impuls_audio_tip, impuls_audio_state, find_file]
                                .spacing(20)
                                .align_y(Center);

                            content = content.push(r_i);
                        }
                        ImpulsFileType::Pdf(i) => {
                            let impuls_name =
                                text(&i.file_name).align_x(Horizontal::Left).width(Fill);
                            let impuls_tip = tooltip(
                                impuls_name,
                                &*i.file_path,
                                tooltip::Position::FollowCursor,
                            );

                            let impuls_state_html = match &i.state_html {
                                ImpulsConvertingState::Default => build_icon_default(),
                                ImpulsConvertingState::Success => build_icon_html_success(),
                                ImpulsConvertingState::Failure(msg) => build_icon_html_error(msg),
                            };

                            let impuls_state_image = match &i.state_image {
                                ImpulsConvertingState::Default => build_icon_default(),
                                ImpulsConvertingState::Success => build_icon_image_success(),
                                ImpulsConvertingState::Failure(msg) => build_icon_image_error(msg),
                            };

                            let impuls_state_html: container::Container<_, _, _> =
                                container(impuls_state_html).align_x(Horizontal::Center);
                            let impuls_state_img =
                                container(impuls_state_image).align_x(Horizontal::Right);

                            let find_file = container(
                                button(build_icon_file_search())
                                    .on_press(Message::FindFile(
                                        i.file_path.replace(".txt", ".pdf").to_string(),
                                    ))
                                    .style(button::secondary)
                                    .width(25)
                                    .height(25)
                                    .padding(0),
                            )
                            .align_x(Horizontal::Right);

                            let r_i =
                                row![impuls_tip, impuls_state_html, impuls_state_img, find_file]
                                    .spacing(20)
                                    .align_y(Center);

                            content = content.push(r_i);
                        }
                        ImpulsFileType::Unknown(file) => {
                            content = content.push(
                                row![text(format!("Filetype for {file} is not supported!"))
                                    .align_x(Horizontal::Left)
                                    .width(Fill)]
                                .spacing(20),
                            )
                        }
                    }
                }

                //control-row
                let control_row = if matches!(self.current_mode, CurrentMode::Converting) {
                    row![
                        progress_bar(0.0..=self.file_queue.len() as f32, self.progress as f32)
                            .width(Fill)
                    ]
                    .spacing(20)
                    .padding(20)
                } else {
                    row![
                        row![
                            button("Neu umwandeln")
                                .on_press(Message::ConvertFileDialog)
                                .style(button::secondary),
                            button("Zurück")
                                .on_press(Message::GoToWelcome)
                                .style(button::secondary)
                        ]
                        .spacing(20)
                        .width(Fill),
                        container(button("Beenden").on_press(Message::Exit))
                            .align_x(Horizontal::Right)
                            .style(container::bordered_box)
                    ]
                    .spacing(20)
                    .padding(20)
                };

                column!(
                    container(title).padding(20),
                    horizontal_rule(1),
                    container(scrollable(content)).padding(20).width(Fill),
                    vertical_space(),
                    horizontal_rule(1),
                    control_row,
                )
                .into()
            }

            CurrentMode::Locked => file_background::file_plus().into(),

            CurrentMode::Settings(mode) => match &mode {
                CurrentModeSettings::None => column![
                    title,
                    container(
                        column![
                            text("Einstellungen").size(24),
                            horizontal_rule(1),
                            row![
                                text("Impuls-PDF-Vorlage(n) testen").width(300),
                                button(text(">")).on_press(Message::Settings(
                                    MessageSettings::TestTemplateFileDialog
                                ))
                            ]
                            .spacing(20)
                            .align_y(Center),
                            horizontal_rule(1),
                            button("zurück")
                                .style(button::secondary)
                                .on_press(Message::GoToWelcome)
                        ]
                        .spacing(20)
                        .align_x(Horizontal::Center)
                    )
                    .width(Fill)
                    .height(Fill)
                ]
                .align_x(Alignment::Center)
                .into(),
                CurrentModeSettings::ShowTemplateValidationResults(tvrs) => {
                    let mut content = column![].spacing(20);
                    for tvr in tvrs {
                        content = content.push(text(format!("{}\n", tvr.template_file)));
                        content = match &tvr.result {
                            Ok(_) => content.push(text("BESTANDEN")),
                            Err(err) => content.push(text(format!(
                                "FEHLER\n{}",
                                err.iter()
                                    .map(|s| format!(" - {s}\n"))
                                    .collect::<Vec<String>>()
                                    .join("")
                            ))),
                        };
                        if tvr != tvrs.last().unwrap() {
                            content = content.push(horizontal_rule(1))
                        }
                    }

                    column![
                        title,
                        text("Testergebnisse").size(24),
                        horizontal_rule(1),
                        scrollable(content),
                        button(text("zurück").center())
                            .style(button::secondary)
                            .on_press(Message::Settings(MessageSettings::None))
                    ]
                    .spacing(20)
                    .align_x(Center)
                    .into()
                }
            },
        }
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }
}

pub fn main() -> iced::Result {
    iced::application(MainView::title, MainView::update, MainView::view)
        .theme(MainView::theme)
        .run_with(MainView::new)
}
