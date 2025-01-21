use iced::widget::svg;
use iced::{widget::container, Alignment, Length::Fill};

use super::Message;
use super::file_assets::AssetImages;

pub fn file_plus() -> container::Container<'static, Message> {
    container(
container(
            svg(
                iced::widget::svg::Handle::from_memory(AssetImages::get("file-plus.svg").unwrap().data)
            )
            .width(Fill)
        )
        .max_height(350)
        .center(Fill)
        .align_y(Alignment::Center)
    )
    .height(Fill).align_y(Alignment::Center)
}