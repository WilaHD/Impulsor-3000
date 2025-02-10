use iced::widget::{container, svg, text, tooltip, Tooltip};

use super::file_assets::AssetImages;
use super::Message;

pub fn build_icon_default() -> Tooltip<'static, Message> {
    let handle_image_success = iced::widget::svg::Handle::from_memory(AssetImages::get("default.svg").unwrap().data);
    let svg_image_success = svg(handle_image_success).height(20).width(20);
    
    let tooltip_message = container(text("Verarbeitung ausstehend")).style(container::bordered_box);
    let image_success = tooltip(svg_image_success, tooltip_message, tooltip::Position::Left);
    image_success
}

pub fn build_icon_image_success() -> Tooltip<'static, Message> {
    let handle_image_success = iced::widget::svg::Handle::from_memory(AssetImages::get("image-success.svg").unwrap().data);
    let svg_image_success = svg(handle_image_success).height(20).width(20);
    
    let tooltip_message = container(text("Bild erfolgreich erstellt")).style(container::bordered_box);
    let image_success = tooltip(svg_image_success, tooltip_message, tooltip::Position::Left);
    image_success
}

pub fn build_icon_image_error(error_msg: &str) -> Tooltip<'static, Message> {
    let tooltip_message = format!("Bildumwandlung fehlerhaft! Fehler:\n{error_msg}");

    let handle_image_error = iced::widget::svg::Handle::from_memory(AssetImages::get("image-error.svg").unwrap().data);
    let svg_image_error = svg(handle_image_error).height(20).width(20);
    
    let tooltip_message = container(text(tooltip_message)).style(container::bordered_box);
    let image_error = tooltip(svg_image_error, tooltip_message, tooltip::Position::Left);
    image_error
}

pub fn build_icon_html_success() -> Tooltip<'static, Message> {
    let handle_html_success = iced::widget::svg::Handle::from_memory(AssetImages::get("html-success.svg").unwrap().data);
    let svg_html_success = svg(handle_html_success).height(20).width(20);
    
    let tooltip_message: container::Container<Message, _, iced::Renderer> = container(text("Wordpress-Text erfolgreich erstellt")).style(container::bordered_box);
    let html_success = tooltip(svg_html_success, tooltip_message, tooltip::Position::Left);
    html_success
}

pub fn build_icon_html_error(error_msg: &str) -> Tooltip<'static, Message> {
    let tooltip_message = format!("Wordpress-Text konnte nicht erstellt werden! Fehler:\n{error_msg}");

    let handle_html_error = iced::widget::svg::Handle::from_memory(AssetImages::get("html-error.svg").unwrap().data);
    let svg_html_error = svg(handle_html_error).height(20).width(20);
    
    let tooltip_message: container::Container<Message, _, iced::Renderer> = container(text(tooltip_message)).style(container::bordered_box);
    let html_error: Tooltip<Message, _, iced::Renderer> =  tooltip(svg_html_error, tooltip_message, tooltip::Position::Left);
    html_error
}

pub fn build_icon_audio_success() -> Tooltip<'static, Message> {
    let handle_html_success = iced::widget::svg::Handle::from_memory(AssetImages::get("audio-success.svg").unwrap().data);
    let svg_html_success = svg(handle_html_success).height(20).width(20);
    
    let tooltip_message: container::Container<Message, _, iced::Renderer> = container(text("Audio-Datei erfolgreich umgewandelt")).style(container::bordered_box);
    let html_success = tooltip(svg_html_success, tooltip_message, tooltip::Position::Left);
    html_success
}

pub fn build_icon_audio_error(error_msg: &str) -> Tooltip<'static, Message> {
    let tooltip_message = format!("Audio-Datei konnte nicht umgewandelt werden! Fehler:\n{error_msg}");

    let handle_html_error = iced::widget::svg::Handle::from_memory(AssetImages::get("audio-error.svg").unwrap().data);
    let svg_html_error = svg(handle_html_error).height(20).width(20);
    
    let tooltip_message: container::Container<Message, _, iced::Renderer> = container(text(tooltip_message)).style(container::bordered_box);
    let html_error: Tooltip<Message, _, iced::Renderer> =  tooltip(svg_html_error, tooltip_message, tooltip::Position::Left);
    html_error
}

pub fn build_icon_file_search() -> Tooltip<'static, Message> {
    let tooltip_message = format!("Datei in Verzeichnis anzeigen.");

    let svg_handle = iced::widget::svg::Handle::from_memory(AssetImages::get("folder-search.svg").unwrap().data);
    let svg = svg(svg_handle).height(20).width(20);
    
    let tooltip_styled: container::Container<Message, _, iced::Renderer> = container(text(tooltip_message)).style(container::bordered_box);
    let icon_file_search: Tooltip<Message, _, iced::Renderer> =  tooltip(svg, tooltip_styled, tooltip::Position::Left);
    icon_file_search
}