use std::path::PathBuf;

use ripley_core::dirs;
use ripley_core::forensic::credentials::{self, CredentialFinding, ExposureReport};
use ripley_core::forensic::ioc::{self, IocFinding, IocProfileSet};
use ripley_core::forensic::persistence::{self, PersistenceFinding};
use ripley_core::platform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ForensicFindingDto {
    pub kind: String,
    pub path: String,
    pub description: String,
    pub severity: String,
    pub profile_id: Option<String>,
    pub rotation_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DeepScanReportDto {
    pub project_path: String,
    pub ioc_count: u32,
    pub persistence_count: u32,
    pub credential_count: u32,
    pub findings: Vec<ForensicFindingDto>,
    pub dead_man_switch_warning: Option<String>,
}

fn ioc_to_dto(f: &IocFinding) -> ForensicFindingDto {
    ForensicFindingDto {
        kind: "ioc".to_string(),
        path: f.path.to_string_lossy().into_owned(),
        description: f.description.clone(),
        severity: f.severity.to_string(),
        profile_id: Some(f.profile_id.clone()),
        rotation_command: None,
    }
}

fn persistence_to_dto(f: &PersistenceFinding) -> ForensicFindingDto {
    ForensicFindingDto {
        kind: "persistence".to_string(),
        path: f.path.to_string_lossy().into_owned(),
        description: f.description.clone(),
        severity: f.severity.to_string(),
        profile_id: None,
        rotation_command: None,
    }
}

fn credential_to_dto(f: &CredentialFinding) -> ForensicFindingDto {
    ForensicFindingDto {
        kind: "credential".to_string(),
        path: f.path.to_string_lossy().into_owned(),
        description: f.description.clone(),
        severity: f.severity.to_string(),
        profile_id: Some(f.profile_id.clone()),
        rotation_command: f.rotation_command.clone(),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn run_deep_scan(project_path: String) -> Result<DeepScanReportDto, String> {
    let path = PathBuf::from(&project_path);
    if !path.exists() {
        return Err(format!("project path does not exist: {project_path}"));
    }

    let project_for_thread = path.clone();
    let report = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let config_dir = dirs::config_dir().map_err(|e| e.to_string())?;
        let iocs_dir = config_dir.join("iocs");

        let compiled = IocProfileSet::load_compiled().map_err(|e| e.to_string())?;
        let user = IocProfileSet::load_user_profiles(&iocs_dir).map_err(|e| e.to_string())?;
        let merged = IocProfileSet::merge(compiled, user);
        let profiles = merged.profiles();

        let ioc_findings = ioc::scan_iocs(&project_for_thread, profiles);
        let home = platform::home_dir().unwrap_or_else(|| project_for_thread.clone());
        let persistence_findings = persistence::audit_persistence(&home);
        let exposure: ExposureReport = credentials::assess_exposure(profiles, &home);

        Ok((ioc_findings, persistence_findings, exposure))
    })
    .await
    .map_err(|e| e.to_string())??;

    let (ioc_findings, persistence_findings, exposure) = report;

    let mut findings = Vec::new();
    findings.extend(ioc_findings.iter().map(ioc_to_dto));
    findings.extend(persistence_findings.iter().map(persistence_to_dto));
    findings.extend(exposure.findings.iter().map(credential_to_dto));

    Ok(DeepScanReportDto {
        project_path,
        ioc_count: ioc_findings.len() as u32,
        persistence_count: persistence_findings.len() as u32,
        credential_count: exposure.findings.len() as u32,
        findings,
        dead_man_switch_warning: exposure.dead_man_switch_warning,
    })
}
