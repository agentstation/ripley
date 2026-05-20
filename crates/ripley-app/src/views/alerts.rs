use iced::Element;
use iced::widget::{column, container, text};

use crate::app::Message;
use crate::theme::Colors;

pub fn view(alerts: &[crate::app::AlertEntry]) -> Element<'_, Message> {
    if alerts.is_empty() {
        return container(
            column![
                text("No findings").size(15).color(Colors::TEXT_PRIMARY),
                text("Your dependencies look clean.")
                    .size(13)
                    .color(Colors::TEXT_SECONDARY),
            ]
            .spacing(8)
            .align_x(iced::Alignment::Center),
        )
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into();
    }

    let rows: Vec<Element<'_, Message>> = alerts
        .iter()
        .map(|alert| {
            container(
                column![
                    text(format!(
                        "[{}] {}@{}",
                        alert.severity, alert.package, alert.version
                    ))
                    .size(13)
                    .color(Colors::TEXT_PRIMARY),
                    text(&alert.summary).size(12).color(Colors::TEXT_SECONDARY),
                ]
                .spacing(4),
            )
            .padding(12)
            .into()
        })
        .collect();

    column(rows).spacing(1).into()
}
