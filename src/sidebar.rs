use chrono::{DateTime, Local};
use iced::{
    widget::{button, column, container, row, scrollable, text, Container},
    Alignment, Background, Element, Length, Theme,
};
use ulid::Ulid;

use crate::{
    helper::{button_icon, button_icon_text},
    history::SavedChat,
    Message,
};

pub struct Sidebar {
    pub chats: Vec<SavedChat<String>>,
}

impl Sidebar {
    pub fn new(chats: Vec<SavedChat<String>>) -> Self {
        Self { chats }
    }

    pub fn add_chat(&mut self, chat: SavedChat<String>) {
        self.chats.push(chat);
        self.chats.sort_by(|a, b| a.ulid.cmp(&b.ulid))
    }

    pub fn remove_chat(&mut self, chat_id: Ulid) -> bool {
        if let Some(idx) = self.chats.iter().position(|c| c.ulid == chat_id) {
            self.chats.remove(idx);
            true
        } else {
            false
        }
    }

    fn view_element<'a>(chat: &'a SavedChat<String>) -> Element<'a, Message> {
        let datetime = chat.ulid.datetime();
        let date: DateTime<Local> = datetime.into();

        button(
            row![]
                .push(
                    column![]
                        .push(text(format!("{}", date.format("%Y-%m-%d %H:%M:%S"))))
                        .push(text(format!("{}", chat.description())).size(12.0))
                        .spacing(5.0)
                        .width(Length::Fill),
                )
                .push(
                    button_icon(iced_fonts::Bootstrap::Trash)
                        .on_press(Message::HistoryDelete(chat.ulid.clone()))
                        .padding(1.0),
                )
                .spacing(5.0)
                .align_y(Alignment::Center),
        )
        .on_press(Message::HistorySelected(chat.ulid.clone()))
        .style(|theme, status| iced::widget::button::text(theme, status))
        .into()
    }

    pub fn view<'a>(&'a self) -> Container<'a, Message> {
        let elements = self.chats.iter().map(Self::view_element);
        container(
            column![]
                .push(
                    button_icon_text(
                        iced_fonts::Bootstrap::ArrowsCollapseVertical,
                        "Close Sidebar",
                    )
                    .on_press(Message::SidebarVisibilityToggle)
                    .width(Length::Fill),
                )
                .push(scrollable(column(elements))),
        )
        .style(|theme: &Theme| {
            let base = theme.extended_palette().background.base.color;
            let color = crate::utils::deviate(base, 0.1);
            container::background(Background::Color(color))
        })
        .height(Length::Fill)
    }

    pub fn view_collapse<'a>(&'a self) -> Container<'a, Message> {
        container(
            button_icon(iced_fonts::Bootstrap::ArrowsExpandVertical)
                .on_press(Message::SidebarVisibilityToggle)
                .width(Length::Fill),
        )
        .height(Length::Fill)
    }
}
