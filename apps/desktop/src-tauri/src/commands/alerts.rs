use std::path::PathBuf;

use ripley_core::db::AdvisoryDb;
use ripley_core::dirs;
use ripley_core::lockfile;
use ripley_core::matcher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AlertSummary {
    pub id: String,
    pub advisory_id: String,
    pub ecosystem: String,
    pub package: String,
    pub version: String,
    pub severity: String,
    pub summary: String,
    pub references: Vec<String>,
    pub project_path: String,
}

#[tauri::command]
#[specta::specta]
pub fn list_alerts(project_path: String) -> Result<Vec<AlertSummary>, String> {
    let path = PathBuf::from(&project_path);
    if !path.exists() {
        return Err(format!("project path does not exist: {project_path}"));
    }

    let data_dir = dirs::data_dir().map_err(|e| e.to_string())?;
    let db_path = data_dir.join("advisories.redb");
    let db = AdvisoryDb::open(&db_path).map_err(|e| e.to_string())?;

    let lockfiles = lockfile::find_lockfiles(&path);
    let mut packages = Vec::new();
    for lockfile_path in &lockfiles {
        if let Ok(parsed) = lockfile::parse_lockfile(lockfile_path) {
            packages.extend(parsed.packages);
        }
    }
    if packages.is_empty() {
        return Ok(Vec::new());
    }

    let advisories = db.get_all_advisories().map_err(|e| e.to_string())?;
    let matches = matcher::find_matches(&advisories, &packages, &path);

    Ok(matches
        .into_iter()
        .map(|m| AlertSummary {
            id: format!(
                "{}::{}::{}",
                m.advisory.id, m.package.name, m.package.version
            ),
            advisory_id: m.advisory.id,
            ecosystem: m.package.ecosystem.to_string(),
            package: m.package.name,
            version: m.package.version.to_string(),
            severity: m
                .advisory
                .severity
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            summary: m.advisory.summary,
            references: m.advisory.references,
            project_path: m.project_path.to_string_lossy().into_owned(),
        })
        .collect())
}
