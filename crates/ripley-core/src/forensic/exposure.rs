use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::credentials;
use super::ioc::{IocProfile, IocProfileSet};
use crate::types::Severity;

#[derive(Debug, thiserror::Error)]
pub enum ExposureError {
    #[error("IOC profile error: {0}")]
    Ioc(#[from] super::ioc::IocError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveExposureReport {
    pub cve_id: String,
    pub attack_name: String,
    pub at_risk: Vec<CredentialRisk>,
    pub clean: Vec<String>,
    pub dead_man_switch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRisk {
    pub path: PathBuf,
    pub exists: bool,
    pub contains_secret: bool,
    pub rotation_command: String,
    pub priority: Severity,
}

pub fn assess_cve_exposure(
    cve_id: &str,
    profiles: &IocProfileSet,
    home: &Path,
) -> Result<CveExposureReport, ExposureError> {
    let relevant: Vec<&IocProfile> = profiles
        .profiles()
        .iter()
        .filter(|p| {
            p.references.iter().any(|r| r.contains(cve_id))
                || p.id.contains(cve_id)
                || p.name.contains(cve_id)
        })
        .collect();

    if relevant.is_empty() {
        return Ok(CveExposureReport {
            cve_id: cve_id.to_string(),
            attack_name: "Unknown".to_string(),
            at_risk: Vec::new(),
            clean: vec!["No IOC profile found for this CVE — manual assessment needed".to_string()],
            dead_man_switch: None,
        });
    }

    let profile = relevant[0];
    let exposure_report =
        credentials::assess_exposure(&relevant.iter().copied().cloned().collect::<Vec<_>>(), home);

    let mut at_risk = Vec::new();
    let mut clean = Vec::new();

    for target in &profile.credentials.targeted {
        let expanded = credentials::expand_cred_path(target, home);
        let path = PathBuf::from(&expanded);
        let exists = path.exists();

        if exists {
            let contains_secret = check_contains_secret(&path);
            let rotation_command = credentials::rotation_command_for(&path)
                .unwrap_or_else(|| "Rotate this credential manually".to_string());
            let priority = if profile.credentials.dead_man_switch {
                Severity::Critical
            } else {
                Severity::High
            };
            at_risk.push(CredentialRisk {
                path,
                exists,
                contains_secret,
                rotation_command,
                priority,
            });
        } else {
            clean.push(format!("{target} — not present"));
        }
    }

    if at_risk.is_empty() && exposure_report.findings.is_empty() {
        clean.push("No targeted credentials found on this machine".to_string());
    }

    for finding in &exposure_report.findings {
        if !at_risk.iter().any(|r| r.path == finding.path) {
            at_risk.push(CredentialRisk {
                path: finding.path.clone(),
                exists: true,
                contains_secret: true,
                rotation_command: finding
                    .rotation_command
                    .clone()
                    .unwrap_or_else(|| "Rotate this credential manually".to_string()),
                priority: finding.severity,
            });
        }
    }

    Ok(CveExposureReport {
        cve_id: cve_id.to_string(),
        attack_name: profile.name.clone(),
        at_risk,
        clean,
        dead_man_switch: exposure_report.dead_man_switch_warning,
    })
}

fn check_contains_secret(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let secret_keys = [
        "password",
        "token",
        "secret",
        "api_key",
        "apikey",
        "auth_token",
        "_auth",
    ];
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        let lower = trimmed.to_lowercase();
        // Look for key=value or key: value with non-empty, non-placeholder values
        for sep in ["=", ": ", "\":"] {
            if let Some(pos) = lower.find(sep) {
                let key = &lower[..pos];
                let value = lower[pos + sep.len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if value.is_empty()
                    || value == "null"
                    || value == "none"
                    || value.starts_with("${")
                    || value.starts_with('$')
                {
                    continue;
                }
                if secret_keys.iter().any(|k| key.contains(k)) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forensic::ioc::{IocCredentials, IocIndicators, IocProfile};

    fn profile_set_with(profiles: Vec<IocProfile>) -> IocProfileSet {
        IocProfileSet::from_profiles(profiles)
    }

    fn test_profile(cve_id: &str, targeted: Vec<String>, dead_man_switch: bool) -> IocProfile {
        IocProfile {
            id: format!("profile-{cve_id}"),
            name: format!("Attack for {cve_id}"),
            description: "test".into(),
            date: "2026-01-01".into(),
            references: vec![format!("https://example.com/{cve_id}")],
            packages: None,
            indicators: IocIndicators::default(),
            credentials: IocCredentials {
                targeted,
                dead_man_switch,
                rotation_warning: if dead_man_switch {
                    Some("Back up before rotating!".into())
                } else {
                    None
                },
            },
        }
    }

    #[test]
    fn test_assess_cve_exposure_with_existing_creds() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".npmrc"), "_authToken=npm_abc123secret\n").expect("write");

        let profile = test_profile("CVE-2024-001", vec!["~/.npmrc".into()], false);
        let profiles = profile_set_with(vec![profile]);

        let report =
            assess_cve_exposure("CVE-2024-001", &profiles, dir.path()).expect("should not error");

        assert_eq!(report.cve_id, "CVE-2024-001");
        assert!(!report.at_risk.is_empty());
        assert!(report.at_risk[0].exists);
        assert!(report.at_risk[0].contains_secret);
        assert_eq!(report.at_risk[0].priority, Severity::High);
        assert!(report.dead_man_switch.is_none());
    }

    #[test]
    fn test_assess_cve_exposure_no_profile() {
        let dir = tempfile::tempdir().expect("tempdir");
        let profiles = profile_set_with(vec![]);

        let report =
            assess_cve_exposure("CVE-2099-9999", &profiles, dir.path()).expect("should not error");

        assert!(report.at_risk.is_empty());
        assert!(!report.clean.is_empty());
        assert!(report.clean[0].contains("No IOC profile"));
    }

    #[test]
    fn test_dead_man_switch_warning() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".npmrc"), "token=secret\n").expect("write");

        let profile = test_profile("CVE-2024-DMS", vec!["~/.npmrc".into()], true);
        let profiles = profile_set_with(vec![profile]);

        let report =
            assess_cve_exposure("CVE-2024-DMS", &profiles, dir.path()).expect("should not error");

        assert!(report.dead_man_switch.is_some());
        assert!(!report.at_risk.is_empty());
        assert_eq!(report.at_risk[0].priority, Severity::Critical);
    }

    #[test]
    fn test_credential_not_present_is_clean() {
        let dir = tempfile::tempdir().expect("tempdir");

        let profile = test_profile(
            "CVE-2024-CLEAN",
            vec!["~/.npmrc".into(), "~/.aws/credentials".into()],
            false,
        );
        let profiles = profile_set_with(vec![profile]);

        let report =
            assess_cve_exposure("CVE-2024-CLEAN", &profiles, dir.path()).expect("should not error");

        assert!(report.at_risk.is_empty());
        assert!(!report.clean.is_empty());
    }
}
