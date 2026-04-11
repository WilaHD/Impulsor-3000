use iced::{window, Element, Task, Theme};
use pdfium_render::prelude::*;

pub mod file_assets;
pub mod file_background;
pub mod file_banner;
pub mod file_icons;
mod screens;

use impulsor3000::platform_paths;
use screens::{conversion, settings, welcome};

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
}

struct MainView {
    screen: Screen,
    pdfium: PdfiumLibState,
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Welcome(welcome) => welcome.view(&self.pdfium).map(Message::Welcome),
            Screen::Settings(settings) => settings.view().map(Message::Settings),
            Screen::Conversion(conversion) => conversion.view().map(Message::Conversion),
        }
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }
}

pub fn main() -> iced::Result {
    iced::application(MainView::title, MainView::update, MainView::view)
        .theme(MainView::theme)
        .window(main_window_settings())
        .run_with(MainView::new)
}

fn main_window_settings() -> window::Settings {
    window::Settings {
        icon: load_app_icon(),
        platform_specific: window::settings::PlatformSpecific {
            application_id: WAYLAND_APPLICATION_ID.to_string(),
            ..window::settings::PlatformSpecific::default()
        },
        ..window::Settings::default()
    }
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
                self.screen = Screen::Settings(settings::State::new());
                Task::none()
            }
            welcome::Action::Exit => window::get_latest().and_then(window::close),
        }
    }

    fn update_settings(&mut self, message: settings::Message) -> Task<Message> {
        let Screen::Settings(settings) = &mut self.screen else {
            return Task::none();
        };

        match settings.update(message, &self.pdfium) {
            settings::Action::None => Task::none(),
            settings::Action::Run(task) => task.map(Message::Settings),
            settings::Action::OpenWelcome => {
                self.screen = Screen::Welcome(welcome::State::new());
                Task::none()
            }
        }
    }

    fn update_conversion(&mut self, message: conversion::Message) -> Task<Message> {
        let Screen::Conversion(conversion) = &mut self.screen else {
            return Task::none();
        };

        match conversion.update(message, &self.pdfium) {
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
