use std::{collections::HashSet, path::PathBuf};

use crate::{
    pdf_engine::PdfEngine,
    update_check::{self, UpdateInfo},
};
use iced::{
    event,
    widget::{button, column, container, opaque, row, stack, text},
    window, Element, Event, Fill, Subscription, Task, Theme,
};

pub mod file_assets;
pub mod file_background;
pub mod file_banner;
pub mod file_icons;
pub mod material_symbols;
mod screens;

use impulsor3000::app_config::{AppConfig, ThemeMode};
use screens::{conversion, settings, template_copy, welcome};

#[cfg(target_os = "linux")]
const WAYLAND_APPLICATION_ID: &str = "impulsor3000";

pub enum PdfEngineState {
    Ok(PdfEngine),
    NotFound(String),
}

enum Screen {
    Welcome(welcome::State),
    Settings(settings::State),
    Conversion(conversion::State),
    TemplateCopy(template_copy::State),
}

#[derive(Debug, Clone)]
pub enum Message {
    Welcome(welcome::Message),
    Settings(settings::Message),
    Conversion(conversion::Message),
    TemplateCopy(template_copy::Message),
    FileHovered(PathBuf),
    FileDropped(PathBuf),
    FilesHoveredLeft,
    UpdateChecked(Option<UpdateInfo>),
    DismissUpdate,
    OpenUpdatePage,
}

struct MainView {
    screen: Screen,
    pdf_engine: PdfEngineState,
    app_config: AppConfig,
    is_drag_hovering: bool,
    hovered_files: Vec<PathBuf>,
    processed_drop_paths: HashSet<PathBuf>,
    update_info: Option<UpdateInfo>,
}

impl MainView {
    fn new() -> (MainView, Task<Message>) {
        let app_config = AppConfig::load();
        let mut pdf_engine = PdfEngine::new();
        pdf_engine.set_line_spacing_percent(app_config.line_spacing_percent);
        let pdf_engine_lib_state = PdfEngineState::Ok(pdf_engine);

        (
            Self {
                screen: Screen::Welcome(welcome::State::new()),
                pdf_engine: pdf_engine_lib_state,
                app_config,
                is_drag_hovering: false,
                hovered_files: vec![],
                processed_drop_paths: HashSet::new(),
                update_info: None,
            },
            Task::perform(
                async {
                    let (sender, receiver) = futures_channel::oneshot::channel();
                    std::thread::spawn(move || {
                        let _ = sender.send(update_check::check());
                    });
                    receiver.await.unwrap_or(None)
                },
                Message::UpdateChecked,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Impulsor 3000")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateChecked(info) => {
                self.update_info = info;
                if self.update_info.is_some() {
                    self.is_drag_hovering = false;
                    self.hovered_files.clear();
                    self.processed_drop_paths.clear();
                }
                Task::none()
            }
            Message::DismissUpdate => {
                self.update_info = None;
                Task::none()
            }
            Message::OpenUpdatePage => {
                let _ = opener::open(update_check::RELEASES_URL);
                Task::none()
            }
            Message::Welcome(message) => self.update_welcome(message),
            Message::Settings(message) => self.update_settings(message),
            Message::Conversion(message) => self.update_conversion(message),
            Message::TemplateCopy(message) => self.update_template_copy(message),
            Message::FileHovered(path) => {
                if self.update_info.is_some() {
                    return Task::none();
                }
                self.is_drag_hovering = true;
                self.processed_drop_paths.clear();
                push_unique_path(&mut self.hovered_files, path);
                Task::none()
            }
            Message::FileDropped(path) => {
                if self.update_info.is_some() {
                    return Task::none();
                }
                if self.processed_drop_paths.remove(&path) {
                    return Task::none();
                }

                let mut dropped_files = if self.hovered_files.is_empty() {
                    vec![path.clone()]
                } else {
                    std::mem::take(&mut self.hovered_files)
                };

                push_unique_path(&mut dropped_files, path);
                self.processed_drop_paths = dropped_files.iter().cloned().collect();
                self.is_drag_hovering = false;

                self.handle_dropped_files(dropped_files)
            }
            Message::FilesHoveredLeft => {
                self.is_drag_hovering = false;
                self.hovered_files.clear();
                self.processed_drop_paths.clear();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = match &self.screen {
            Screen::Welcome(welcome) => welcome.view(&self.pdf_engine).map(Message::Welcome),
            Screen::Settings(settings) => settings.view().map(Message::Settings),
            Screen::Conversion(conversion) => conversion.view().map(Message::Conversion),
            Screen::TemplateCopy(template_copy) => template_copy.view().map(Message::TemplateCopy),
        };

        let content = if self.is_drag_hovering {
            stack([
                content,
                opaque(
                    iced::widget::container(file_background::file_plus::<Message>())
                        .width(Fill)
                        .height(Fill),
                ),
            ])
            .width(Fill)
            .height(Fill)
            .into()
        } else {
            content
        };

        if let Some(info) = &self.update_info {
            let dialog = container(
                column![
                    text("Neue Version verfügbar").size(24),
                    text(format!(
                        "Installierte Version: {}\nNeue Version: {}",
                        info.current_version, info.latest_version
                    )),
                    text("Eine neue Version von Impulsor 3000 ist verfügbar."),
                    row![
                        button("Zur Downloadseite").on_press(Message::OpenUpdatePage),
                        button("OK").on_press(Message::DismissUpdate),
                    ]
                    .spacing(12),
                ]
                .spacing(16),
            )
            .padding(24)
            .max_width(520)
            .style(container::bordered_box);
            stack([
                content,
                opaque(container(dialog).width(Fill).height(Fill).center(Fill)),
            ])
            .width(Fill)
            .height(Fill)
            .into()
        } else {
            content
        }
    }

    fn theme(&self) -> Theme {
        match self.app_config.theme_mode {
            ThemeMode::Auto => Theme::default(),
            ThemeMode::Light => Theme::Light,
            ThemeMode::Dark => Theme::Dark,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(handle_window_event)
    }
}

pub fn main() -> iced::Result {
    prefer_x11_when_wayland_file_drop_is_unavailable();

    iced::application(MainView::title, MainView::update, MainView::view)
        .font(material_symbols::FONT_BYTES)
        .theme(MainView::theme)
        .window(main_window_settings())
        .subscription(MainView::subscription)
        .run_with(MainView::new)
}

fn prefer_x11_when_wayland_file_drop_is_unavailable() {
    #[cfg(target_os = "linux")]
    {
        // `winit` 0.30 handles dropped files on X11, but not on Wayland.
        // When both backends are available, prefer X11 unless the user explicitly opts out.
        let prefer_wayland = std::env::var_os("IMPULSOR_PREFER_WAYLAND").is_some();
        let has_x11 = std::env::var_os("DISPLAY")
            .map(|value| !value.is_empty())
            .unwrap_or(false);
        let has_wayland = ["WAYLAND_DISPLAY", "WAYLAND_SOCKET"]
            .into_iter()
            .any(|name| {
                std::env::var_os(name)
                    .map(|value| !value.is_empty())
                    .unwrap_or(false)
            });

        if !prefer_wayland && has_x11 && has_wayland {
            std::env::remove_var("WAYLAND_DISPLAY");
            std::env::remove_var("WAYLAND_SOCKET");
        }
    }
}

fn main_window_settings() -> window::Settings {
    window::Settings {
        icon: load_app_icon(),
        platform_specific: main_window_platform_settings(),
        ..window::Settings::default()
    }
}

#[cfg(target_os = "linux")]
fn main_window_platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        application_id: WAYLAND_APPLICATION_ID.to_string(),
        ..window::settings::PlatformSpecific::default()
    }
}

#[cfg(not(target_os = "linux"))]
fn main_window_platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific::default()
}

fn load_app_icon() -> Option<window::Icon> {
    let pixels = image::load_from_memory_with_format(
        include_bytes!("../imgs/logo.png"),
        image::ImageFormat::Png,
    )
    .ok()?
    .to_rgba8();

    window::icon::from_rgba(pixels.to_vec(), pixels.width(), pixels.height()).ok()
}

fn handle_window_event(
    event: Event,
    _status: event::Status,
    _window: window::Id,
) -> Option<Message> {
    match event {
        Event::Window(window::Event::Resized(size)) => Some(Message::Conversion(
            conversion::Message::WindowResized(size.width),
        )),
        Event::Window(window::Event::FileHovered(path)) => Some(Message::FileHovered(path)),
        Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
        Event::Window(window::Event::FilesHoveredLeft) => Some(Message::FilesHoveredLeft),
        _ => None,
    }
}

impl MainView {
    fn update_welcome(&mut self, message: welcome::Message) -> Task<Message> {
        let Screen::Welcome(welcome) = &mut self.screen else {
            return Task::none();
        };

        match welcome.update(message) {
            welcome::Action::OpenConversion => {
                let mut conversion = conversion::State::new();
                let task = conversion.request_file_selection();

                self.screen = Screen::Conversion(conversion);

                task.map(Message::Conversion)
            }
            welcome::Action::OpenSettings => {
                self.screen = Screen::Settings(settings::State::new(
                    self.app_config.theme_mode,
                    self.app_config.line_spacing_percent,
                ));
                Task::none()
            }
            welcome::Action::Exit => window::get_latest().and_then(window::close),
        }
    }

    fn update_settings(&mut self, message: settings::Message) -> Task<Message> {
        let action = {
            let Screen::Settings(settings) = &mut self.screen else {
                return Task::none();
            };

            settings.update(message, &self.pdf_engine)
        };

        match action {
            settings::Action::None => Task::none(),
            settings::Action::Run(task) => task.map(Message::Settings),
            settings::Action::SaveThemeMode(theme_mode) => {
                self.app_config.theme_mode = theme_mode;

                let save_error = self.app_config.save().err();
                if let Screen::Settings(settings) = &mut self.screen {
                    settings.set_settings_error(save_error);
                }

                Task::none()
            }
            settings::Action::SaveLineSpacing(percent) => {
                self.app_config.line_spacing_percent = percent;
                if let PdfEngineState::Ok(engine) = &mut self.pdf_engine {
                    engine.set_line_spacing_percent(percent);
                }
                let save_error = self.app_config.save().err();
                if let Screen::Settings(settings) = &mut self.screen {
                    settings.set_settings_error(save_error);
                }
                Task::none()
            }
            settings::Action::OpenWelcome => {
                self.screen = Screen::Welcome(welcome::State::new());
                Task::none()
            }
            settings::Action::OpenTemplateCopy => {
                self.screen = Screen::TemplateCopy(template_copy::State::new());
                Task::none()
            }
        }
    }

    fn update_template_copy(&mut self, message: template_copy::Message) -> Task<Message> {
        let action = {
            let Screen::TemplateCopy(template_copy) = &mut self.screen else {
                return Task::none();
            };

            template_copy.update(message, &self.pdf_engine)
        };

        match action {
            template_copy::Action::None => Task::none(),
            template_copy::Action::Run(task) => task.map(Message::TemplateCopy),
            template_copy::Action::RevealFile(path) => {
                let _ = opener::reveal(path);
                Task::none()
            }
            template_copy::Action::OpenSettings => {
                self.screen = Screen::Settings(settings::State::new(
                    self.app_config.theme_mode,
                    self.app_config.line_spacing_percent,
                ));
                Task::none()
            }
        }
    }

    fn update_conversion(&mut self, message: conversion::Message) -> Task<Message> {
        let action = {
            let Screen::Conversion(conversion) = &mut self.screen else {
                return Task::none();
            };

            conversion.update(message, &self.pdf_engine)
        };

        self.apply_conversion_action(action)
    }

    fn handle_dropped_files(&mut self, paths: Vec<PathBuf>) -> Task<Message> {
        let action = match &mut self.screen {
            Screen::Conversion(conversion) => {
                if conversion.has_files() {
                    conversion.add_files_from_paths(paths)
                } else {
                    conversion.replace_files_from_paths(paths)
                }
            }
            Screen::Welcome(_) | Screen::Settings(_) | Screen::TemplateCopy(_) => {
                let mut conversion = conversion::State::new();
                let action = conversion.replace_files_from_paths(paths);
                self.screen = Screen::Conversion(conversion);
                action
            }
        };

        self.apply_conversion_action(action)
    }

    fn apply_conversion_action(&mut self, action: conversion::Action) -> Task<Message> {
        match action {
            conversion::Action::None => Task::none(),
            conversion::Action::Run(task) => task.map(Message::Conversion),
            conversion::Action::OpenWelcome => {
                self.screen = Screen::Welcome(welcome::State::new());
                Task::none()
            }
            conversion::Action::Exit => window::get_latest().and_then(window::close),
            conversion::Action::RevealFile(path) => {
                let _ = opener::reveal(path);
                Task::none()
            }
        }
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing_path| existing_path == &path) {
        paths.push(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_notice_is_shown_and_can_be_dismissed() {
        let (mut app, _startup_task) = MainView::new();
        assert!(app.update_info.is_none());

        let _ = app.update(Message::UpdateChecked(None));
        assert!(app.update_info.is_none());

        let info = UpdateInfo {
            current_version: "1.0.0".into(),
            latest_version: "v2.0.0".into(),
        };
        let _ = app.update(Message::UpdateChecked(Some(info.clone())));
        assert_eq!(app.update_info, Some(info));
        let _ = app.view();

        let _ = app.update(Message::FileHovered(PathBuf::from("example.pdf")));
        assert!(!app.is_drag_hovering);
        let _ = app.update(Message::DismissUpdate);
        assert!(app.update_info.is_none());
        let _ = app.update(Message::FileHovered(PathBuf::from("example.pdf")));
        assert!(app.is_drag_hovering);
    }
}
