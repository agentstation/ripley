use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};

use ripley_core::audit::TrafficLight;
use ripley_core::harden::HardenReport;

use crate::app::Message;
use crate::theme::Colors;

pub fn view(report: &Option<HardenReport>) -> Element<'_, Message> {
    let mut content = column![
        text("Posture & Hardening")
            .size(18)
            .color(Colors::TEXT_PRIMARY)
    ]
    .spacing(12)
    .width(Length::Fill);

    match report {
        None => {
            content = content.push(
                text("Run `ripley harden` to generate a hardening report.")
                    .color(Colors::TEXT_SECONDARY),
            );
        }
        Some(report) => {
            if report.detected_pms.is_empty() {
                content = content
                    .push(text("No package managers detected.").color(Colors::TEXT_SECONDARY));
            } else {
                let pms_text = report
                    .detected_pms
                    .iter()
                    .map(|pm| {
                        let ver = pm.version.as_deref().unwrap_or("?");
                        format!("{} v{ver}", pm.name)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                content = content.push(
                    text(format!("Detected: {pms_text}"))
                        .size(13)
                        .color(Colors::TEXT_SECONDARY),
                );

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

                        let pm_tag = if finding.pm.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", finding.pm)
                        };

                        let finding_text =
                            text(format!("  {}{pm_tag} — {}", finding.name, finding.detail))
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
    }

    scrollable(container(content).padding(16).width(Length::Fill)).into()
}
