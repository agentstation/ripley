pub mod creds;
pub mod pinning;
pub mod provenance;
pub mod settings;

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::audit::TrafficLight;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardenCategory {
    DependencyPinning,
    PmHardening,
    Provenance,
    CredentialHygiene,
}

impl fmt::Display for HardenCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HardenCategory::DependencyPinning => write!(f, "Dependency Pinning"),
            HardenCategory::PmHardening => write!(f, "PM Hardening"),
            HardenCategory::Provenance => write!(f, "Provenance"),
            HardenCategory::CredentialHygiene => write!(f, "Credential Hygiene"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardenFinding {
    pub name: String,
    pub status: TrafficLight,
    pub detail: String,
    pub fix_command: Option<String>,
    pub pm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardenCategoryReport {
    pub category: HardenCategory,
    pub findings: Vec<HardenFinding>,
    pub overall: TrafficLight,
}

impl HardenCategoryReport {
    pub fn new(category: HardenCategory, findings: Vec<HardenFinding>) -> Self {
        let overall = if findings.iter().any(|f| f.status == TrafficLight::Red) {
            TrafficLight::Red
        } else if findings.iter().any(|f| f.status == TrafficLight::Yellow) {
            TrafficLight::Yellow
        } else {
            TrafficLight::Green
        };
        Self {
            category,
            findings,
            overall,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPm {
    pub name: String,
    pub version: Option<String>,
    pub lockfile_path: PathBuf,
    pub config_path: Option<PathBuf>,
    pub binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardenReport {
    pub detected_pms: Vec<DetectedPm>,
    pub categories: Vec<HardenCategoryReport>,
    pub timestamp: String,
}

pub fn detect_package_managers(path: &Path) -> Vec<DetectedPm> {
    let lockfile_map: &[(&str, &str, bool)] = &[
        ("package-lock.json", "npm", false),
        ("pnpm-lock.yaml", "pnpm", false),
        ("yarn.lock", "yarn", false),
        ("bun.lockb", "bun", true),
        ("Cargo.lock", "cargo", false),
        ("go.sum", "go", false),
        ("Pipfile.lock", "pip", false),
        ("poetry.lock", "pip", false),
        ("Gemfile.lock", "gem", false),
    ];

    let mut detected = Vec::new();

    for (lockfile, pm_name, binary) in lockfile_map {
        let lockfile_path = path.join(lockfile);
        if lockfile_path.exists() {
            let version = get_pm_version(pm_name);
            let config_path = find_config_path(path, pm_name);
            detected.push(DetectedPm {
                name: pm_name.to_string(),
                version,
                lockfile_path,
                config_path,
                binary: *binary,
            });
        }
    }

    detected
}

fn get_pm_version(pm_name: &str) -> Option<String> {
    let cmd = match pm_name {
        "pip" => "pip3",
        other => other,
    };
    std::process::Command::new(cmd)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn find_config_path(project_path: &Path, pm_name: &str) -> Option<PathBuf> {
    match pm_name {
        "npm" | "pnpm" => {
            let npmrc = project_path.join(".npmrc");
            if npmrc.exists() { Some(npmrc) } else { None }
        }
        "yarn" => {
            let yarnrc = project_path.join(".yarnrc.yml");
            if yarnrc.exists() { Some(yarnrc) } else { None }
        }
        "bun" => {
            let bunfig = project_path.join("bunfig.toml");
            if bunfig.exists() { Some(bunfig) } else { None }
        }
        _ => None,
    }
}

pub fn run_harden(path: &Path, home: &Path) -> HardenReport {
    let detected_pms = detect_package_managers(path);

    let mut categories = Vec::new();

    if !detected_pms.is_empty() {
        let mut pinning_findings = Vec::new();
        let mut settings_findings = Vec::new();

        for pm in &detected_pms {
            pinning_findings.extend(pinning::check_dependency_pinning(pm));
            settings_findings.extend(settings::check_pm_hardening(pm));
        }

        categories.push(HardenCategoryReport::new(
            HardenCategory::DependencyPinning,
            pinning_findings,
        ));
        categories.push(HardenCategoryReport::new(
            HardenCategory::PmHardening,
            settings_findings,
        ));

        let provenance_findings = provenance::check_provenance(path);
        categories.push(HardenCategoryReport::new(
            HardenCategory::Provenance,
            provenance_findings,
        ));

        let cred_findings = creds::check_credential_hygiene(home, &detected_pms);
        categories.push(HardenCategoryReport::new(
            HardenCategory::CredentialHygiene,
            cred_findings,
        ));
    }

    HardenReport {
        detected_pms,
        categories,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_npm_from_lockfile() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("package-lock.json"), "{}").expect("write");
        let pms = detect_package_managers(dir.path());
        assert_eq!(pms.len(), 1);
        assert_eq!(pms[0].name, "npm");
        assert!(!pms[0].binary);
    }

    #[test]
    fn test_detect_multiple_pms() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("package-lock.json"), "{}").expect("write");
        std::fs::write(dir.path().join("pnpm-lock.yaml"), "").expect("write");
        let pms = detect_package_managers(dir.path());
        assert_eq!(pms.len(), 2);
    }

    #[test]
    fn test_detect_no_lockfiles() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pms = detect_package_managers(dir.path());
        assert!(pms.is_empty());
    }

    #[test]
    fn test_detect_bun_binary_flag() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("bun.lockb"), [0u8; 10]).expect("write");
        let pms = detect_package_managers(dir.path());
        assert_eq!(pms.len(), 1);
        assert_eq!(pms[0].name, "bun");
        assert!(pms[0].binary);
    }

    #[test]
    fn test_detect_cargo_lockfile() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("Cargo.lock"), "").expect("write");
        let pms = detect_package_managers(dir.path());
        assert_eq!(pms.len(), 1);
        assert_eq!(pms[0].name, "cargo");
        assert!(!pms[0].binary);
    }

    #[test]
    fn test_detect_go_lockfile() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("go.sum"), "").expect("write");
        let pms = detect_package_managers(dir.path());
        assert_eq!(pms.len(), 1);
        assert_eq!(pms[0].name, "go");
    }

    #[test]
    fn test_detect_pip_lockfile() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("Pipfile.lock"), "{}").expect("write");
        let pms = detect_package_managers(dir.path());
        assert_eq!(pms.len(), 1);
        assert_eq!(pms[0].name, "pip");
    }

    #[test]
    fn test_detect_gem_lockfile() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("Gemfile.lock"), "").expect("write");
        let pms = detect_package_managers(dir.path());
        assert_eq!(pms.len(), 1);
        assert_eq!(pms[0].name, "gem");
    }

    #[test]
    fn test_find_config_path_bun() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bunfig = dir.path().join("bunfig.toml");
        std::fs::write(&bunfig, "").expect("write");
        let result = find_config_path(dir.path(), "bun");
        assert_eq!(result, Some(bunfig));
    }

    #[test]
    fn test_find_config_path_bun_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let result = find_config_path(dir.path(), "bun");
        assert_eq!(result, None);
    }

    #[test]
    fn test_harden_category_display() {
        assert_eq!(
            HardenCategory::DependencyPinning.to_string(),
            "Dependency Pinning"
        );
        assert_eq!(HardenCategory::PmHardening.to_string(), "PM Hardening");
        assert_eq!(HardenCategory::Provenance.to_string(), "Provenance");
        assert_eq!(
            HardenCategory::CredentialHygiene.to_string(),
            "Credential Hygiene"
        );
    }

    #[test]
    fn test_harden_finding_partial_eq() {
        let a = HardenFinding {
            name: "test".to_string(),
            status: TrafficLight::Green,
            detail: "ok".to_string(),
            fix_command: None,
            pm: "npm".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
