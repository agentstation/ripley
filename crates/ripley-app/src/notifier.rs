use ripley_core::matcher::Match;
use ripley_core::monitor::process::ProcessAlert;
use ripley_core::types::Severity;

#[allow(dead_code)]
pub fn notify_match(m: &Match) {
    let severity = m.advisory.severity.unwrap_or(Severity::Low);
    let severity_label = severity_to_label(severity);

    let summary = format!("{}@{} is vulnerable", m.advisory.package, m.package.version);
    let body = format!(
        "{} · {} · {}\nAffects: {}",
        m.advisory.id,
        m.advisory.summary,
        severity_label,
        m.project_path.display()
    );

    send_notification(&summary, &body, severity);
}

pub fn notify_monitor_alert(alert: &ProcessAlert) {
    let label = severity_to_label(alert.severity);
    let summary = format!("{label}: {}", alert.reason);
    let body = match &alert.connection {
        Some(c) => format!("{} (PID {})", c.process, c.pid),
        None => format!("{}", alert.reason),
    };

    send_notification(&summary, &body, alert.severity);
}

fn severity_to_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "CRITICAL",
        Severity::High => "HIGH",
        Severity::Medium => "MEDIUM",
        Severity::Low => "LOW",
    }
}

fn send_notification(summary: &str, body: &str, _severity: Severity) {
    if let Err(e) = notify_rust::Notification::new()
        .appname("Ripley")
        .summary(summary)
        .body(body)
        .show()
    {
        tracing::warn!("failed to send notification: {e}");
    }
}
