use iced::Element;
use iced::widget::{column, container, scrollable, text};

use crate::app::Message;
use crate::theme::Colors;

pub fn view(entries: &[crate::app::GuardLogEntry]) -> Element<'_, Message> {
    if entries.is_empty() {
        return container(
            text("No guard log entries yet.")
                .size(13)
                .color(Colors::TEXT_SECONDARY),
        )
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into();
    }

    let rows: Vec<Element<'_, Message>> = entries
        .iter()
        .map(|entry| {
            container(
                column![
                    text(format!(
                        "{} · {} · {}",
                        entry.timestamp, entry.package, entry.action
                    ))
                    .size(13)
                    .color(Colors::TEXT_PRIMARY),
                    text(format!(
                        "Risk: {} · Script: {}",
                        entry.risk_level, entry.script
                    ))
                    .size(12)
                    .color(Colors::TEXT_SECONDARY),
                ]
                .spacing(4),
            )
            .padding(12)
            .into()
        })
        .collect();

    scrollable(column(rows).spacing(1)).into()
}
