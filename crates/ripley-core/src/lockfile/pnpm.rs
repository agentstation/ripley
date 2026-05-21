use std::collections::BTreeMap;

use serde::Deserialize;

use super::{InstalledPackage, LockfileError, LockfileWarning, ParsedLockfile, RiskySpec};
use crate::types::{Ecosystem, Severity};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PnpmLock {
    lockfile_version: serde_yaml::Value,
    #[serde(default)]
    packages: Option<BTreeMap<String, PnpmPackageEntry>>,
    #[serde(default)]
    snapshots: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Deserialize, Default)]
struct PnpmPackageEntry {
    #[serde(default)]
    resolution: Option<PnpmResolution>,
    #[serde(default)]
    version: Option<String>,
}

#[derive(Deserialize, Default)]
struct PnpmResolution {
    #[serde(default)]
    integrity: Option<String>,
    #[serde(default)]
    tarball: Option<String>,
    #[serde(default)]
    directory: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    resolution_type: Option<String>,
}

pub fn parse_pnpm_lock(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let lock: PnpmLock =
        serde_yaml::from_str(content).map_err(|e| LockfileError::Parse(e.to_string()))?;

    let is_v9 = match &lock.lockfile_version {
        serde_yaml::Value::String(s) => s.starts_with('9'),
        serde_yaml::Value::Number(n) => n.as_f64().is_some_and(|v| v >= 9.0),
        _ => false,
    };

    let packages = match &lock.packages {
        Some(p) => p,
        None => {
            return Ok(ParsedLockfile {
                packages: Vec::new(),
                risky_specs: Vec::new(),
                warnings: Vec::new(),
            });
        }
    };

    let mut parsed_packages = Vec::new();
    let mut risky_specs = Vec::new();
    let mut warnings = Vec::new();

    for (key, entry) in packages {
        let (name, version_str) = if is_v9 {
            parse_v9_key(key)
        } else {
            parse_v6_key(key)
        };

        let name = match name {
            Some(n) => n,
            None => continue,
        };

        let version_str = match version_str.or_else(|| entry.version.clone()) {
            Some(v) => v,
            None => continue,
        };

        let version = match semver::Version::parse(&version_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        parsed_packages.push(InstalledPackage {
            name: name.clone(),
            version,
            ecosystem: Ecosystem::Npm,
        });

        if let Some(resolution) = &entry.resolution {
            check_resolution(&name, resolution, &mut risky_specs, &mut warnings);
        }
    }

    if is_v9 && let Some(snapshots) = &lock.snapshots {
        for key in snapshots.keys() {
            let (name, version_str) = parse_v9_key(key);
            let name = match name {
                Some(n) => n,
                None => continue,
            };
            let version_str = match version_str {
                Some(v) => v,
                None => continue,
            };
            let version = match semver::Version::parse(&version_str) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if !parsed_packages
                .iter()
                .any(|p| p.name == name && p.version == version)
            {
                parsed_packages.push(InstalledPackage {
                    name,
                    version,
                    ecosystem: Ecosystem::Npm,
                });
            }
        }
    }

    parsed_packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages: parsed_packages,
        risky_specs,
        warnings,
    })
}

fn parse_v6_key(key: &str) -> (Option<String>, Option<String>) {
    parse_package_key(key, true)
}

fn parse_v9_key(key: &str) -> (Option<String>, Option<String>) {
    parse_package_key(key, false)
}

fn parse_package_key(key: &str, strip_leading_slash: bool) -> (Option<String>, Option<String>) {
    let key = if strip_leading_slash {
        key.strip_prefix('/').unwrap_or(key)
    } else {
        key
    };

    let (name, rest) = if let Some(after_at) = key.strip_prefix('@') {
        let after_scope = match after_at.find('/') {
            Some(pos) => pos + 1,
            None => return (None, None),
        };
        let after_name = match key[after_scope + 1..].find('@') {
            Some(pos) => after_scope + 1 + pos,
            None => {
                let name = key.split('(').next().unwrap_or(key);
                return (Some(name.to_string()), None);
            }
        };
        (&key[..after_name], &key[after_name + 1..])
    } else {
        match key.find('@') {
            Some(pos) if pos > 0 => (&key[..pos], &key[pos + 1..]),
            _ => {
                let name = key.split('(').next().unwrap_or(key);
                return (Some(name.to_string()), None);
            }
        }
    };

    let version = rest.split('(').next().unwrap_or(rest);
    (Some(name.to_string()), Some(version.to_string()))
}

fn check_resolution(
    name: &str,
    resolution: &PnpmResolution,
    risky_specs: &mut Vec<RiskySpec>,
    warnings: &mut Vec<LockfileWarning>,
) {
    if let Some(tarball) = &resolution.tarball {
        if tarball.starts_with("git+") || tarball.starts_with("git://") {
            risky_specs.push(RiskySpec {
                package: name.to_string(),
                specifier: tarball.clone(),
                reason: "resolves from git source — bypasses registry integrity".to_string(),
            });
        } else if !tarball.starts_with("https://registry.npmjs.org")
            && !tarball.starts_with("https://registry.yarnpkg.com")
        {
            risky_specs.push(RiskySpec {
                package: name.to_string(),
                specifier: tarball.clone(),
                reason: "resolves from non-standard tarball URL".to_string(),
            });
        }
    }

    if resolution.directory.is_some() || resolution.resolution_type.as_deref() == Some("directory")
    {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: resolution
                .directory
                .clone()
                .unwrap_or_else(|| "link:".to_string()),
            reason: "resolves from local directory — not reproducible".to_string(),
        });
    }

    if resolution.integrity.is_none()
        && resolution.tarball.is_none()
        && resolution.directory.is_none()
    {
        warnings.push(LockfileWarning {
            package: name.to_string(),
            field: "resolution.integrity".to_string(),
            message: "missing integrity hash".to_string(),
            severity: Severity::Medium,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_v6_fixture() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/pnpm-lock-v6.yaml").expect("read");
        let parsed = parse_pnpm_lock(&content).expect("parse");

        assert!(
            parsed.packages.len() >= 15,
            "expected at least 15 packages, got {}",
            parsed.packages.len()
        );

        let express = parsed
            .packages
            .iter()
            .find(|p| p.name == "express")
            .expect("express should be present");
        assert_eq!(express.version, semver::Version::new(4, 18, 2));
        assert_eq!(express.ecosystem, Ecosystem::Npm);

        let tanstack = parsed.packages.iter().find(|p| p.name.contains("tanstack"));
        assert!(tanstack.is_some(), "scoped package should be present");
    }

    #[test]
    fn test_parse_v9_fixture() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/pnpm-lock-v9.yaml").expect("read");
        let parsed = parse_pnpm_lock(&content).expect("parse");

        assert!(
            parsed.packages.len() >= 15,
            "expected at least 15 packages, got {}",
            parsed.packages.len()
        );

        let express = parsed
            .packages
            .iter()
            .find(|p| p.name == "express")
            .expect("express should be present");
        assert_eq!(express.version, semver::Version::new(4, 18, 2));
    }

    #[test]
    fn test_risky_specs() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/pnpm-lock-v6.yaml").expect("read");
        let parsed = parse_pnpm_lock(&content).expect("parse");

        let git_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("git"))
            .collect();
        assert!(!git_risks.is_empty(), "should flag git+ sources");

        let dir_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("directory"))
            .collect();
        assert!(!dir_risks.is_empty(), "should flag directory sources");
    }

    #[test]
    fn test_parse_v6_snapshot() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/pnpm-lock-v6.yaml").expect("read");
        let parsed = parse_pnpm_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }

    #[test]
    fn test_parse_v9_snapshot() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/pnpm-lock-v9.yaml").expect("read");
        let parsed = parse_pnpm_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }
}
