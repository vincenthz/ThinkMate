use iced::{
    widget::{container, Container},
    Background, Length, Theme,
};

use crate::{
    helper::{button_icon, button_icon_text},
    Message,
};

pub struct Sidebar {}

impl Sidebar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view<'a>(&'a self) -> Container<'a, Message> {
        container(
            button_icon_text(
                iced_fonts::Bootstrap::ArrowsCollapseVertical,
                "Close Sidebar",
            )
            .on_press(Message::SidebarVisibilityToggle)
            .width(Length::Fill),
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
