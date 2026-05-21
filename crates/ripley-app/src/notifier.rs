use ripley_core::matcher::Match;
use ripley_core::types::Severity;

#[allow(dead_code)]
pub fn notify_match(m: &Match) {
    let severity = m.advisory.severity.unwrap_or(Severity::Low);
    let severity_label = match severity {
        Severity::Critical => "CRITICAL",
        Severity::High => "HIGH",
        Severity::Medium => "MEDIUM",
        Severity::Low => "LOW",
    };

    let summary = format!("{}@{} is vulnerable", m.advisory.package, m.package.version);
    let body = format!(
        "{} · {} · {}\nAffects: {}",
        m.advisory.id,
        m.advisory.summary,
        severity_label,
        m.project_path.display()
    );

    if let Err(e) = notify_rust::Notification::new()
        .appname("Ripley")
        .summary(&summary)
        .body(&body)
        .show()
    {
        tracing::warn!("failed to send notification: {e}");
    }
}
