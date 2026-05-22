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
            text("Monitor").size(14).color(Colors::TEXT_PRIMARY),
            text(format!(
                "Enabled: {}",
                if config.monitor.enabled { "yes" } else { "no" }
            ))
            .size(13)
            .color(Colors::TEXT_SECONDARY),
            text(format!(
                "Watch processes: {}",
                if config.monitor.watch_processes {
                    "yes"
                } else {
                    "no"
                }
            ))
            .size(13)
            .color(Colors::TEXT_SECONDARY),
            text(format!(
                "Watch persistence: {}",
                if config.monitor.watch_persistence {
                    "yes"
                } else {
                    "no"
                }
            ))
            .size(13)
            .color(Colors::TEXT_SECONDARY),
            text(format!(
                "Watch lockfiles: {}",
                if config.monitor.watch_lockfiles {
                    "yes"
                } else {
                    "no"
                }
            ))
            .size(13)
            .color(Colors::TEXT_SECONDARY),
        ]
        .spacing(8),
    )
    .padding(16)
    .into()
}
