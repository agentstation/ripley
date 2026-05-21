use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Theme};

use crate::app::Message;
use crate::theme::Colors;

#[allow(dead_code)]
pub struct GuardDialogData {
    pub package: String,
    pub version: String,
    pub hook_name: String,
    pub risk_level: String,
    pub script_content: String,
    pub matched_rules: Vec<MatchedRuleDisplay>,
    pub countdown_secs: u32,
}

#[allow(dead_code)]
pub struct MatchedRuleDisplay {
    pub severity: String,
    pub name: String,
    pub description: String,
}

#[allow(dead_code)]
pub fn view(data: &GuardDialogData) -> Element<'_, Message> {
    let header = row![
        text(format!("● {} RISK", data.risk_level.to_uppercase()))
            .size(13)
            .color(Colors::SEVERITY_CRITICAL),
        text(format!("⏱ {}s", data.countdown_secs))
            .size(13)
            .color(if data.countdown_secs < 10 {
                Colors::TEXT_PRIMARY
            } else {
                Colors::TEXT_SECONDARY
            }),
    ]
    .spacing(12);

    let package_label = text(format!(
        "{}@{} · {}",
        data.package, data.version, data.hook_name
    ))
    .size(15)
    .color(Colors::TEXT_PRIMARY);

    let script_view = container(
        text(&data.script_content)
            .size(12)
            .color(Colors::TEXT_PRIMARY),
    )
    .padding(12)
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(Colors::BG)),
        border: iced::Border {
            color: Colors::BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    let rules_section = if data.matched_rules.is_empty() {
        column![]
    } else {
        let mut col = column![
            text("Matched rules:")
                .size(12)
                .color(Colors::TEXT_SECONDARY)
        ]
        .spacing(4);
        for rule in &data.matched_rules {
            col = col.push(
                text(format!(
                    "[{}] {} — {}",
                    rule.severity, rule.name, rule.description
                ))
                .size(12)
                .color(Colors::TEXT_SECONDARY),
            );
        }
        col
    };

    let buttons = row![
        button(text("Allow Once").size(13))
            .on_press(Message::Noop)
            .padding([8, 16]),
        button(text(format!("Block (default in {}s)", data.countdown_secs)).size(13))
            .on_press(Message::Noop)
            .padding([8, 16]),
        button(text("Always Trust").size(13))
            .on_press(Message::Noop)
            .padding([8, 16]),
    ]
    .spacing(8);

    container(
        column![header, package_label, script_view, rules_section, buttons,]
            .spacing(12)
            .padding(16)
            .max_width(600),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(Colors::SURFACE)),
        border: iced::Border {
            color: Colors::BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}
