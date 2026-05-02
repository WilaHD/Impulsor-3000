use std::{collections::HashSet, path::PathBuf};

use iced::{
    event,
    widget::{opaque, stack},
    window, Element, Event, Fill, Subscription, Task, Theme,
};
use pdfium_render::prelude::*;

pub mod file_assets;
pub mod file_background;
pub mod file_banner;
pub mod file_icons;
pub mod material_symbols;
mod screens;

use impulsor3000::{
    app_config::{AppConfig, ThemeMode},
    platform_paths,
};
use screens::{conversion, settings, welcome};

#[cfg(target_os = "linux")]
const WAYLAND_APPLICATION_ID: &str = "impulsor3000";

pub enum PdfiumLibState {
    Ok(Pdfium),
    NotFound(String),
}

enum Screen {
    Welcome(welcome::State),
    Settings(settings::State),
    Conversion(conversion::State),
}

#[derive(Debug, Clone)]
pub enum Message {
    Welcome(welcome::Message),
    Settings(settings::Message),
    Conversion(conversion::Message),
    FileHovered(PathBuf),
    FileDropped(PathBuf),
    FilesHoveredLeft,
}

struct MainView {
    screen: Screen,
    pdfium: PdfiumLibState,
    app_config: AppConfig,
    is_drag_hovering: bool,
    hovered_files: Vec<PathBuf>,
    processed_drop_paths: HashSet<PathBuf>,
}

impl MainView {
    fn new() -> (MainView, Task<Message>) {
        let pdfium_lib_state = match platform_paths::pdfium_library_path() {
            Ok(pdfium_path) => match Pdfium::bind_to_library(&pdfium_path) {
                Ok(pdfium) => {
                    let pdfium = Pdfium::new(pdfium);
                    PdfiumLibState::Ok(pdfium)
                }
                Err(e) => PdfiumLibState::NotFound(format!(
                    "Pdfium library not found at {}. Error: {e:?}",
                    pdfium_path.display()
                )),
            },
            Err(e) => PdfiumLibState::NotFound(e),
        };

        (
            Self {
                screen: Screen::Welcome(welcome::State::new()),
                pdfium: pdfium_lib_state,
                app_config: AppConfig::load(),
                is_drag_hovering: false,
                hovered_files: vec![],
                processed_drop_paths: HashSet::new(),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Impulsor 3000")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Welcome(message) => self.update_welcome(message),
            Message::Settings(message) => self.update_settings(message),
            Message::Conversion(message) => self.update_conversion(message),
            Message::FileHovered(path) => {
                self.is_drag_hovering = true;
                self.processed_drop_paths.clear();
                push_unique_path(&mut self.hovered_files, path);
                Task::none()
            }
            Message::FileDropped(path) => {
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
            Screen::Welcome(welcome) => welcome.view(&self.pdfium).map(Message::Welcome),
            Screen::Settings(settings) => settings.view().map(Message::Settings),
            Screen::Conversion(conversion) => conversion.view().map(Message::Conversion),
        };

        if self.is_drag_hovering {
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
                self.screen = Screen::Settings(settings::State::new(self.app_config.theme_mode));
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

            settings.update(message, &self.pdfium)
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
            settings::Action::OpenWelcome => {
                self.screen = Screen::Welcome(welcome::State::new());
                Task::none()
            }
        }
    }

    fn update_conversion(&mut self, message: conversion::Message) -> Task<Message> {
        let action = {
            let Screen::Conversion(conversion) = &mut self.screen else {
                return Task::none();
            };

            conversion.update(message, &self.pdfium)
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
            Screen::Welcome(_) | Screen::Settings(_) => {
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
