use std::collections::BTreeSet;

use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Length, Theme};

use crate::app::{CredentialEntry, DeepScanEntry, DeepScanFinding, Message};
use crate::theme::{Colors, severity_color};

pub fn view<'a>(
    report: &'a Option<DeepScanEntry>,
    expanded: &'a BTreeSet<String>,
) -> Element<'a, Message> {
    let report = match report {
        Some(r) => r,
        None => return empty_state(),
    };

    let mut sections: Vec<Element<'a, Message>> = Vec::new();

    sections.push(
        text("Deep Scan Report")
            .size(15)
            .color(Colors::TEXT_PRIMARY)
            .into(),
    );

    sections.push(summary_cards(report));
    sections.push(Space::with_height(8).into());

    let ioc_label = "IOC Files Found";
    let persist_label = "Persistence";
    let creds_label = "Credentials at Risk";
    let mcp_label = "MCP Config";

    sections.push(collapsible_section(
        ioc_label,
        &report.ioc_findings,
        expanded.contains(ioc_label),
    ));
    sections.push(collapsible_section(
        persist_label,
        &report.persistence_findings,
        expanded.contains(persist_label),
    ));
    sections.push(credential_section(
        creds_label,
        &report.credential_findings,
        expanded.contains(creds_label),
    ));
    sections.push(collapsible_section(
        mcp_label,
        &report.mcp_findings,
        expanded.contains(mcp_label),
    ));

    if let Some(ref warning) = report.dead_man_switch_warning {
        sections.push(dead_man_switch_box(warning));
    }

    sections.push(Space::with_height(8).into());
    sections.push(action_buttons());

    column(sections).spacing(8).into()
}

fn empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            text("Deep Scan").size(15).color(Colors::TEXT_PRIMARY),
            text("No deep scan results yet.")
                .size(13)
                .color(Colors::TEXT_SECONDARY),
            Space::with_height(12),
            button(text("Run Deep Scan").size(13).color(Colors::ACCENT))
                .on_press(Message::RunDeepScan)
                .padding([8, 16]),
        ]
        .spacing(8)
        .align_x(iced::Alignment::Center),
    )
    .center_x(iced::Fill)
    .center_y(iced::Fill)
    .into()
}

fn summary_cards(report: &DeepScanEntry) -> Element<'_, Message> {
    let cards = vec![
        ("Vulns", report.vuln_count),
        ("IOCs", report.ioc_findings.len()),
        ("Persist.", report.persistence_findings.len()),
        ("Creds Risk", report.credential_findings.len()),
        ("MCP", report.mcp_findings.len()),
    ];

    let card_elements: Vec<Element<'_, Message>> = cards
        .into_iter()
        .map(|(label, count)| {
            let color = if count == 0 {
                Colors::SEVERITY_CLEAN
            } else {
                Colors::SEVERITY_CRITICAL
            };

            container(
                column![
                    text(count.to_string()).size(18).color(color),
                    text(label).size(11).color(Colors::TEXT_SECONDARY),
                ]
                .align_x(iced::Alignment::Center)
                .spacing(2),
            )
            .padding([12, 16])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(Colors::SURFACE)),
                border: iced::Border {
                    color: Colors::BORDER_SUBTLE,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .width(Length::Fill)
            .into()
        })
        .collect();

    row(card_elements).spacing(8).into()
}

fn section_header<'a>(label: &'a str, count: usize, is_expanded: bool) -> Element<'a, Message> {
    let arrow = if is_expanded { "v" } else { ">" };
    let count_text = if count == 0 {
        "clean".to_string()
    } else {
        count.to_string()
    };
    let count_color = if count == 0 {
        Colors::SEVERITY_CLEAN
    } else {
        Colors::TEXT_PRIMARY
    };

    button(
        row![
            text(arrow).size(11).color(Colors::TEXT_MUTED),
            text(label).size(13).color(Colors::TEXT_PRIMARY),
            Space::with_width(Length::Fill),
            text(count_text).size(12).color(count_color),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .on_press(Message::ToggleSection(label.to_string()))
    .padding([6, 12])
    .width(Length::Fill)
    .into()
}

fn collapsible_section<'a>(
    label: &'a str,
    findings: &'a [DeepScanFinding],
    is_expanded: bool,
) -> Element<'a, Message> {
    let mut items: Vec<Element<'a, Message>> = Vec::new();
    items.push(section_header(label, findings.len(), is_expanded));

    if is_expanded {
        for finding in findings {
            items.push(finding_row(finding));
        }
        if findings.is_empty() {
            items.push(
                container(
                    text("No findings in this category.")
                        .size(12)
                        .color(Colors::TEXT_MUTED),
                )
                .padding([4, 24])
                .into(),
            );
        }
    }

    container(column(items).spacing(2))
        .style(|_theme: &Theme| container::Style {
            border: iced::Border {
                color: Colors::BORDER_SUBTLE,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding(4)
        .into()
}

fn finding_row(finding: &DeepScanFinding) -> Element<'_, Message> {
    let color = severity_color(&finding.severity);
    let severity_label = format!("{}", finding.severity);

    container(
        row![
            text(severity_label).size(11).color(color),
            column![
                text(&finding.description)
                    .size(12)
                    .color(Colors::TEXT_PRIMARY),
                text(finding.path.display().to_string())
                    .size(11)
                    .color(Colors::TEXT_MUTED),
            ]
            .spacing(1),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Start),
    )
    .padding([4, 24])
    .into()
}

fn credential_section<'a>(
    label: &'a str,
    findings: &'a [CredentialEntry],
    is_expanded: bool,
) -> Element<'a, Message> {
    let mut items: Vec<Element<'a, Message>> = Vec::new();
    items.push(section_header(label, findings.len(), is_expanded));

    if is_expanded {
        for finding in findings {
            items.push(credential_row(finding));
        }
        if findings.is_empty() {
            items.push(
                container(
                    text("No credential exposure found.")
                        .size(12)
                        .color(Colors::TEXT_MUTED),
                )
                .padding([4, 24])
                .into(),
            );
        }
    }

    container(column(items).spacing(2))
        .style(|_theme: &Theme| container::Style {
            border: iced::Border {
                color: Colors::BORDER_SUBTLE,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding(4)
        .into()
}

fn credential_row(finding: &CredentialEntry) -> Element<'_, Message> {
    let color = severity_color(&finding.severity);
    let severity_label = format!("{}", finding.severity);

    let mut col_items: Vec<Element<'_, Message>> = vec![
        text(&finding.description)
            .size(12)
            .color(Colors::TEXT_PRIMARY)
            .into(),
        text(finding.path.display().to_string())
            .size(11)
            .color(Colors::TEXT_MUTED)
            .into(),
    ];

    if let Some(ref cmd) = finding.rotation_command {
        col_items.push(
            text(format!("-> {cmd}"))
                .size(11)
                .color(Colors::ACCENT)
                .into(),
        );
    }

    container(
        row![
            text(severity_label).size(11).color(color),
            column(col_items).spacing(1),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Start),
    )
    .padding([4, 24])
    .into()
}

fn dead_man_switch_box(warning: &str) -> Element<'_, Message> {
    container(
        column![
            text("DEAD MAN SWITCH DETECTED")
                .size(13)
                .color(Colors::SEVERITY_CRITICAL),
            text(warning).size(12).color(Colors::TEXT_PRIMARY),
            Space::with_height(4),
            text("1. Back up your home directory")
                .size(11)
                .color(Colors::TEXT_SECONDARY),
            text("2. Disable network access on this machine")
                .size(11)
                .color(Colors::TEXT_SECONDARY),
            text("3. Rotate credentials from a different device")
                .size(11)
                .color(Colors::TEXT_SECONDARY),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(Colors::SURFACE)),
        border: iced::Border {
            color: Colors::SEVERITY_CRITICAL,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn action_buttons<'a>() -> Element<'a, Message> {
    row![
        button(text("Export Report").size(12).color(Colors::ACCENT))
            .on_press(Message::ExportReport)
            .padding([8, 16]),
        button(
            text("Copy Rotation Checklist")
                .size(12)
                .color(Colors::ACCENT)
        )
        .on_press(Message::CopyRotationChecklist)
        .padding([8, 16]),
    ]
    .spacing(8)
    .into()
}
