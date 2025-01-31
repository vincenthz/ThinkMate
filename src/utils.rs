// simple iced utility
use iced::Color;
use palette::{rgb::Rgb, FromColor, Hsl};

pub(crate) fn darken(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lightness = if hsl.lightness - amount < 0.0 {
        0.0
    } else {
        hsl.lightness - amount
    };

    from_hsl(hsl)
}

pub(crate) fn lighten(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lightness = if hsl.lightness + amount > 1.0 {
        1.0
    } else {
        hsl.lightness + amount
    };

    from_hsl(hsl)
}

pub(crate) fn deviate(color: Color, amount: f32) -> Color {
    if is_dark(color) {
        lighten(color, amount)
    } else {
        darken(color, amount)
    }
}

pub(crate) fn is_dark(color: Color) -> bool {
    to_hsl(color).lightness < 0.6
}

fn to_hsl(color: Color) -> Hsl {
    Hsl::from_color(Rgb::from(color))
}

fn from_hsl(hsl: Hsl) -> Color {
    Rgb::from_color(hsl).into()
}
