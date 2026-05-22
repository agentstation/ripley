use std::path::Path;
use std::time::Instant;

use crate::sandbox::{SandboxProfile, SandboxResult, ViolationKind};
use crate::types::{Ecosystem, Severity};

use super::report::{
    AnomalyKind, BehavioralAnomaly, BehavioralReport, DeclaredBehavior, FsRead, FsWrite,
    NetworkAttempt, ObservedBehavior, ProcessSpawn,
};

#[derive(Debug, thiserror::Error)]
pub enum BehavioralError {
    #[error("manifest not found at {0}")]
    ManifestNotFound(String),
    #[error("parse error: {0}")]
    ParseError(String),
}

pub fn extract_declared_behavior(
    package_dir: &Path,
    ecosystem: Ecosystem,
) -> Result<DeclaredBehavior, BehavioralError> {
    match ecosystem {
        Ecosystem::Npm => extract_npm_declared(package_dir),
        Ecosystem::Cargo => extract_cargo_declared(package_dir),
        Ecosystem::PyPI => extract_pip_declared(package_dir),
        _ => Ok(DeclaredBehavior::default()),
    }
}

fn extract_npm_declared(package_dir: &Path) -> Result<DeclaredBehavior, BehavioralError> {
    let manifest_path = package_dir.join("package.json");
    let content = std::fs::read_to_string(&manifest_path).map_err(|_| {
        BehavioralError::ManifestNotFound(manifest_path.to_string_lossy().to_string())
    })?;

    let parsed: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| BehavioralError::ParseError(e.to_string()))?;

    let install_script_keys = ["preinstall", "postinstall", "install", "prepare"];
    let mut script_names = Vec::new();
    let mut has_install_scripts = false;

    if let Some(scripts) = parsed.get("scripts").and_then(|s| s.as_object()) {
        for key in &install_script_keys {
            if scripts.contains_key(*key) {
                has_install_scripts = true;
                script_names.push((*key).to_string());
            }
        }
    }

    let name = parsed
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or_default();
    let known_build_tool = is_known_build_tool(name, Ecosystem::Npm);

    let declared_dependencies = parsed
        .get("dependencies")
        .and_then(|d| d.as_object())
        .map(|deps| deps.keys().cloned().collect())
        .unwrap_or_default();

    Ok(DeclaredBehavior {
        has_install_scripts,
        script_names,
        declared_dependencies,
        known_build_tool,
    })
}

fn extract_cargo_declared(package_dir: &Path) -> Result<DeclaredBehavior, BehavioralError> {
    let manifest_path = package_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest_path).map_err(|_| {
        BehavioralError::ManifestNotFound(manifest_path.to_string_lossy().to_string())
    })?;

    let parsed: toml::Value =
        toml::from_str(&content).map_err(|e| BehavioralError::ParseError(e.to_string()))?;

    let has_build_script = parsed.get("package").and_then(|p| p.get("build")).is_some();

    let has_links = parsed.get("package").and_then(|p| p.get("links")).is_some();

    let name = parsed
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or_default();

    let known_build_tool = is_known_build_tool(name, Ecosystem::Cargo);

    let mut script_names = Vec::new();
    if has_build_script {
        script_names.push("build.rs".to_string());
    }

    Ok(DeclaredBehavior {
        has_install_scripts: has_build_script || has_links,
        script_names,
        declared_dependencies: Vec::new(),
        known_build_tool,
    })
}

fn extract_pip_declared(package_dir: &Path) -> Result<DeclaredBehavior, BehavioralError> {
    let manifest_path = package_dir.join("pyproject.toml");
    let content = std::fs::read_to_string(&manifest_path).map_err(|_| {
        BehavioralError::ManifestNotFound(manifest_path.to_string_lossy().to_string())
    })?;

    let parsed: toml::Value =
        toml::from_str(&content).map_err(|e| BehavioralError::ParseError(e.to_string()))?;

    let has_build_system = parsed.get("build-system").is_some();

    Ok(DeclaredBehavior {
        has_install_scripts: has_build_system,
        script_names: if has_build_system {
            vec!["build-system".to_string()]
        } else {
            Vec::new()
        },
        declared_dependencies: Vec::new(),
        known_build_tool: false,
    })
}

pub fn is_known_build_tool(package_name: &str, ecosystem: Ecosystem) -> bool {
    match ecosystem {
        Ecosystem::Npm => {
            const NPM_BUILD_TOOLS: &[&str] = &[
                "node-gyp",
                "node-pre-gyp",
                "prebuild-install",
                "esbuild",
                "webpack",
                "rollup",
                "vite",
                "turbo",
                "parcel",
                "babel",
                "typescript",
                "tsc",
                "swc",
            ];
            NPM_BUILD_TOOLS.contains(&package_name)
        }
        Ecosystem::Cargo => {
            const CARGO_BUILD_TOOLS: &[&str] =
                &["cc", "cmake", "pkg-config", "bindgen", "protobuf-codegen"];
            CARGO_BUILD_TOOLS.contains(&package_name)
        }
        _ => false,
    }
}

pub fn sandbox_result_to_observed(
    result: &SandboxResult,
    profile: &SandboxProfile,
) -> ObservedBehavior {
    let mut network_attempts = Vec::new();
    let mut fs_writes = Vec::new();
    let mut fs_reads = Vec::new();
    let mut process_spawns = Vec::new();

    for violation in &result.sandbox_violations {
        match violation.kind {
            ViolationKind::NetworkAccess => {
                let host = extract_host_from_detail(&violation.detail);
                network_attempts.push(NetworkAttempt {
                    host,
                    port: None,
                    blocked: true,
                });
            }
            ViolationKind::FileWriteOutsideScope => {
                let path = extract_path_from_detail(&violation.detail);
                let outside_package = !profile
                    .writable_paths
                    .iter()
                    .any(|wp| path.starts_with(&wp.to_string_lossy().to_string()));
                fs_writes.push(FsWrite {
                    path,
                    blocked: true,
                    outside_package,
                });
            }
            ViolationKind::ProcessSpawn => {
                process_spawns.push(ProcessSpawn {
                    command: violation.detail.clone(),
                    args: Vec::new(),
                });
            }
            ViolationKind::Other => {
                if violation.detail.contains("file-read") {
                    let path = extract_path_from_detail(&violation.detail);
                    let sensitive = is_sensitive_path(&path);
                    fs_reads.push(FsRead { path, sensitive });
                }
            }
        }
    }

    ObservedBehavior {
        network_attempts,
        fs_writes,
        fs_reads,
        process_spawns,
        exit_code: result.exit_code,
    }
}

pub fn detect_anomalies(
    declared: &DeclaredBehavior,
    observed: &ObservedBehavior,
) -> Vec<BehavioralAnomaly> {
    let mut anomalies = Vec::new();

    // Rule 1: Network from non-network package
    if !observed.network_attempts.is_empty() && !declared.known_build_tool {
        for attempt in &observed.network_attempts {
            anomalies.push(BehavioralAnomaly {
                kind: AnomalyKind::UnexpectedNetwork,
                severity: Severity::High,
                description: format!("package attempted network access to {}", attempt.host),
                evidence: attempt.host.clone(),
            });
        }
    }

    // Rule 2: Writes outside package dir
    for write in &observed.fs_writes {
        if write.outside_package {
            anomalies.push(BehavioralAnomaly {
                kind: AnomalyKind::ScopeEscape,
                severity: Severity::High,
                description: format!("write attempt outside package directory: {}", write.path),
                evidence: write.path.clone(),
            });
        }
    }

    // Rule 3: Reads credential paths
    for read in &observed.fs_reads {
        if read.sensitive {
            anomalies.push(BehavioralAnomaly {
                kind: AnomalyKind::CredentialAccess,
                severity: Severity::Critical,
                description: format!("read attempt on sensitive path: {}", read.path),
                evidence: read.path.clone(),
            });
        }
    }

    // Rule 4: Unexpected shell spawns
    for spawn in &observed.process_spawns {
        if is_suspicious_spawn(&spawn.command) {
            anomalies.push(BehavioralAnomaly {
                kind: AnomalyKind::SuspiciousSpawn,
                severity: Severity::Medium,
                description: format!("suspicious process spawn: {}", spawn.command),
                evidence: spawn.command.clone(),
            });
        }
    }

    // Rule 5: Behavior from package with no install scripts
    if !declared.has_install_scripts
        && (!observed.network_attempts.is_empty()
            || !observed.fs_writes.is_empty()
            || !observed.process_spawns.is_empty())
    {
        anomalies.push(BehavioralAnomaly {
            kind: AnomalyKind::BehaviorMismatch,
            severity: Severity::Critical,
            description: "observed behavior from package with no declared install scripts"
                .to_string(),
            evidence: "no install scripts declared".to_string(),
        });
    }

    anomalies
}

pub fn calculate_risk_score(anomalies: &[BehavioralAnomaly]) -> f64 {
    let mut score: f64 = 0.0;
    for anomaly in anomalies {
        score += match anomaly.severity {
            Severity::Critical => 0.4,
            Severity::High => 0.25,
            Severity::Medium => 0.15,
            Severity::Low => 0.05,
        };
    }
    score.min(1.0)
}

pub fn analyze_behavior(
    package_dir: &Path,
    ecosystem: Ecosystem,
    sandbox_result: &SandboxResult,
    profile: &SandboxProfile,
) -> Result<BehavioralReport, BehavioralError> {
    let start = Instant::now();

    let declared = extract_declared_behavior(package_dir, ecosystem).unwrap_or_default();
    let observed = sandbox_result_to_observed(sandbox_result, profile);
    let anomalies = detect_anomalies(&declared, &observed);
    let risk_score = calculate_risk_score(&anomalies);

    let package = std::env::var("npm_package_name").unwrap_or_default();
    let version = std::env::var("npm_package_version").unwrap_or_default();

    Ok(BehavioralReport {
        package,
        version,
        ecosystem,
        declared,
        observed,
        anomalies,
        risk_score,
        analysis_duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn extract_host_from_detail(detail: &str) -> String {
    // Try to extract a hostname/IP from sandbox denial messages
    // Example: "sandbox-exec(12345) deny(1) network-outbound connect 93.184.216.34:443"
    let parts: Vec<&str> = detail.split_whitespace().collect();
    if let Some(last) = parts.last() {
        if last.contains(':') {
            return last.split(':').next().unwrap_or(last).to_string();
        }
        if last.contains('.') || last.contains("::") {
            return (*last).to_string();
        }
    }
    "unknown".to_string()
}

fn extract_path_from_detail(detail: &str) -> String {
    // Try to extract a filesystem path from denial messages
    // Example: "sandbox-exec(12345) deny(1) file-write* /etc/evil"
    for part in detail.split_whitespace().rev() {
        if part.starts_with('/') {
            return part.to_string();
        }
    }
    detail.to_string()
}

fn is_sensitive_path(path: &str) -> bool {
    const SENSITIVE_PREFIXES: &[&str] = &[
        "/.ssh",
        "/.npmrc",
        "/.aws",
        "/.gnupg",
        "/.config/gh",
        "/.netrc",
        "/.docker/config",
        "/.kube/config",
        "/etc/shadow",
        "/etc/passwd",
    ];
    SENSITIVE_PREFIXES
        .iter()
        .any(|prefix| path.contains(prefix))
}

fn is_suspicious_spawn(command: &str) -> bool {
    const SUSPICIOUS: &[&str] = &[
        "curl",
        "wget",
        "nc",
        "ncat",
        "base64",
        "eval",
        "python -c",
        "ruby -e",
        "perl -e",
    ];
    let lower = command.to_lowercase();
    SUSPICIOUS.iter().any(|s| lower.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::SandboxViolation;

    #[test]
    fn test_extract_npm_declared_with_postinstall() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        std::fs::write(
            tmpdir.path().join("package.json"),
            r#"{"name": "test-pkg", "scripts": {"postinstall": "node setup.js"}}"#,
        )
        .expect("write");

        let declared = extract_declared_behavior(tmpdir.path(), Ecosystem::Npm).expect("extract");
        assert!(declared.has_install_scripts);
        assert!(declared.script_names.contains(&"postinstall".to_string()));
    }

    #[test]
    fn test_extract_npm_declared_without_scripts() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        std::fs::write(
            tmpdir.path().join("package.json"),
            r#"{"name": "simple-pkg", "scripts": {"test": "jest"}}"#,
        )
        .expect("write");

        let declared = extract_declared_behavior(tmpdir.path(), Ecosystem::Npm).expect("extract");
        assert!(!declared.has_install_scripts);
        assert!(declared.script_names.is_empty());
    }

    #[test]
    fn test_extract_npm_known_build_tool() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        std::fs::write(
            tmpdir.path().join("package.json"),
            r#"{"name": "node-gyp", "scripts": {"install": "node-gyp rebuild"}}"#,
        )
        .expect("write");

        let declared = extract_declared_behavior(tmpdir.path(), Ecosystem::Npm).expect("extract");
        assert!(declared.known_build_tool);
    }

    #[test]
    fn test_extract_cargo_declared_with_build() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        std::fs::write(
            tmpdir.path().join("Cargo.toml"),
            "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\nbuild = \"build.rs\"\n",
        )
        .expect("write");

        let declared = extract_declared_behavior(tmpdir.path(), Ecosystem::Cargo).expect("extract");
        assert!(declared.has_install_scripts);
        assert!(declared.script_names.contains(&"build.rs".to_string()));
    }

    #[test]
    fn test_manifest_not_found() {
        let tmpdir = tempfile::tempdir().expect("tmpdir");
        let result = extract_declared_behavior(tmpdir.path(), Ecosystem::Npm);
        assert!(matches!(result, Err(BehavioralError::ManifestNotFound(_))));
    }

    #[test]
    fn test_is_known_build_tool_npm() {
        assert!(is_known_build_tool("node-gyp", Ecosystem::Npm));
        assert!(is_known_build_tool("esbuild", Ecosystem::Npm));
        assert!(!is_known_build_tool("lodash", Ecosystem::Npm));
    }

    #[test]
    fn test_is_known_build_tool_cargo() {
        assert!(is_known_build_tool("cc", Ecosystem::Cargo));
        assert!(!is_known_build_tool("serde", Ecosystem::Cargo));
    }

    #[test]
    fn test_sandbox_result_to_observed_network() {
        let result = SandboxResult {
            exit_code: 1,
            stderr_output: String::new(),
            stdout_output: String::new(),
            network_blocked: true,
            duration_ms: 50,
            sandbox_violations: vec![SandboxViolation {
                kind: ViolationKind::NetworkAccess,
                detail: "deny(1) network-outbound connect 93.184.216.34:443".to_string(),
            }],
        };
        let profile = SandboxProfile::for_ecosystem(Ecosystem::Npm, std::path::Path::new("/pkg"));

        let observed = sandbox_result_to_observed(&result, &profile);
        assert_eq!(observed.network_attempts.len(), 1);
        assert_eq!(observed.network_attempts[0].host, "93.184.216.34");
        assert!(observed.network_attempts[0].blocked);
    }

    #[test]
    fn test_sandbox_result_to_observed_fs_write() {
        let result = SandboxResult {
            exit_code: 1,
            stderr_output: String::new(),
            stdout_output: String::new(),
            network_blocked: false,
            duration_ms: 50,
            sandbox_violations: vec![SandboxViolation {
                kind: ViolationKind::FileWriteOutsideScope,
                detail: "deny(1) file-write* /etc/evil".to_string(),
            }],
        };
        let profile = SandboxProfile::for_ecosystem(Ecosystem::Npm, std::path::Path::new("/pkg"));

        let observed = sandbox_result_to_observed(&result, &profile);
        assert_eq!(observed.fs_writes.len(), 1);
        assert_eq!(observed.fs_writes[0].path, "/etc/evil");
        assert!(observed.fs_writes[0].outside_package);
    }

    #[test]
    fn test_sandbox_result_clean_stderr() {
        let result = SandboxResult {
            exit_code: 0,
            stderr_output: String::new(),
            stdout_output: "ok\n".to_string(),
            network_blocked: false,
            duration_ms: 10,
            sandbox_violations: Vec::new(),
        };
        let profile = SandboxProfile::for_ecosystem(Ecosystem::Npm, std::path::Path::new("/pkg"));

        let observed = sandbox_result_to_observed(&result, &profile);
        assert!(observed.network_attempts.is_empty());
        assert!(observed.fs_writes.is_empty());
        assert_eq!(observed.exit_code, 0);
    }

    #[test]
    fn test_detect_anomalies_network_from_non_network_pkg() {
        let declared = DeclaredBehavior {
            has_install_scripts: true,
            known_build_tool: false,
            ..Default::default()
        };
        let observed = ObservedBehavior {
            network_attempts: vec![NetworkAttempt {
                host: "evil.com".to_string(),
                port: Some(443),
                blocked: true,
            }],
            ..Default::default()
        };

        let anomalies = detect_anomalies(&declared, &observed);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].kind, AnomalyKind::UnexpectedNetwork);
        assert_eq!(anomalies[0].severity, Severity::High);
    }

    #[test]
    fn test_detect_anomalies_no_anomaly_for_build_tool() {
        let declared = DeclaredBehavior {
            has_install_scripts: true,
            known_build_tool: true,
            ..Default::default()
        };
        let observed = ObservedBehavior {
            network_attempts: vec![NetworkAttempt {
                host: "registry.npmjs.org".to_string(),
                port: Some(443),
                blocked: true,
            }],
            ..Default::default()
        };

        let anomalies = detect_anomalies(&declared, &observed);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_detect_anomalies_credential_access() {
        let declared = DeclaredBehavior {
            has_install_scripts: true,
            ..Default::default()
        };
        let observed = ObservedBehavior {
            fs_reads: vec![FsRead {
                path: "/home/user/.ssh/id_rsa".to_string(),
                sensitive: true,
            }],
            ..Default::default()
        };

        let anomalies = detect_anomalies(&declared, &observed);
        assert!(
            anomalies
                .iter()
                .any(|a| a.kind == AnomalyKind::CredentialAccess)
        );
        assert!(anomalies.iter().any(|a| a.severity == Severity::Critical));
    }

    #[test]
    fn test_detect_anomalies_behavior_mismatch() {
        let declared = DeclaredBehavior {
            has_install_scripts: false,
            ..Default::default()
        };
        let observed = ObservedBehavior {
            network_attempts: vec![NetworkAttempt {
                host: "evil.com".to_string(),
                port: None,
                blocked: true,
            }],
            ..Default::default()
        };

        let anomalies = detect_anomalies(&declared, &observed);
        assert!(
            anomalies
                .iter()
                .any(|a| a.kind == AnomalyKind::BehaviorMismatch)
        );
        assert!(
            anomalies.iter().any(
                |a| a.kind == AnomalyKind::BehaviorMismatch && a.severity == Severity::Critical
            )
        );
    }

    #[test]
    fn test_calculate_risk_score_benign() {
        let anomalies: Vec<BehavioralAnomaly> = Vec::new();
        assert_eq!(calculate_risk_score(&anomalies), 0.0);
    }

    #[test]
    fn test_calculate_risk_score_critical() {
        let anomalies = vec![BehavioralAnomaly {
            kind: AnomalyKind::CredentialAccess,
            severity: Severity::Critical,
            description: "test".to_string(),
            evidence: "test".to_string(),
        }];
        assert!((calculate_risk_score(&anomalies) - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_risk_score_capped() {
        let anomalies: Vec<BehavioralAnomaly> = (0..10)
            .map(|_| BehavioralAnomaly {
                kind: AnomalyKind::CredentialAccess,
                severity: Severity::Critical,
                description: "test".to_string(),
                evidence: "test".to_string(),
            })
            .collect();
        assert_eq!(calculate_risk_score(&anomalies), 1.0);
    }

    #[test]
    fn test_is_sensitive_path() {
        assert!(is_sensitive_path("/home/user/.ssh/id_rsa"));
        assert!(is_sensitive_path("/home/user/.npmrc"));
        assert!(is_sensitive_path("/home/user/.aws/credentials"));
        assert!(!is_sensitive_path("/usr/local/bin/node"));
        assert!(!is_sensitive_path("/tmp/build.log"));
    }

    #[test]
    fn test_extract_host_from_detail() {
        assert_eq!(
            extract_host_from_detail("deny(1) network-outbound connect 93.184.216.34:443"),
            "93.184.216.34"
        );
        assert_eq!(
            extract_host_from_detail("deny(1) network-outbound connect 10.0.0.1"),
            "10.0.0.1"
        );
        assert_eq!(extract_host_from_detail("some unknown message"), "unknown");
    }

    #[test]
    fn test_extract_path_from_detail() {
        assert_eq!(
            extract_path_from_detail("deny(1) file-write* /etc/evil"),
            "/etc/evil"
        );
        assert_eq!(extract_path_from_detail("no path here"), "no path here");
    }
}
