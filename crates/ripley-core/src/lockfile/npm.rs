use std::collections::HashMap;

use serde::Deserialize;

use super::{InstalledPackage, LockfileError, LockfileWarning, ParsedLockfile, RiskySpec};
use crate::types::{Ecosystem, Severity};

#[derive(Deserialize)]
struct PackageLock {
    #[serde(default)]
    packages: HashMap<String, PackageEntry>,
}

#[derive(Deserialize)]
struct PackageEntry {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    resolved: Option<String>,
    #[serde(default)]
    integrity: Option<String>,
}

pub fn parse_package_lock(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let lock: PackageLock =
        serde_json::from_str(content).map_err(|e| LockfileError::Parse(e.to_string()))?;

    let mut packages = Vec::new();
    let mut risky_specs = Vec::new();
    let mut warnings = Vec::new();

    for (key, entry) in &lock.packages {
        if key.is_empty() {
            continue;
        }

        let name = extract_package_name(key);
        let version_str = match &entry.version {
            Some(v) => v,
            None => continue,
        };

        let version = match semver::Version::parse(version_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        packages.push(InstalledPackage {
            name: name.clone(),
            version,
            ecosystem: Ecosystem::Npm,
        });

        if let Some(ref resolved) = entry.resolved {
            check_resolved_url(&name, resolved, &mut risky_specs, &mut warnings);
        }

        check_integrity(&name, &entry.integrity, &mut warnings);
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages,
        risky_specs,
        warnings,
    })
}

fn extract_package_name(key: &str) -> String {
    let stripped = key
        .rfind("node_modules/")
        .map(|pos| &key[pos + "node_modules/".len()..])
        .unwrap_or(key);
    stripped.to_string()
}

fn check_resolved_url(
    name: &str,
    resolved: &str,
    risky_specs: &mut Vec<RiskySpec>,
    warnings: &mut Vec<LockfileWarning>,
) {
    if resolved.starts_with("git+") || resolved.starts_with("git://") {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: resolved.to_string(),
            reason: "resolves from git source — bypasses registry integrity".to_string(),
        });
    }

    if resolved.starts_with("file:") {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: resolved.to_string(),
            reason: "resolves from local file — not reproducible".to_string(),
        });
    }

    if resolved.starts_with("http://") {
        warnings.push(LockfileWarning {
            package: name.to_string(),
            field: "resolved".to_string(),
            message: "HTTP downgrade — resolved URL uses http:// instead of https://".to_string(),
            severity: Severity::High,
        });
    }

    if !resolved.starts_with("https://registry.npmjs.org")
        && !resolved.starts_with("git+")
        && !resolved.starts_with("git://")
        && !resolved.starts_with("file:")
        && !resolved.starts_with("http://")
        && !resolved.is_empty()
    {
        // Non-standard registry — not necessarily bad, but worth noting
    }
}

fn check_integrity(name: &str, integrity: &Option<String>, warnings: &mut Vec<LockfileWarning>) {
    match integrity {
        None => {
            warnings.push(LockfileWarning {
                package: name.to_string(),
                field: "integrity".to_string(),
                message: "missing integrity hash".to_string(),
                severity: Severity::Medium,
            });
        }
        Some(hash) => {
            if hash.starts_with("sha1-") {
                warnings.push(LockfileWarning {
                    package: name.to_string(),
                    field: "integrity".to_string(),
                    message: "weak integrity hash (SHA-1) — should use SHA-512".to_string(),
                    severity: Severity::Medium,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let content = std::fs::read_to_string("../../tests/fixtures/package-lock.json")
            .expect("read fixture");
        let parsed = parse_package_lock(&content).expect("parse");

        assert!(!parsed.packages.is_empty());
        assert!(
            parsed.packages.len() >= 15,
            "expected at least 15 packages, got {}",
            parsed.packages.len()
        );

        let lodash = parsed
            .packages
            .iter()
            .find(|p| p.name == "lodash")
            .expect("lodash should be present");
        assert_eq!(lodash.version, semver::Version::new(4, 17, 20));
        assert_eq!(lodash.ecosystem, Ecosystem::Npm);

        let tanstack = parsed
            .packages
            .iter()
            .find(|p| p.name == "@tanstack/react-query")
            .expect("scoped package should be present");
        assert_eq!(tanstack.ecosystem, Ecosystem::Npm);
    }

    #[test]
    fn test_risky_specs() {
        let content = std::fs::read_to_string("../../tests/fixtures/package-lock-risky.json")
            .expect("read fixture");
        let parsed = parse_package_lock(&content).expect("parse");

        let git_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("git"))
            .collect();
        assert!(!git_risks.is_empty(), "should flag git+ resolved URLs");

        let file_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("file"))
            .collect();
        assert!(!file_risks.is_empty(), "should flag file: resolved URLs");

        let http_warnings: Vec<_> = parsed
            .warnings
            .iter()
            .filter(|w| w.message.contains("HTTP downgrade"))
            .collect();
        assert!(!http_warnings.is_empty(), "should warn about http:// URLs");

        let missing_integrity: Vec<_> = parsed
            .warnings
            .iter()
            .filter(|w| w.message.contains("missing integrity"))
            .collect();
        assert!(
            !missing_integrity.is_empty(),
            "should warn about missing integrity hashes"
        );

        let weak_hash: Vec<_> = parsed
            .warnings
            .iter()
            .filter(|w| w.message.contains("weak integrity"))
            .collect();
        assert!(!weak_hash.is_empty(), "should warn about SHA-1 hashes");
    }

    #[test]
    fn test_scoped_packages() {
        assert_eq!(
            extract_package_name("node_modules/@tanstack/react-query"),
            "@tanstack/react-query"
        );
        assert_eq!(
            extract_package_name("node_modules/@babel/core"),
            "@babel/core"
        );
        assert_eq!(extract_package_name("node_modules/lodash"), "lodash");
    }

    #[test]
    fn test_clean_lockfile() {
        let content = std::fs::read_to_string("../../tests/fixtures/package-lock-clean.json")
            .expect("read fixture");
        let parsed = parse_package_lock(&content).expect("parse");

        assert_eq!(parsed.packages.len(), 3);
        assert!(parsed.risky_specs.is_empty());
        assert!(
            parsed.warnings.is_empty(),
            "clean lockfile should have no warnings, got: {:?}",
            parsed.warnings
        );
    }

    #[test]
    fn test_parse_snapshot() {
        let content = std::fs::read_to_string("../../tests/fixtures/package-lock-clean.json")
            .expect("read fixture");
        let parsed = parse_package_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }
}
