use std::collections::BTreeSet;
use std::path::PathBuf;

use iced::widget::{Rule, button, column, container, row, scrollable, text};
use iced::{Element, Length, Task as IcedTask, Theme};

use crate::theme::Colors;
use crate::views;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AlertEntry {
    pub advisory_id: String,
    pub package: String,
    pub version: String,
    pub severity: String,
    pub summary: String,
    pub project_path: String,
}

#[derive(Debug, Clone)]
pub struct GuardLogEntry {
    pub timestamp: String,
    pub package: String,
    pub script: String,
    pub risk_level: String,
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct DeepScanEntry {
    pub vuln_count: usize,
    pub ioc_findings: Vec<DeepScanFinding>,
    pub persistence_findings: Vec<DeepScanFinding>,
    pub credential_findings: Vec<CredentialEntry>,
    pub mcp_findings: Vec<DeepScanFinding>,
    pub dead_man_switch_warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeepScanFinding {
    pub path: PathBuf,
    pub description: String,
    pub severity: ripley_core::types::Severity,
    #[allow(dead_code)]
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct CredentialEntry {
    pub path: PathBuf,
    pub description: String,
    pub severity: ripley_core::types::Severity,
    pub rotation_command: Option<String>,
}

#[derive(Debug, Clone)]
pub enum View {
    Alerts,
    Guard,
    DeepScan,
    Audit,
    Posture,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(View),
    #[allow(dead_code)]
    ScanNow,
    ToggleSection(String),
    RunDeepScan,
    ExportReport,
    CopyRotationChecklist,
    Noop,
}

pub struct RipleyApp {
    current_view: View,
    alerts: Vec<AlertEntry>,
    guard_log: Vec<GuardLogEntry>,
    deep_scan: Option<DeepScanEntry>,
    audit_report: Option<ripley_core::audit::AuditReport>,
    harden_report: Option<ripley_core::harden::HardenReport>,
    expanded_sections: BTreeSet<String>,
    config: ripley_core::config::Config,
}

impl RipleyApp {
    fn new() -> (Self, IcedTask<Message>) {
        let config = ripley_core::config::Config::default();
        (
            Self {
                current_view: View::Alerts,
                alerts: Vec::new(),
                guard_log: Vec::new(),
                deep_scan: None,
                audit_report: None,
                harden_report: None,
                expanded_sections: BTreeSet::new(),
                config,
            },
            IcedTask::none(),
        )
    }

    fn title(&self) -> String {
        "Ripley".into()
    }

    fn update(&mut self, message: Message) -> IcedTask<Message> {
        match message {
            Message::NavigateTo(view) => {
                self.current_view = view;
            }
            Message::ToggleSection(section) => {
                if !self.expanded_sections.remove(&section) {
                    self.expanded_sections.insert(section);
                }
            }
            Message::ScanNow
            | Message::RunDeepScan
            | Message::ExportReport
            | Message::CopyRotationChecklist
            | Message::Noop => {}
        }
        IcedTask::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.sidebar();
        let content = match self.current_view {
            View::Alerts => views::alerts::view(&self.alerts),
            View::Guard => views::guard_log::view(&self.guard_log),
            View::DeepScan => views::deep_scan::view(&self.deep_scan, &self.expanded_sections),
            View::Audit => views::audit::view(&self.audit_report),
            View::Posture => views::posture::view(&self.harden_report),
            View::Settings => views::settings::view(&self.config),
        };

        let main_content = container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(Colors::BG)),
                ..Default::default()
            })
            .padding(16);

        row![sidebar, main_content].into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let is_alerts = matches!(self.current_view, View::Alerts);
        let is_guard = matches!(self.current_view, View::Guard);
        let is_deep_scan = matches!(self.current_view, View::DeepScan);
        let is_audit = matches!(self.current_view, View::Audit);
        let is_posture = matches!(self.current_view, View::Posture);
        let is_settings = matches!(self.current_view, View::Settings);

        container(
            column![
                text("Ripley").size(15).color(Colors::TEXT_PRIMARY),
                Rule::horizontal(1),
                nav_button("Alerts", Message::NavigateTo(View::Alerts), is_alerts),
                nav_button("Guard", Message::NavigateTo(View::Guard), is_guard),
                nav_button(
                    "Deep Scan",
                    Message::NavigateTo(View::DeepScan),
                    is_deep_scan
                ),
                nav_button("Audit", Message::NavigateTo(View::Audit), is_audit),
                nav_button("Posture", Message::NavigateTo(View::Posture), is_posture),
                nav_button("Settings", Message::NavigateTo(View::Settings), is_settings),
            ]
            .spacing(4)
            .padding(8),
        )
        .width(200)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(Colors::SURFACE)),
            border: iced::Border {
                color: Colors::BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

fn nav_button(label: &str, msg: Message, is_active: bool) -> Element<'_, Message> {
    let color = if is_active {
        Colors::ACCENT
    } else {
        Colors::TEXT_SECONDARY
    };
    button(text(label).size(13).color(color))
        .on_press(msg)
        .padding([8, 16])
        .width(Length::Fill)
        .into()
}

pub fn run() -> anyhow::Result<()> {
    iced::application(RipleyApp::title, RipleyApp::update, RipleyApp::view)
        .window_size((900.0, 640.0))
        .run_with(RipleyApp::new)?;
    Ok(())
}
