use iced::Element;
use iced::widget::{column, container, text};

use crate::app::Message;
use crate::theme::Colors;

pub fn network_failure_banner(cache_age: &str) -> Element<'_, Message> {
    container(
        text(format!(
            "⚠ Advisory feeds unreachable. Using cached data ({cache_age} old). Retrying..."
        ))
        .size(12)
        .color(Colors::SEVERITY_MEDIUM),
    )
    .padding([8, 16])
    .width(iced::Fill)
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba8(
            0xd2, 0x99, 0x22, 0.1,
        ))),
        ..Default::default()
    })
    .into()
}

pub fn no_lockfiles_found(path: &str) -> Element<'_, Message> {
    container(
        column![
            text(format!("No lockfiles found in {path}"))
                .size(15)
                .color(Colors::TEXT_PRIMARY),
            text("Ripley looks for:")
                .size(13)
                .color(Colors::TEXT_SECONDARY),
            text("  • package-lock.json (npm)")
                .size(12)
                .color(Colors::TEXT_MUTED),
            text("  • yarn.lock (Yarn)")
                .size(12)
                .color(Colors::TEXT_MUTED),
            text("  • pnpm-lock.yaml (pnpm)")
                .size(12)
                .color(Colors::TEXT_MUTED),
            text("  • Cargo.lock (Rust)")
                .size(12)
                .color(Colors::TEXT_MUTED),
            text("  • go.sum (Go)").size(12).color(Colors::TEXT_MUTED),
            text("  • Gemfile.lock (Ruby)")
                .size(12)
                .color(Colors::TEXT_MUTED),
            text("Make sure you're scanning a directory with dependencies installed.")
                .size(13)
                .color(Colors::TEXT_SECONDARY),
        ]
        .spacing(4),
    )
    .padding(24)
    .center_x(iced::Fill)
    .center_y(iced::Fill)
    .into()
}
