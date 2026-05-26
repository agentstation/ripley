use ripley_core::audit::{
    self, AuditCategory, AuditFinding, AuditReport, CategoryReport, TrafficLight,
};
use ripley_core::platform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AuditFindingDto {
    pub name: String,
    pub status: String,
    pub detail: String,
    pub fix_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CategoryReportDto {
    pub category: String,
    pub findings: Vec<AuditFindingDto>,
    pub overall: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AuditReportDto {
    pub categories: Vec<CategoryReportDto>,
    pub timestamp: String,
}

fn traffic_to_string(t: &TrafficLight) -> String {
    t.to_string()
}

impl From<&AuditFinding> for AuditFindingDto {
    fn from(f: &AuditFinding) -> Self {
        Self {
            name: f.name.clone(),
            status: traffic_to_string(&f.status),
            detail: f.detail.clone(),
            fix_command: f.fix_command.clone(),
        }
    }
}

fn category_label(c: &AuditCategory) -> String {
    c.to_string()
}

impl From<&CategoryReport> for CategoryReportDto {
    fn from(c: &CategoryReport) -> Self {
        Self {
            category: category_label(&c.category),
            findings: c.findings.iter().map(Into::into).collect(),
            overall: traffic_to_string(&c.overall),
        }
    }
}

impl From<&AuditReport> for AuditReportDto {
    fn from(r: &AuditReport) -> Self {
        Self {
            categories: r.categories.iter().map(Into::into).collect(),
            timestamp: r.timestamp.clone(),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn run_audit_report() -> Result<AuditReportDto, String> {
    let home =
        platform::home_dir().ok_or_else(|| "could not determine home directory".to_string())?;
    let report = tokio::task::spawn_blocking(move || audit::run_audit(&home))
        .await
        .map_err(|e| e.to_string())?;
    Ok(AuditReportDto::from(&report))
}
