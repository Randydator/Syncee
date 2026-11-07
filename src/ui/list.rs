use super::gui::GuiMessage;
use super::gui::SynceeGui;
use crate::types::SyncConfig;
use crate::utils::config;
use chrono::DateTime;
use chrono::Utc;
use iced::Alignment;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::widget::Column;
use iced::widget::Text;
use iced::widget::scrollable;
use iced::widget::{button, column, row, text};
use uuid::Uuid;

pub fn draw_list_item(list_item: &SyncConfig) -> Element<'_, GuiMessage> {
    // 1. Style the title (bold, larger size)
    let title_text = text(&list_item.title).size(16).font(Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    });

    // 2. Style the description (smaller, lighter/subtle color)
    let desc_text = text(&list_item.description).size(12);

    // Group title and description vertically in a column
    let info_column = column![title_text, desc_text].spacing(4);

    // Format the timestamp nicely
    let time_str = list_item.last_synced.format("%Y-%m-%d %H:%M").to_string();
    let time_text = text(time_str).size(11);

    // 3. Combine everything into a horizontal row
    row![
        info_column.width(Length::Fill), // Takes up available space
        time_text,
        button("Sync Now")
            .on_press(GuiMessage::SendLog(String::from("Pressed Sync Now in UI")))
            .padding(6)
    ]
    .spacing(15)
    .align_y(Alignment::Center)
    .padding(10)
    .into()
}

pub fn draw_list_ui<'a, T, F>(
    title: &'a str,
    items: &'a [T],
    item_drawer: F,
) -> Element<'a, GuiMessage>
where
    F: Fn(&'a T) -> Element<'a, GuiMessage>,
{
    let mut list_column = column![text(title).size(28),].spacing(15);

    // Iterate over whatever slice of items is passed in
    for item in items {
        list_column = list_column.push(item_drawer(item));
    }

    // Wrapping it in scrollable is usually helpful for lists!
    scrollable(list_column).into()
}
