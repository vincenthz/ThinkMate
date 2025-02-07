use iced::{
    widget::{button, column, container, horizontal_rule, row, text, Container},
    Alignment, Element, Length, Padding,
};

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

pub fn dialog<'a, M: 'a + Clone>(
    title: &'a str,
    inner: impl Into<Element<'a, M>>,
    on_close: M,
) -> Container<'a, M> {
    let action = row!(
        text(title)
            .size(30)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        button_icon(iced_fonts::Bootstrap::X)
            .style(button::danger)
            .height(38)
            .on_press(on_close)
    )
    .align_y(Alignment::Center)
    .width(Length::Fill);
    let dialog_content = column!(
        action,
        horizontal_rule(1),
        container(inner.into())
            .padding(20)
            .center_y(Length::Fill)
            .center_x(Length::Fill)
    );
    let inner = container(dialog_content).style(|t| container::bordered_box(t));
    container(inner).padding(Padding::from([40, 60]))
}
