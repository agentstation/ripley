use std::path::PathBuf;

use ripley_core::harden::{self, DetectedPm, HardenCategoryReport, HardenFinding, HardenReport};
use ripley_core::platform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DetectedPmDto {
    pub name: String,
    pub version: Option<String>,
    pub lockfile_path: String,
    pub binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct HardenFindingDto {
    pub name: String,
    pub status: String,
    pub detail: String,
    pub fix_command: Option<String>,
    pub pm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct HardenCategoryReportDto {
    pub category: String,
    pub findings: Vec<HardenFindingDto>,
    pub overall: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct HardenReportDto {
    pub detected_pms: Vec<DetectedPmDto>,
    pub categories: Vec<HardenCategoryReportDto>,
    pub timestamp: String,
}

impl From<&DetectedPm> for DetectedPmDto {
    fn from(p: &DetectedPm) -> Self {
        Self {
            name: p.name.clone(),
            version: p.version.clone(),
            lockfile_path: p.lockfile_path.to_string_lossy().into_owned(),
            binary: p.binary,
        }
    }
}

impl From<&HardenFinding> for HardenFindingDto {
    fn from(f: &HardenFinding) -> Self {
        Self {
            name: f.name.clone(),
            status: f.status.to_string(),
            detail: f.detail.clone(),
            fix_command: f.fix_command.clone(),
            pm: f.pm.clone(),
        }
    }
}

impl From<&HardenCategoryReport> for HardenCategoryReportDto {
    fn from(c: &HardenCategoryReport) -> Self {
        Self {
            category: c.category.to_string(),
            findings: c.findings.iter().map(Into::into).collect(),
            overall: c.overall.to_string(),
        }
    }
}

impl From<&HardenReport> for HardenReportDto {
    fn from(r: &HardenReport) -> Self {
        Self {
            detected_pms: r.detected_pms.iter().map(Into::into).collect(),
            categories: r.categories.iter().map(Into::into).collect(),
            timestamp: r.timestamp.clone(),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn run_harden_report(project_path: String) -> Result<HardenReportDto, String> {
    let path = PathBuf::from(&project_path);
    if !path.exists() {
        return Err(format!("project path does not exist: {project_path}"));
    }
    let home =
        platform::home_dir().ok_or_else(|| "could not determine home directory".to_string())?;
    let report = tokio::task::spawn_blocking(move || harden::run_harden(&path, &home))
        .await
        .map_err(|e| e.to_string())?;
    Ok(HardenReportDto::from(&report))
}
