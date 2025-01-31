use iced::widget::{button, row, text};

pub fn button_icon_text<'a, M: 'a>(
    icon: iced_fonts::Bootstrap,
    name: &'static str,
) -> iced::widget::Button<'a, M> {
    button(row![icon_to_text(icon), name].spacing(10))
}

pub fn button_icon<'a, M: 'a>(icon: iced_fonts::Bootstrap) -> iced::widget::Button<'a, M> {
    button(row![icon_to_text(icon)])
}

pub fn button_icon_small<'a, M: 'a>(icon: iced_fonts::Bootstrap) -> iced::widget::Button<'a, M> {
    button(icon_to_text(icon).size(10.0))
}

pub fn icon_to_text<'a>(icon: iced_fonts::Bootstrap) -> iced::widget::Text<'a> {
    text(iced_fonts::bootstrap::icon_to_char(icon)).font(iced_fonts::BOOTSTRAP_FONT)
}
