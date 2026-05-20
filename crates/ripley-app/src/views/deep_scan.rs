use iced::Element;
use iced::widget::{column, container, text};

use crate::app::{DeepScanEntry, Message};
use crate::theme::{Colors, severity_color};

pub fn view(report: &Option<DeepScanEntry>) -> Element<'_, Message> {
    let report = match report {
        Some(r) => r,
        None => {
            return container(
                column![
                    text("Deep Scan").size(15).color(Colors::TEXT_PRIMARY),
                    text("No deep scan results yet. Run `ripley scan --deep` to perform a forensic audit.")
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
    };

    let mut sections: Vec<Element<'_, Message>> = Vec::new();

    sections.push(
        text("Deep Scan Report")
            .size(15)
            .color(Colors::TEXT_PRIMARY)
            .into(),
    );

    let total = report.ioc_count + report.persistence_count + report.credential_count;
    let summary_text = if total == 0 {
        "No forensic findings.".to_string()
    } else {
        let s = if total == 1 { "" } else { "s" };
        let ioc_s = if report.ioc_count == 1 { "" } else { "s" };
        let cred_s = if report.credential_count == 1 {
            ""
        } else {
            "s"
        };
        format!(
            "{total} finding{s}: {} IOC{ioc_s}, {} persistence, {} credential{cred_s}",
            report.ioc_count, report.persistence_count, report.credential_count,
        )
    };

    let summary_color = if total == 0 {
        Colors::SEVERITY_CLEAN
    } else {
        Colors::SEVERITY_CRITICAL
    };

    sections.push(text(summary_text).size(13).color(summary_color).into());

    for finding in &report.findings {
        let color = severity_color(&finding.severity);
        sections.push(
            container(
                column![
                    text(format!("[{}] {}", finding.severity, finding.category))
                        .size(12)
                        .color(color),
                    text(&finding.description)
                        .size(12)
                        .color(Colors::TEXT_PRIMARY),
                    text(finding.path.display().to_string())
                        .size(11)
                        .color(Colors::TEXT_MUTED),
                ]
                .spacing(2),
            )
            .padding(8)
            .into(),
        );
    }

    if let Some(ref warning) = report.dead_man_switch_warning {
        sections.push(
            container(
                text(format!("DEAD MAN SWITCH: {warning}"))
                    .size(13)
                    .color(Colors::SEVERITY_CRITICAL),
            )
            .padding(12)
            .into(),
        );
    }

    column(sections).spacing(8).into()
}
