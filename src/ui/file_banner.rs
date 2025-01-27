use iced::{widget::{container, svg}, Length::Fill};

use super::Message;
use super::file_assets::AssetImages;

pub fn banner() -> container::Container<'static, Message> {
    let title = container(
        svg(
            iced::widget::svg::Handle::from_memory(AssetImages::get("banner.svg").unwrap().data)
        )
        .width(Fill)
    )
        .max_height(150)
        .center_x(Fill);
    title
}