use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use ripley_core::audit::{AuditReport, TrafficLight};

use crate::app::Message;
use crate::theme::Colors;

pub fn view(report: &Option<AuditReport>) -> Element<'_, Message> {
    let mut content = column![
        text("Environment Audit")
            .size(18)
            .color(Colors::TEXT_PRIMARY)
    ]
    .spacing(12)
    .width(Length::Fill);

    match report {
        None => {
            content = content.push(
                text("Run `ripley audit` to generate an environment security report.")
                    .color(Colors::TEXT_SECONDARY),
            );
        }
        Some(report) => {
            for cat in &report.categories {
                let indicator = match cat.overall {
                    TrafficLight::Green => "●",
                    TrafficLight::Yellow => "◐",
                    TrafficLight::Red => "○",
                };
                let color = match cat.overall {
                    TrafficLight::Green => Colors::STATUS_GREEN,
                    TrafficLight::Yellow => Colors::STATUS_YELLOW,
                    TrafficLight::Red => Colors::STATUS_RED,
                };

                let header = row![
                    text(indicator).size(14).color(color),
                    text(format!(" {} ", cat.category))
                        .size(14)
                        .color(Colors::TEXT_PRIMARY),
                    text(format!("[{}]", cat.overall)).size(12).color(color),
                ]
                .spacing(4);

                content = content.push(header);

                for finding in &cat.findings {
                    let f_color = match finding.status {
                        TrafficLight::Green => Colors::STATUS_GREEN,
                        TrafficLight::Yellow => Colors::STATUS_YELLOW,
                        TrafficLight::Red => Colors::STATUS_RED,
                    };
                    let finding_text = text(format!("  {} — {}", finding.name, finding.detail))
                        .size(12)
                        .color(f_color);
                    content = content.push(finding_text);

                    if let Some(ref fix) = finding.fix_command {
                        let fix_text = text(format!("    → {fix}"))
                            .size(11)
                            .color(Colors::TEXT_SECONDARY);
                        content = content.push(fix_text);
                    }
                }
            }

            let summary = format!(
                "Summary: {} green, {} yellow, {} red",
                report
                    .categories
                    .iter()
                    .filter(|c| c.overall == TrafficLight::Green)
                    .count(),
                report
                    .categories
                    .iter()
                    .filter(|c| c.overall == TrafficLight::Yellow)
                    .count(),
                report
                    .categories
                    .iter()
                    .filter(|c| c.overall == TrafficLight::Red)
                    .count(),
            );
            content = content.push(text(summary).size(12).color(Colors::TEXT_SECONDARY));
        }
    }

    container(content).padding(16).width(Length::Fill).into()
}
