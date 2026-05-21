use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::glob_to_regex;
use crate::platform;
use crate::types::Severity;

#[derive(Debug, thiserror::Error)]
pub enum IocError {
    #[error("could not parse compiled IOC profile: {0}")]
    ParseCompiled(toml::de::Error),
    #[error("could not parse IOC profile {path}: {source}")]
    ParseFile {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("could not read IOC directory {path}: {source}")]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not read IOC profile {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct IocProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub date: String,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub packages: Option<IocPackages>,
    #[serde(default)]
    pub indicators: IocIndicators,
    #[serde(default)]
    pub credentials: IocCredentials,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IocPackages {
    pub ecosystem: String,
    pub names: Vec<String>,
    pub bad_versions: Vec<String>,
    pub clean_version: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct IocIndicators {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub persistence_paths: Vec<String>,
    #[serde(default)]
    pub mcp_configs: Vec<String>,
    #[serde(default)]
    pub ai_tool_configs: Vec<String>,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub ips: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct IocCredentials {
    #[serde(default)]
    pub targeted: Vec<String>,
    #[serde(default)]
    pub dead_man_switch: bool,
    #[serde(default)]
    pub rotation_warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IocFinding {
    pub path: PathBuf,
    pub description: String,
    pub severity: Severity,
    pub profile_id: String,
}

pub struct IocProfileSet {
    profiles: Vec<IocProfile>,
}

impl IocProfileSet {
    pub fn load_compiled() -> Result<Self, IocError> {
        let mut profiles = Vec::new();

        let sources: &[&str] = &[
            include_str!("../../../../iocs/tanstack-2026-05.toml"),
            include_str!("../../../../iocs/mini-shai-hulud.toml"),
        ];

        for source in sources {
            let profile: IocProfile = toml::from_str(source).map_err(IocError::ParseCompiled)?;
            profiles.push(profile);
        }

        Ok(Self { profiles })
    }

    pub fn load_user_profiles(dir: &Path) -> Result<Self, IocError> {
        let mut profiles = Vec::new();

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self { profiles });
            }
            Err(e) => {
                return Err(IocError::ReadDir {
                    path: dir.to_path_buf(),
                    source: e,
                });
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("could not read entry in {}: {e}", dir.display());
                    continue;
                }
            };
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            let content = std::fs::read_to_string(&path).map_err(|e| IocError::ReadFile {
                path: path.clone(),
                source: e,
            })?;

            let profile: IocProfile =
                toml::from_str(&content).map_err(|e| IocError::ParseFile {
                    path: path.clone(),
                    source: e,
                })?;
            profiles.push(profile);
        }

        Ok(Self { profiles })
    }

    pub fn merge(base: Self, user: Self) -> Self {
        let mut profiles = base.profiles;
        for user_profile in user.profiles {
            if let Some(pos) = profiles.iter().position(|p| p.id == user_profile.id) {
                profiles[pos] = user_profile;
            } else {
                profiles.push(user_profile);
            }
        }
        Self { profiles }
    }

    pub fn from_profiles(profiles: Vec<IocProfile>) -> Self {
        Self { profiles }
    }

    pub fn profiles(&self) -> &[IocProfile] {
        &self.profiles
    }
}

pub fn scan_iocs(path: &Path, profiles: &[IocProfile]) -> Vec<IocFinding> {
    let mut findings = Vec::new();

    for profile in profiles {
        for pattern in &profile.indicators.files {
            let expanded = expand_tilde(pattern);
            let matched_paths = resolve_glob(path, &expanded);
            for matched in matched_paths {
                if matched.exists() {
                    findings.push(IocFinding {
                        path: matched,
                        description: format!("IOC file detected ({}): {}", profile.name, pattern),
                        severity: Severity::Critical,
                        profile_id: profile.id.clone(),
                    });
                }
            }
        }

        for pattern in &profile.indicators.persistence_paths {
            let expanded = expand_tilde(pattern);
            let matched_paths = resolve_glob(path, &expanded);
            for matched in matched_paths {
                if matched.exists() {
                    findings.push(IocFinding {
                        path: matched,
                        description: format!(
                            "Persistence path exists ({}): {}",
                            profile.name, pattern
                        ),
                        severity: Severity::High,
                        profile_id: profile.id.clone(),
                    });
                }
            }
        }

        for pattern in &profile.indicators.mcp_configs {
            let expanded = expand_tilde(pattern);
            let matched_paths = resolve_glob(path, &expanded);
            for matched in matched_paths {
                if matched.exists() {
                    findings.push(IocFinding {
                        path: matched,
                        description: format!("MCP config detected ({}): {}", profile.name, pattern),
                        severity: Severity::High,
                        profile_id: profile.id.clone(),
                    });
                }
            }
        }
    }

    findings
}

fn expand_tilde(pattern: &str) -> String {
    platform::expand_tilde(pattern)
}

fn resolve_glob(base: &Path, pattern: &str) -> Vec<PathBuf> {
    let pattern_path = Path::new(pattern);

    if pattern_path.is_absolute() {
        return resolve_glob_absolute(pattern);
    }

    resolve_glob_relative(base, pattern)
}

fn resolve_glob_absolute(pattern: &str) -> Vec<PathBuf> {
    if !pattern.contains('*') {
        return vec![PathBuf::from(pattern)];
    }

    let parts: Vec<&str> = pattern.split('/').collect();
    let mut candidates = vec![PathBuf::from("/")];

    for part in &parts {
        if part.is_empty() {
            continue;
        }
        candidates = expand_glob_segment(&candidates, part);
    }

    candidates
}

fn resolve_glob_relative(base: &Path, pattern: &str) -> Vec<PathBuf> {
    if !pattern.contains('*') {
        return vec![base.join(pattern)];
    }

    let parts: Vec<&str> = pattern.split('/').collect();
    let mut candidates = vec![base.to_path_buf()];

    for part in &parts {
        if part.is_empty() {
            continue;
        }
        candidates = expand_glob_segment(&candidates, part);
    }

    candidates
}

fn expand_glob_segment(parents: &[PathBuf], segment: &str) -> Vec<PathBuf> {
    if !segment.contains('*') {
        return parents.iter().map(|p| p.join(segment)).collect();
    }

    let regex_pattern = glob_to_regex(segment);
    let re = match regex::Regex::new(&regex_pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for parent in parents {
        let entries = match std::fs::read_dir(parent) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && re.is_match(name)
            {
                results.push(entry.path());
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile() -> IocProfile {
        IocProfile {
            id: "test-profile".into(),
            name: "Test Profile".into(),
            description: "For testing".into(),
            date: "2026-05-20".into(),
            references: vec![],
            packages: None,
            indicators: IocIndicators {
                files: vec![
                    ".claude/execution.js".into(),
                    ".claude/setup.mjs".into(),
                    ".vscode/tasks.json".into(),
                    ".mcp.json".into(),
                ],
                persistence_paths: vec![],
                mcp_configs: vec![".cursor/mcp.json".into()],
                ai_tool_configs: vec![],
                domains: vec![],
                ips: vec![],
            },
            credentials: IocCredentials::default(),
        }
    }

    #[test]
    fn test_scan_iocs_finds_planted_files() {
        let dir = tempfile::tempdir().expect("tempdir");

        let claude_dir = dir.path().join(".claude");
        std::fs::create_dir_all(&claude_dir).expect("mkdir");
        std::fs::write(claude_dir.join("execution.js"), "malicious code").expect("write");

        std::fs::write(dir.path().join(".mcp.json"), "{}").expect("write");

        let profile = test_profile();
        let findings = scan_iocs(dir.path(), &[profile]);

        assert_eq!(findings.len(), 2);
        assert!(findings.iter().any(|f| f.path.ends_with("execution.js")));
        assert!(findings.iter().any(|f| f.path.ends_with(".mcp.json")));
        assert!(findings.iter().all(|f| f.profile_id == "test-profile"));
    }

    #[test]
    fn test_scan_iocs_empty_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let profile = test_profile();
        let findings = scan_iocs(dir.path(), &[profile]);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_scan_iocs_no_profiles() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".mcp.json"), "{}").expect("write");
        let findings = scan_iocs(dir.path(), &[]);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_scan_iocs_glob_pattern() {
        let dir = tempfile::tempdir().expect("tempdir");
        let tanstack_dir = dir.path().join("node_modules/@tanstack/react-router");
        std::fs::create_dir_all(&tanstack_dir).expect("mkdir");
        std::fs::write(tanstack_dir.join("router_init.js"), "payload").expect("write");

        let profile = IocProfile {
            id: "glob-test".into(),
            name: "Glob Test".into(),
            description: "Tests glob matching".into(),
            date: "2026-05-20".into(),
            references: vec![],
            packages: None,
            indicators: IocIndicators {
                files: vec!["node_modules/@tanstack/*/router_init.js".into()],
                persistence_paths: vec![],
                mcp_configs: vec![],
                ai_tool_configs: vec![],
                domains: vec![],
                ips: vec![],
            },
            credentials: IocCredentials::default(),
        };

        let findings = scan_iocs(dir.path(), &[profile]);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].path.ends_with("router_init.js"));
    }

    #[test]
    fn test_scan_iocs_mcp_configs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cursor_dir = dir.path().join(".cursor");
        std::fs::create_dir_all(&cursor_dir).expect("mkdir");
        std::fs::write(cursor_dir.join("mcp.json"), "{}").expect("write");

        let profile = test_profile();
        let findings = scan_iocs(dir.path(), &[profile]);

        assert_eq!(findings.len(), 1);
        assert!(findings[0].path.ends_with("mcp.json"));
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_scan_iocs_severity_by_category() {
        let dir = tempfile::tempdir().expect("tempdir");

        let claude_dir = dir.path().join(".claude");
        std::fs::create_dir_all(&claude_dir).expect("mkdir");
        std::fs::write(claude_dir.join("execution.js"), "malicious").expect("write");

        let profile = IocProfile {
            id: "severity-test".into(),
            name: "Severity Test".into(),
            description: "Tests severity assignment".into(),
            date: "2026-05-20".into(),
            references: vec![],
            packages: None,
            indicators: IocIndicators {
                files: vec![".claude/execution.js".into()],
                persistence_paths: vec![".claude/execution.js".into()],
                mcp_configs: vec![],
                ai_tool_configs: vec![],
                domains: vec![],
                ips: vec![],
            },
            credentials: IocCredentials::default(),
        };

        let findings = scan_iocs(dir.path(), &[profile]);
        let file_finding = findings
            .iter()
            .find(|f| f.description.contains("IOC file"))
            .expect("should have IOC file finding");
        let persist_finding = findings
            .iter()
            .find(|f| f.description.contains("Persistence"))
            .expect("should have persistence finding");

        assert_eq!(file_finding.severity, Severity::Critical);
        assert_eq!(persist_finding.severity, Severity::High);
    }

    #[test]
    fn test_glob_to_regex() {
        let re = glob_to_regex("*.js");
        assert_eq!(re, "^.*\\.js$");

        let re = glob_to_regex("router_init.js");
        assert_eq!(re, "^router_init\\.js$");
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/test/path");
        assert!(!expanded.starts_with('~'));

        let no_tilde = expand_tilde("/absolute/path");
        assert_eq!(no_tilde, "/absolute/path");

        let relative = expand_tilde("relative/path");
        assert_eq!(relative, "relative/path");
    }

    #[test]
    fn test_ioc_profile_deserialization() {
        let toml_str = r#"
id = "test-001"
name = "Test Attack"
description = "A test attack profile"
date = "2026-05-20"
references = ["https://example.com"]

[packages]
ecosystem = "npm"
names = ["bad-package"]
bad_versions = ["1.0.0"]
clean_version = "1.0.1"

[indicators]
files = [".evil/payload.js"]
persistence_paths = ["~/.bashrc"]
mcp_configs = [".mcp.json"]
domains = ["evil.example.com"]
ips = ["192.0.2.1"]

[credentials]
targeted = ["~/.npmrc"]
dead_man_switch = true
rotation_warning = "Back up first"
"#;

        let profile: IocProfile = toml::from_str(toml_str).expect("parse");
        assert_eq!(profile.id, "test-001");
        assert_eq!(profile.indicators.files.len(), 1);
        assert_eq!(profile.credentials.targeted.len(), 1);
        assert!(profile.credentials.dead_man_switch);
        assert!(profile.packages.is_some());

        let pkgs = profile.packages.expect("packages");
        assert_eq!(pkgs.ecosystem, "npm");
        assert_eq!(pkgs.bad_versions, vec!["1.0.0"]);
    }

    #[test]
    fn test_profile_set_merge_overrides() {
        let base = IocProfileSet {
            profiles: vec![IocProfile {
                id: "base-001".into(),
                name: "Base".into(),
                description: "Base profile".into(),
                date: "2026-01-01".into(),
                references: vec![],
                packages: None,
                indicators: IocIndicators::default(),
                credentials: IocCredentials::default(),
            }],
        };
        let user = IocProfileSet {
            profiles: vec![IocProfile {
                id: "base-001".into(),
                name: "User Override".into(),
                description: "Overridden".into(),
                date: "2026-05-20".into(),
                references: vec![],
                packages: None,
                indicators: IocIndicators::default(),
                credentials: IocCredentials::default(),
            }],
        };

        let merged = IocProfileSet::merge(base, user);
        assert_eq!(merged.profiles().len(), 1);
        assert_eq!(merged.profiles()[0].name, "User Override");
    }

    #[test]
    fn test_profile_set_merge_appends() {
        let base = IocProfileSet {
            profiles: vec![IocProfile {
                id: "base-001".into(),
                name: "Base".into(),
                description: "Base profile".into(),
                date: "2026-01-01".into(),
                references: vec![],
                packages: None,
                indicators: IocIndicators::default(),
                credentials: IocCredentials::default(),
            }],
        };
        let user = IocProfileSet {
            profiles: vec![IocProfile {
                id: "user-001".into(),
                name: "User".into(),
                description: "User profile".into(),
                date: "2026-05-20".into(),
                references: vec![],
                packages: None,
                indicators: IocIndicators::default(),
                credentials: IocCredentials::default(),
            }],
        };

        let merged = IocProfileSet::merge(base, user);
        assert_eq!(merged.profiles().len(), 2);
    }

    #[test]
    fn test_user_profiles_nonexistent_dir() {
        let set = IocProfileSet::load_user_profiles(Path::new("/nonexistent/path"))
            .expect("should succeed");
        assert!(set.profiles().is_empty());
    }

    #[test]
    fn test_load_compiled_profiles() {
        let set = IocProfileSet::load_compiled().expect("load compiled profiles");
        let profiles = set.profiles();
        assert_eq!(profiles.len(), 2);

        let tanstack = profiles.iter().find(|p| p.id == "tanstack-2026-05");
        assert!(tanstack.is_some());
        let tanstack = tanstack.expect("checked above");
        assert!(!tanstack.indicators.files.is_empty());
        assert!(tanstack.packages.is_some());

        let shai = profiles.iter().find(|p| p.id == "mini-shai-hulud");
        assert!(shai.is_some());
        let shai = shai.expect("checked above");
        assert!(shai.credentials.dead_man_switch);
        assert!(shai.credentials.rotation_warning.is_some());
    }
}
