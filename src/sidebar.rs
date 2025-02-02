use iced::{
    widget::{button, column, container, scrollable, text, Container},
    Background, Length, Theme,
};

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

    pub fn view<'a>(&'a self) -> Container<'a, Message> {
        let elements = self.chats.iter().map(|c| {
            button(text(format!("{}", &c.ulid)))
                .on_press(Message::HistorySelected(c.ulid.clone()))
                .into()
        });
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
