use iced::Element;
use iced::widget::{column, container, text};

use crate::app::Message;
use crate::theme::Colors;

pub fn view(config: &ripley_core::config::Config) -> Element<'_, Message> {
    container(
        column![
            text("Settings").size(15).color(Colors::TEXT_PRIMARY),
            text(format!(
                "Poll interval: {}s",
                config.general.poll_interval_secs
            ))
            .size(13)
            .color(Colors::TEXT_SECONDARY),
            text(format!("Guard mode: {:?}", config.guard.mode))
                .size(13)
                .color(Colors::TEXT_SECONDARY),
            text(format!("Trusted packages: {}", config.guard.trust.len()))
                .size(13)
                .color(Colors::TEXT_SECONDARY),
            text(format!(
                "Project roots: {}",
                config.monitoring.project_roots.len()
            ))
            .size(13)
            .color(Colors::TEXT_SECONDARY),
        ]
        .spacing(8),
    )
    .padding(16)
    .into()
}
