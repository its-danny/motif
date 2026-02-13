use iced::widget::{container, row, text};
use iced::{Background, Border, Element, Fill, Font, Theme};

use crate::app::{Message, Mode};
use crate::theme;

pub fn view(mode: &Mode) -> Element<'_, Message> {
    let mode_badge = container(
        text(mode.label())
            .font(Font::MONOSPACE)
            .size(12)
            .color(theme::ZINC_950),
    )
    .padding([2, 8])
    .style(move |_theme: &Theme| container::Style {
        background: Some(Background::Color(mode.color())),
        border: Border {
            radius: 3.0.into(),
            ..Border::default()
        },
        ..Default::default()
    });

    let bpm = text("♩ 120")
        .font(Font::MONOSPACE)
        .size(12)
        .color(theme::ZINC_500);

    let position = text("001 : 1 : 000")
        .font(Font::MONOSPACE)
        .size(12)
        .color(theme::ZINC_400);

    let bar = row![mode_badge, bpm, position]
        .spacing(12)
        .align_y(iced::Alignment::Center);

    container(bar)
        .width(Fill)
        .padding([6, 12])
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(theme::ZINC_900)),
            border: Border {
                color: theme::ZINC_800,
                width: 1.0,
                ..Border::default()
            },
            ..Default::default()
        })
        .into()
}
