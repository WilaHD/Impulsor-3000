use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use iced::{
    alignment::Horizontal,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, radio, row, scrollable, text,
        tooltip,
    },
    Alignment::Center,
    Element, Fill, Task,
};
use rfd::FileHandle;

use crate::{impuls::copy_impuls_fields_to_template, ui::PdfiumLibState};

use super::super::{
    file_background, file_banner, file_icons::build_icon_file_search, material_symbols,
};

const CONTENT_WIDTH: u16 = 720;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DestinationMode {
    PrefixNew,
    SelectedDirectory,
    #[default]
    ReplaceAndArchive,
}

#[derive(Debug, Default)]
enum Mode {
    #[default]
    Configure,
    ConfirmOverwrite(Vec<String>),
    Copying,
    Results,
}

#[derive(Debug)]
struct CopyResult {
    source: PathBuf,
    result: Result<PathBuf, String>,
}

#[derive(Debug, Default)]
pub struct State {
    source_files: Vec<PathBuf>,
    template_file: Option<PathBuf>,
    destination_mode: DestinationMode,
    selected_directory: Option<PathBuf>,
    mode: Mode,
    results: Vec<CopyResult>,
    next_source_index: usize,
    is_selecting: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    PickSources,
    SourcesPicked(Option<Vec<FileHandle>>),
    PickTemplate,
    TemplatePicked(Option<FileHandle>),
    DestinationModeSelected(DestinationMode),
    PickDestinationDirectory,
    DestinationDirectoryPicked(Option<FileHandle>),
    Start,
    ConfirmOverwrite,
    CancelOverwrite,
    CopyNext,
    CopyPath(String),
    RevealFile(PathBuf),
    BackToSettings,
}

pub enum Action {
    None,
    Run(Task<Message>),
    RevealFile(PathBuf),
    OpenSettings,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: Message, pdfium: &PdfiumLibState) -> Action {
        match message {
            Message::PickSources => Action::Run(self.pick_sources()),
            Message::SourcesPicked(files) => {
                self.is_selecting = false;
                if let Some(files) = files {
                    self.source_files = unique_paths(file_handles_to_paths(files));
                    self.results.clear();
                }
                Action::None
            }
            Message::PickTemplate => Action::Run(self.pick_template()),
            Message::TemplatePicked(file) => {
                self.is_selecting = false;
                if let Some(file) = file {
                    self.template_file = Some(file.path().to_path_buf());
                    self.results.clear();
                }
                Action::None
            }
            Message::DestinationModeSelected(mode) => {
                self.destination_mode = mode;
                self.results.clear();
                Action::None
            }
            Message::PickDestinationDirectory => Action::Run(self.pick_destination_directory()),
            Message::DestinationDirectoryPicked(directory) => {
                self.is_selecting = false;
                if let Some(directory) = directory {
                    self.selected_directory = Some(directory.path().to_path_buf());
                    self.results.clear();
                }
                Action::None
            }
            Message::Start => self.prepare_start(),
            Message::ConfirmOverwrite => {
                self.mode = Mode::Copying;
                self.results.clear();
                self.next_source_index = 0;
                Action::Run(next_copy_task())
            }
            Message::CancelOverwrite => {
                self.mode = Mode::Configure;
                Action::None
            }
            Message::CopyNext => self.copy_next(pdfium),
            Message::CopyPath(path) => Action::Run(iced::clipboard::write(path)),
            Message::RevealFile(path) => Action::RevealFile(path),
            Message::BackToSettings => Action::OpenSettings,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.is_selecting {
            return file_background::file_plus().into();
        }

        let title = file_banner::banner();
        let content = match &self.mode {
            Mode::Configure => self.with_navigation(
                scrollable(container(self.configuration_view()).center_x(Fill))
                    .width(Fill)
                    .height(Fill)
                    .into(),
                self.configuration_navigation(),
            ),
            Mode::ConfirmOverwrite(conflicts) => self.with_navigation(
                scrollable(container(self.confirmation_view(conflicts)).center_x(Fill))
                    .width(Fill)
                    .height(Fill)
                    .into(),
                self.confirmation_navigation(),
            ),
            Mode::Copying => self.copying_view(),
            Mode::Results => self.results_view(),
        };

        column![
            title,
            container(content).width(Fill).height(Fill).center_x(Fill)
        ]
        .align_x(Center)
        .height(Fill)
        .into()
    }

    fn configuration_view(&self) -> Element<'_, Message> {
        let sources: Element<'_, Message> = if self.source_files.is_empty() {
            text("Keine alten PDF-Dateien ausgewählt").into()
        } else {
            self.source_files
                .iter()
                .fold(column!().spacing(4), |list, path| {
                    list.push(text(path.display().to_string()))
                })
                .into()
        };

        let template = self
            .template_file
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Keine neue Vorlage ausgewählt".to_string());
        let destination_directory = self
            .selected_directory
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Kein Verzeichnis ausgewählt".to_string());
        let mut content = column![
            text("Kopie in neue Vorlage").size(24),
            horizontal_rule(1),
            text("1. Alte PDF-Dateien"),
            button("PDF-Dateien auswählen").on_press(Message::PickSources),
            sources,
            horizontal_rule(1),
            text("2. Neue Vorlage"),
            button("Neue PDF-Vorlage auswählen").on_press(Message::PickTemplate),
            text(template),
            horizontal_rule(1),
            text("3. Zielverzeichnis"),
            radio(
                "Selbes Verzeichnis, neuer Dateiname beginnt mit NEU_",
                DestinationMode::PrefixNew,
                Some(self.destination_mode),
                Message::DestinationModeSelected,
            ),
            radio(
                "Verzeichnis auswählen",
                DestinationMode::SelectedDirectory,
                Some(self.destination_mode),
                Message::DestinationModeSelected,
            ),
            radio(
                "Selbes Verzeichnis, alte Dateien nach old/ verschieben (Standard)",
                DestinationMode::ReplaceAndArchive,
                Some(self.destination_mode),
                Message::DestinationModeSelected,
            ),
        ]
        .spacing(16);

        if self.destination_mode == DestinationMode::SelectedDirectory {
            content = content.push(
                column![
                    button("Zielverzeichnis auswählen").on_press(Message::PickDestinationDirectory),
                    text(destination_directory),
                ]
                .spacing(8),
            );
        }

        content.padding(50).width(CONTENT_WIDTH).into()
    }

    fn confirmation_view(&self, conflicts: &[String]) -> Element<'_, Message> {
        let conflict_list = conflicts
            .iter()
            .fold(column!().spacing(8), |list, conflict| {
                list.push(text(conflict.clone()))
            });

        column![
            text("Vorhandene Dateien überschreiben?").size(24),
            text("Die folgenden Dateien werden ersetzt. Sie können den Vorgang noch abbrechen."),
            scrollable(conflict_list).height(260),
        ]
        .spacing(20)
        .padding(50)
        .width(CONTENT_WIDTH)
        .into()
    }

    fn copying_view(&self) -> Element<'_, Message> {
        let current = self.next_source_index.min(self.source_files.len()) + 1;
        column![
            text("Kopie in neue Vorlage").size(24),
            text(format!(
                "Datei {current} von {} wird verarbeitet …",
                self.source_files.len()
            )),
        ]
        .spacing(20)
        .padding(50)
        .width(CONTENT_WIDTH)
        .into()
    }

    fn results_view(&self) -> Element<'_, Message> {
        let results = self
            .results
            .iter()
            .fold(column!().spacing(16), |list, result| {
                let (reveal_path, error) = match &result.result {
                    Ok(destination) => (destination.clone(), None),
                    Err(error) => (result.source.clone(), Some(error.clone())),
                };
                let path_text = reveal_path.display().to_string();
                let file_name = reveal_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| path_text.clone());
                let file_name = tooltip(
                    button(text(file_name))
                        .style(button::text)
                        .on_press(Message::CopyPath(path_text.clone())),
                    container(text(path_text)).style(container::bordered_box),
                    tooltip::Position::FollowCursor,
                );
                let reveal_button = button(build_icon_file_search())
                    .style(button::secondary)
                    .width(30)
                    .height(30)
                    .padding(0)
                    .on_press(Message::RevealFile(reveal_path));
                let status: Element<'_, Message> = if error.is_some() {
                    text("FEHLER").into()
                } else {
                    material_symbols::symbol(material_symbols::icon::CHECK).into()
                };
                let file_row = row![
                    status,
                    file_name,
                    container(reveal_button)
                        .width(Fill)
                        .align_x(Horizontal::Right),
                ]
                .spacing(8)
                .align_y(Center)
                .width(Fill);
                let details: Element<'_, Message> = match error {
                    Some(error) => column![file_row, text(error)].spacing(4).into(),
                    None => file_row.into(),
                };
                list.push(column![details, horizontal_rule(1),].spacing(16))
            });

        column![
            container(text("Ergebnis: Kopie in neue Vorlage").size(24)).padding(20),
            horizontal_rule(1),
            container(scrollable(results))
                .padding(20)
                .width(Fill)
                .height(Fill),
            horizontal_rule(1),
            container(button("Zurück zu den Einstellungen").on_press(Message::BackToSettings))
                .width(Fill)
                .padding(20)
                .align_x(Horizontal::Right),
        ]
        .height(Fill)
        .into()
    }

    fn with_navigation<'a>(
        &self,
        content: Element<'a, Message>,
        navigation: Element<'a, Message>,
    ) -> Element<'a, Message> {
        column![
            container(content).width(Fill).height(Fill),
            horizontal_rule(1),
            container(navigation).width(Fill).padding(20),
        ]
        .height(Fill)
        .into()
    }

    fn configuration_navigation(&self) -> Element<'_, Message> {
        let can_start = !self.source_files.is_empty()
            && self.template_file.is_some()
            && (self.destination_mode != DestinationMode::SelectedDirectory
                || self.selected_directory.is_some());
        let start = if can_start {
            button("Kopiervorgang starten").on_press(Message::Start)
        } else {
            button("Kopiervorgang starten")
        };

        row![
            start,
            horizontal_space(),
            button("Zurück zu den Einstellungen")
                .style(button::secondary)
                .on_press(Message::BackToSettings),
        ]
        .spacing(20)
        .align_y(Center)
        .width(Fill)
        .into()
    }

    fn confirmation_navigation(&self) -> Element<'_, Message> {
        row![
            button("Abbrechen")
                .style(button::secondary)
                .on_press(Message::CancelOverwrite),
            horizontal_space(),
            button("Überschreiben und starten").on_press(Message::ConfirmOverwrite),
        ]
        .spacing(20)
        .align_y(Center)
        .width(Fill)
        .into()
    }

    fn pick_sources(&mut self) -> Task<Message> {
        self.is_selecting = true;
        let dialog = rfd::AsyncFileDialog::new()
            .set_title("Alte Impuls-PDF-Dateien auswählen")
            .add_filter("PDF (.pdf)", &["pdf"]);
        Task::perform(dialog.pick_files(), Message::SourcesPicked)
    }

    fn pick_template(&mut self) -> Task<Message> {
        self.is_selecting = true;
        let dialog = rfd::AsyncFileDialog::new()
            .set_title("Neue Impuls-PDF-Vorlage auswählen")
            .add_filter("PDF (.pdf)", &["pdf"]);
        Task::perform(dialog.pick_file(), Message::TemplatePicked)
    }

    fn pick_destination_directory(&mut self) -> Task<Message> {
        self.is_selecting = true;
        Task::perform(
            rfd::AsyncFileDialog::new()
                .set_title("Zielverzeichnis auswählen")
                .pick_folder(),
            Message::DestinationDirectoryPicked,
        )
    }

    fn prepare_start(&mut self) -> Action {
        let conflicts = self.planned_conflicts();
        if conflicts.is_empty() {
            self.mode = Mode::Copying;
            self.results.clear();
            self.next_source_index = 0;
            Action::Run(next_copy_task())
        } else {
            self.mode = Mode::ConfirmOverwrite(conflicts);
            Action::None
        }
    }

    fn copy_next(&mut self, pdfium: &PdfiumLibState) -> Action {
        let Some(source) = self.source_files.get(self.next_source_index).cloned() else {
            self.mode = Mode::Results;
            return Action::None;
        };

        let result = self.copy_source(&source, pdfium);
        self.results.push(CopyResult { source, result });
        self.next_source_index += 1;

        if self.next_source_index < self.source_files.len() {
            Action::Run(next_copy_task())
        } else {
            self.mode = Mode::Results;
            Action::None
        }
    }

    fn copy_source(&self, source: &Path, state: &PdfiumLibState) -> Result<PathBuf, String> {
        let pdfium = match state {
            PdfiumLibState::Ok(pdfium) => pdfium,
            PdfiumLibState::NotFound(error) => return Err(error.clone()),
        };
        let template = self
            .template_file
            .as_ref()
            .ok_or_else(|| "Keine neue Vorlage ausgewählt.".to_string())?;
        let destination = self.destination_for(source)?;
        let temporary = temporary_path_near(&destination)?;

        if let Err(error) = copy_impuls_fields_to_template(source, template, &temporary, pdfium) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        let commit_result = match self.destination_mode {
            DestinationMode::ReplaceAndArchive => {
                self.commit_with_archive(source, &destination, &temporary)
            }
            DestinationMode::PrefixNew | DestinationMode::SelectedDirectory => {
                replace_with_temporary(&temporary, &destination)
            }
        };

        if let Err(error) = commit_result {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }

        Ok(destination)
    }

    fn commit_with_archive(
        &self,
        source: &Path,
        destination: &Path,
        temporary: &Path,
    ) -> Result<(), String> {
        let archive = archive_path(source)?;
        let archive_directory = archive
            .parent()
            .ok_or_else(|| "Archivordner konnte nicht bestimmt werden.".to_string())?;
        fs::create_dir_all(archive_directory)
            .map_err(|error| format!("Archivordner konnte nicht erstellt werden: {error}"))?;

        if archive.exists() {
            fs::remove_file(&archive).map_err(|error| {
                format!("Vorhandene Archivdatei konnte nicht ersetzt werden: {error}")
            })?;
        }
        fs::rename(source, &archive).map_err(|error| {
            format!("Alte PDF konnte nicht nach old/ verschoben werden: {error}")
        })?;

        if let Err(error) = fs::rename(temporary, destination) {
            let restore_result = fs::rename(&archive, source);
            return Err(match restore_result {
                Ok(_) => format!("Neue PDF konnte nicht an ihr Ziel verschoben werden: {error}"),
                Err(restore_error) => format!(
                    "Neue PDF konnte nicht an ihr Ziel verschoben werden: {error}. Die alte PDF konnte nicht zurückgestellt werden: {restore_error}"
                ),
            });
        }

        Ok(())
    }

    fn destination_for(&self, source: &Path) -> Result<PathBuf, String> {
        let file_name = source
            .file_name()
            .ok_or_else(|| "Dateiname der Quell-PDF konnte nicht bestimmt werden.".to_string())?;
        let source_directory = source
            .parent()
            .ok_or_else(|| "Quellverzeichnis konnte nicht bestimmt werden.".to_string())?;

        match self.destination_mode {
            DestinationMode::PrefixNew => {
                Ok(source_directory.join(format!("NEU_{}", file_name.to_string_lossy())))
            }
            DestinationMode::SelectedDirectory => self
                .selected_directory
                .as_ref()
                .map(|directory| directory.join(file_name))
                .ok_or_else(|| "Kein Zielverzeichnis ausgewählt.".to_string()),
            DestinationMode::ReplaceAndArchive => Ok(source.to_path_buf()),
        }
    }

    fn planned_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();
        let mut planned_destinations: HashMap<PathBuf, PathBuf> = HashMap::new();

        for source in &self.source_files {
            let Ok(destination) = self.destination_for(source) else {
                continue;
            };

            if self.destination_mode != DestinationMode::ReplaceAndArchive && destination.exists() {
                conflicts.push(destination.display().to_string());
            }
            if self.destination_mode == DestinationMode::ReplaceAndArchive {
                if let Ok(archive) = archive_path(source) {
                    if archive.exists() {
                        conflicts.push(archive.display().to_string());
                    }
                }
            }
            if let Some(previous_source) = planned_destinations.insert(destination, source.clone())
            {
                conflicts.push(format!(
                    "{} wird durch das spätere Ergebnis von {} ersetzt.",
                    previous_source.display(),
                    source.display()
                ));
            }
        }

        unique_strings(conflicts)
    }
}

fn next_copy_task() -> Task<Message> {
    Task::perform(async { () }, |_| Message::CopyNext)
}

fn file_handles_to_paths(files: Vec<FileHandle>) -> Vec<PathBuf> {
    files
        .into_iter()
        .map(|file| file.path().to_path_buf())
        .collect()
}

fn unique_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths
        .into_iter()
        .filter(|path| seen.insert(path.clone()))
        .collect()
}

fn unique_strings(strings: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    strings
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn archive_path(source: &Path) -> Result<PathBuf, String> {
    let directory = source
        .parent()
        .ok_or_else(|| "Quellverzeichnis konnte nicht bestimmt werden.".to_string())?;
    let file_name = source
        .file_name()
        .ok_or_else(|| "Dateiname der Quell-PDF konnte nicht bestimmt werden.".to_string())?;
    Ok(directory.join("old").join(file_name))
}

fn temporary_path_near(destination: &Path) -> Result<PathBuf, String> {
    let directory = destination
        .parent()
        .ok_or_else(|| "Zielverzeichnis konnte nicht bestimmt werden.".to_string())?;
    let file_name = destination
        .file_name()
        .ok_or_else(|| "Zieldateiname konnte nicht bestimmt werden.".to_string())?
        .to_string_lossy();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Temporärer Dateiname konnte nicht erstellt werden: {error}"))?
        .as_nanos();

    for attempt in 0..1000 {
        let temporary = directory.join(format!(".{file_name}.impulsor-{timestamp}-{attempt}.tmp"));
        if !temporary.exists() {
            return Ok(temporary);
        }
    }

    Err("Es konnte kein freier temporärer Dateiname erstellt werden.".to_string())
}

fn replace_with_temporary(temporary: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        fs::remove_file(destination).map_err(|error| {
            format!("Vorhandene Zieldatei konnte nicht ersetzt werden: {error}")
        })?;
    }
    fs::rename(temporary, destination)
        .map_err(|error| format!("Neue PDF konnte nicht an ihr Ziel verschoben werden: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_path_uses_old_subdirectory() {
        assert_eq!(
            archive_path(Path::new("/input/impuls.pdf")).unwrap(),
            PathBuf::from("/input/old/impuls.pdf")
        );
    }

    #[test]
    fn unique_paths_preserves_selection_order() {
        let paths = unique_paths(vec![
            PathBuf::from("one.pdf"),
            PathBuf::from("two.pdf"),
            PathBuf::from("one.pdf"),
        ]);
        assert_eq!(
            paths,
            vec![PathBuf::from("one.pdf"), PathBuf::from("two.pdf")]
        );
    }

    #[test]
    fn destination_paths_follow_the_selected_mode() {
        let source = Path::new("/input/impuls.pdf");
        let mut state = State::new();

        state.destination_mode = DestinationMode::PrefixNew;
        assert_eq!(
            state.destination_for(source).unwrap(),
            PathBuf::from("/input/NEU_impuls.pdf")
        );

        state.destination_mode = DestinationMode::SelectedDirectory;
        state.selected_directory = Some(PathBuf::from("/output"));
        assert_eq!(
            state.destination_for(source).unwrap(),
            PathBuf::from("/output/impuls.pdf")
        );

        state.destination_mode = DestinationMode::ReplaceAndArchive;
        assert_eq!(state.destination_for(source).unwrap(), source);
    }

    #[test]
    fn replacing_with_temporary_replaces_existing_file() {
        let directory = std::env::temp_dir().join(format!(
            "impulsor-template-copy-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        let temporary = directory.join("temporary.pdf");
        let destination = directory.join("destination.pdf");
        fs::write(&temporary, b"new").unwrap();
        fs::write(&destination, b"old").unwrap();

        replace_with_temporary(&temporary, &destination).unwrap();

        assert_eq!(fs::read(&destination).unwrap(), b"new");
        assert!(!temporary.exists());
        fs::remove_dir_all(directory).unwrap();
    }
}
