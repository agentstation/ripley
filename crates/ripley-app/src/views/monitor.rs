use iced::Element;
use iced::widget::{button, column, container, text};

use crate::app::{Message, MonitorAlertEntry};
use crate::theme::Colors;

pub fn view(enabled: bool, alerts: &[MonitorAlertEntry]) -> Element<'_, Message> {
    if !enabled {
        return container(
            column![
                text("Monitor disabled")
                    .size(15)
                    .color(Colors::TEXT_PRIMARY),
                text("Enable monitoring in Settings to detect active threats.")
                    .size(13)
                    .color(Colors::TEXT_SECONDARY),
                button(text("Enable Monitor").size(13).color(Colors::ACCENT))
                    .on_press(Message::ToggleMonitor)
                    .padding([8, 16]),
            ]
            .spacing(8)
            .align_x(iced::Alignment::Center),
        )
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into();
    }

    if alerts.is_empty() {
        return container(
            column![
                text("Monitoring active")
                    .size(15)
                    .color(Colors::TEXT_PRIMARY),
                text("No suspicious activity detected.")
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
            let severity_color = match alert.severity.as_str() {
                "critical" => Colors::SEVERITY_CRITICAL,
                "high" => Colors::SEVERITY_HIGH,
                "medium" => Colors::SEVERITY_MEDIUM,
                "low" => Colors::SEVERITY_LOW,
                _ => Colors::TEXT_SECONDARY,
            };

            let time = format_timestamp(alert.timestamp);
            let mut col = column![
                text(format!(
                    "[{time}] [{}] {} (PID {})",
                    alert.severity.to_uppercase(),
                    alert.process,
                    alert.pid
                ))
                .size(13)
                .color(severity_color),
                text(&alert.reason).size(12).color(Colors::TEXT_SECONDARY),
            ]
            .spacing(4);

            if !alert.detail.is_empty() {
                col = col.push(text(&alert.detail).size(11).color(Colors::TEXT_MUTED));
            }

            container(col).padding(12).into()
        })
        .collect();

    column(rows).spacing(1).into()
}

fn format_timestamp(ts: u64) -> String {
    let secs = ts % 86400;
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{hours:02}:{mins:02}:{s:02}Z")
}
