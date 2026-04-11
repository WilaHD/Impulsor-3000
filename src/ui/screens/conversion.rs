use std::collections::HashSet;

use iced::{
    alignment::Horizontal,
    widget::{
        button, checkbox, column, container, horizontal_rule, progress_bar, row, scrollable, text,
        tooltip, vertical_space,
    },
    Alignment::{self, Center},
    Element, Fill, Task,
};
use rfd::FileHandle;

use crate::{
    core::impuls_file::{
        audio::{AudioConvertingState, AudioModel, SUPPORTED_AUDIO_TYPES},
        ImpulsFileType,
    },
    impuls::{Impuls, ImpulsConvertingState, ImpulsModel},
    ui::PdfiumLibState,
};

use super::super::{
    file_background, file_banner,
    file_icons::{
        build_icon_audio_error, build_icon_audio_success, build_icon_default,
        build_icon_file_search, build_icon_html_error, build_icon_html_success,
        build_icon_image_error, build_icon_image_success,
    },
};

pub struct State {
    file_queue: Vec<ImpulsFileType>,
    selected_files: Vec<bool>,
    pending_files: Vec<usize>,
    mode: Mode,
    progress: usize,
    is_selecting_files: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Default,
    Converting,
}

#[derive(Debug, Clone)]
pub enum Message {
    ConvertSelected,
    AddFiles,
    FilesPicked(Option<Vec<FileHandle>>),
    AdditionalFilesPicked(Option<Vec<FileHandle>>),
    SetAllFileSelections(bool),
    ToggleFileSelection(usize, bool),
    ConvertNext,
    ConvertDone,
    GoToWelcome,
    Exit,
    RevealFile(String),
}

pub enum Action {
    None,
    Run(Task<Message>),
    OpenWelcome,
    Exit,
    RevealFile(String),
}

impl State {
    pub fn new() -> Self {
        Self {
            file_queue: vec![],
            selected_files: vec![],
            pending_files: vec![],
            mode: Mode::Default,
            progress: 0,
            is_selecting_files: false,
        }
    }

    pub fn request_file_selection(&mut self) -> Task<Message> {
        self.request_files("Impuls-/Audio-Datei(en) auswählen", Message::FilesPicked)
    }

    fn request_file_addition(&mut self) -> Task<Message> {
        self.request_files("Datei(en) hinzufügen", Message::AdditionalFilesPicked)
    }

    fn request_files(
        &mut self,
        title: &'static str,
        map_message: fn(Option<Vec<FileHandle>>) -> Message,
    ) -> Task<Message> {
        self.is_selecting_files = true;

        let mut extensions = vec!["pdf"];
        extensions.extend(SUPPORTED_AUDIO_TYPES.iter().copied());

        let picked_files_future = rfd::AsyncFileDialog::new()
            .set_title(title)
            .add_filter("Impuls (.pdf) / Audio (.m4a .ogg .mp4)", &extensions);

        Task::perform(picked_files_future.pick_files(), map_message)
    }

    pub fn update(&mut self, message: Message, pdfium: &PdfiumLibState) -> Action {
        match message {
            Message::ConvertSelected => self.start_selected_conversion(),
            Message::AddFiles => Action::Run(self.request_file_addition()),
            Message::FilesPicked(picked_files) => {
                self.is_selecting_files = false;

                let Some(picked_files) = picked_files else {
                    self.mode = Mode::Default;

                    if self.file_queue.is_empty() {
                        return Action::OpenWelcome;
                    }

                    return Action::None;
                };

                self.file_queue = build_file_queue(picked_files);
                self.selected_files = vec![true; self.file_queue.len()];

                self.start_selected_conversion()
            }
            Message::AdditionalFilesPicked(picked_files) => {
                self.is_selecting_files = false;

                let Some(picked_files) = picked_files else {
                    return Action::None;
                };

                self.append_files(build_file_queue(picked_files));
                Action::None
            }
            Message::SetAllFileSelections(is_selected) => {
                self.set_all_file_selections(is_selected);
                Action::None
            }
            Message::ToggleFileSelection(index, is_selected) => {
                if let Some(selected) = self.selected_files.get_mut(index) {
                    *selected = is_selected;
                }
                Action::None
            }
            Message::ConvertNext => Action::Run(self.process_file(pdfium)),
            Message::ConvertDone => {
                self.mode = Mode::Default;
                self.pending_files.clear();
                Action::None
            }
            Message::GoToWelcome => Action::OpenWelcome,
            Message::Exit => Action::Exit,
            Message::RevealFile(path) => Action::RevealFile(path),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.is_selecting_files {
            return file_background::file_plus().into();
        }

        let title = file_banner::banner();
        let mut content = iced::widget::Column::new()
            .align_x(Alignment::Center)
            .spacing(2);
        let selection_toolbar: Option<Element<'_, Message>> =
            if self.mode == Mode::Default && !self.file_queue.is_empty() {
                let all_files_selected = self.are_all_files_selected();

                Some(
                    column![
                        row![
                            container(
                                checkbox("", all_files_selected)
                                    .on_toggle(Message::SetAllFileSelections)
                            )
                            .align_x(Horizontal::Center)
                            .width(30),
                            text("Alle").align_x(Horizontal::Left).width(Fill)
                        ]
                        .spacing(20)
                        .align_y(Center)
                        .width(Fill)
                        .padding(
                            iced::Padding::default()
                                .top(20)
                                .right(20)
                                .bottom(12)
                                .left(20),
                        ),
                        horizontal_rule(1),
                    ]
                    .spacing(12)
                    .into(),
                )
            } else {
                None
            };

        if self.file_queue.is_empty() {
            content = content.push(
                text("Keine Dateien ausgewählt")
                    .align_x(Horizontal::Center)
                    .width(Fill),
            );
        } else {
            for (index, file) in self.file_queue.iter().enumerate() {
                let is_selected = self.selected_files.get(index).copied().unwrap_or(false);
                let file_selection = if self.mode == Mode::Default {
                    checkbox("", is_selected).on_toggle(move |is_checked| {
                        Message::ToggleFileSelection(index, is_checked)
                    })
                } else {
                    checkbox("", is_selected)
                };
                let file_selection = container(file_selection)
                    .align_x(Horizontal::Center)
                    .width(30);

                match file {
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
                                .on_press(Message::RevealFile(audio_model.get_path_input_str()))
                                .style(button::secondary)
                                .width(25)
                                .height(25)
                                .padding(0),
                        )
                        .align_x(Horizontal::Right);

                        let row_item = row![
                            file_selection,
                            impuls_audio_tip,
                            impuls_audio_state,
                            find_file
                        ]
                        .spacing(20)
                        .align_y(Center);

                        content = content.push(row_item);
                    }
                    ImpulsFileType::Pdf(model) => {
                        let impuls_name =
                            text(&model.file_name).align_x(Horizontal::Left).width(Fill);
                        let impuls_tip = tooltip(
                            impuls_name,
                            model.file_path.as_str(),
                            tooltip::Position::FollowCursor,
                        );

                        let impuls_state_html = match &model.state_html {
                            ImpulsConvertingState::Default => build_icon_default(),
                            ImpulsConvertingState::Success => build_icon_html_success(),
                            ImpulsConvertingState::Failure(msg) => build_icon_html_error(msg),
                        };

                        let impuls_state_image = match &model.state_image {
                            ImpulsConvertingState::Default => build_icon_default(),
                            ImpulsConvertingState::Success => build_icon_image_success(),
                            ImpulsConvertingState::Failure(msg) => build_icon_image_error(msg),
                        };

                        let impuls_state_html =
                            container(impuls_state_html).align_x(Horizontal::Center);
                        let impuls_state_image =
                            container(impuls_state_image).align_x(Horizontal::Right);

                        let find_file = container(
                            button(build_icon_file_search())
                                .on_press(Message::RevealFile(model.file_path.clone()))
                                .style(button::secondary)
                                .width(25)
                                .height(25)
                                .padding(0),
                        )
                        .align_x(Horizontal::Right);

                        let row_item = row![
                            file_selection,
                            impuls_tip,
                            impuls_state_html,
                            impuls_state_image,
                            find_file
                        ]
                        .spacing(20)
                        .align_y(Center);

                        content = content.push(row_item);
                    }
                    ImpulsFileType::Unknown(file) => {
                        content = content.push(
                            row![
                                file_selection,
                                text(format!("Filetype for {file} is not supported!"))
                                    .align_x(Horizontal::Left)
                                    .width(Fill)
                            ]
                            .spacing(20),
                        );
                    }
                }
            }
        }

        let control_row = if self.mode == Mode::Converting {
            row![
                progress_bar(0.0..=self.pending_files.len() as f32, self.progress as f32)
                    .width(Fill)
            ]
            .spacing(20)
            .padding(20)
        } else {
            let convert_selected_button = {
                let button = button("Neu umwandeln").style(button::secondary);

                if self.has_selected_files() {
                    button.on_press(Message::ConvertSelected)
                } else {
                    button
                }
            };

            row![
                row![
                    convert_selected_button,
                    button("Dateien hinzufügen")
                        .on_press(Message::AddFiles)
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

        column![
            container(title).padding(20),
            horizontal_rule(1),
            selection_toolbar.unwrap_or_else(|| vertical_space().height(0).into()),
            container(scrollable(content)).padding(20).width(Fill),
            vertical_space(),
            horizontal_rule(1),
            control_row,
        ]
        .into()
    }

    fn process_file(&mut self, pdfium: &PdfiumLibState) -> Task<Message> {
        let Some(next_index) = self.pending_files.get(self.progress).copied() else {
            self.progress = self.pending_files.len();
            return Task::perform(async { () }, |_| Message::ConvertDone);
        };

        let Some(next_file) = self.file_queue.get_mut(next_index) else {
            self.progress += 1;
            return self.next_conversion_task();
        };

        match next_file {
            ImpulsFileType::Audio(audio_model) => {
                println!("Convert Audio: {}", audio_model.get_path_input_str());
                audio_model.convert();
            }
            ImpulsFileType::Pdf(impuls_model) => match pdfium {
                PdfiumLibState::Ok(pdfium) => {
                    match Impuls::build_from_model(impuls_model, pdfium) {
                        Ok(impuls_loaded) => {
                            println!("Build HTML");
                            impuls_model.state_html = match impuls_loaded.save_as_txt(impuls_model)
                            {
                                Ok(_) => ImpulsConvertingState::Success,
                                Err(error) => ImpulsConvertingState::Failure(error.to_string()),
                            };

                            println!("Build Image");
                            impuls_model.state_image = match impuls_loaded.save_as_jpg(impuls_model)
                            {
                                Ok(_) => ImpulsConvertingState::Success,
                                Err(error) => ImpulsConvertingState::Failure(error.to_string()),
                            };
                        }
                        Err(error) => {
                            let error = error.to_string();
                            impuls_model.state_html = ImpulsConvertingState::Failure(error.clone());
                            impuls_model.state_image = ImpulsConvertingState::Failure(error);
                        }
                    }
                }
                PdfiumLibState::NotFound(error) => {
                    impuls_model.state_html = ImpulsConvertingState::Failure(error.clone());
                    impuls_model.state_image = ImpulsConvertingState::Failure(error.clone());
                }
            },
            ImpulsFileType::Unknown(file) => {
                println!("Filetype for file '{file}' is not supported");
            }
        }

        self.progress += 1;

        self.next_conversion_task()
    }

    fn append_files(&mut self, files: Vec<ImpulsFileType>) {
        let mut known_paths = self
            .file_queue
            .iter()
            .map(file_queue_entry_path)
            .collect::<HashSet<_>>();
        let mut unique_files = Vec::new();

        for file in files {
            if known_paths.insert(file_queue_entry_path(&file)) {
                unique_files.push(file);
            }
        }

        let additional_file_count = unique_files.len();

        self.file_queue.extend(unique_files);
        self.selected_files
            .extend(std::iter::repeat(true).take(additional_file_count));
    }

    fn has_selected_files(&self) -> bool {
        self.selected_files.iter().any(|is_selected| *is_selected)
    }

    fn are_all_files_selected(&self) -> bool {
        !self.selected_files.is_empty()
            && self.selected_files.iter().all(|is_selected| *is_selected)
    }

    fn set_all_file_selections(&mut self, is_selected: bool) {
        self.selected_files.fill(is_selected);
    }

    fn start_selected_conversion(&mut self) -> Action {
        self.pending_files = self
            .selected_files
            .iter()
            .enumerate()
            .filter_map(|(index, is_selected)| is_selected.then_some(index))
            .collect();

        if self.pending_files.is_empty() {
            return Action::None;
        }

        self.reset_pending_file_states();
        self.progress = 0;
        self.mode = Mode::Converting;

        Action::Run(Task::perform(async { () }, |_| Message::ConvertNext))
    }

    fn reset_pending_file_states(&mut self) {
        for index in self.pending_files.iter().copied() {
            let Some(file) = self.file_queue.get_mut(index) else {
                continue;
            };

            match file {
                ImpulsFileType::Audio(audio_model) => {
                    audio_model.state = AudioConvertingState::Default;
                }
                ImpulsFileType::Pdf(impuls_model) => {
                    impuls_model.state_html = ImpulsConvertingState::Default;
                    impuls_model.state_image = ImpulsConvertingState::Default;
                }
                ImpulsFileType::Unknown(_) => {}
            }
        }
    }

    fn next_conversion_task(&self) -> Task<Message> {
        if self.progress < self.pending_files.len() {
            Task::perform(async { () }, |_| Message::ConvertNext)
        } else {
            Task::perform(async { () }, |_| Message::ConvertDone)
        }
    }
}

fn build_file_queue(picked_files: Vec<FileHandle>) -> Vec<ImpulsFileType> {
    picked_files
        .into_iter()
        .map(|file| {
            let path = file.path().to_path_buf();

            match path.extension().and_then(|extension| extension.to_str()) {
                Some("pdf") => ImpulsFileType::Pdf(ImpulsModel::build_from_path_buf(&path)),
                Some(extension) if SUPPORTED_AUDIO_TYPES.contains(&extension) => {
                    ImpulsFileType::Audio(AudioModel::build(path))
                }
                Some(_) | None => ImpulsFileType::Unknown(path.to_string_lossy().to_string()),
            }
        })
        .collect()
}

fn file_queue_entry_path(file: &ImpulsFileType) -> String {
    match file {
        ImpulsFileType::Audio(audio_model) => audio_model.get_path_input_str(),
        ImpulsFileType::Pdf(model) => model.file_path.clone(),
        ImpulsFileType::Unknown(path) => path.clone(),
    }
}
