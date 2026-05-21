use std::collections::BTreeMap;

use serde::Deserialize;

use super::{InstalledPackage, LockfileError, LockfileWarning, ParsedLockfile, RiskySpec};
use crate::types::{Ecosystem, Severity};

pub fn parse_pipfile_lock(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let lock: PipfileLock =
        serde_json::from_str(content).map_err(|e| LockfileError::Parse(e.to_string()))?;

    let mut packages = Vec::new();
    let mut risky_specs = Vec::new();
    let mut warnings = Vec::new();

    for section in [&lock.default, &lock.develop] {
        for (name, entry) in section {
            process_pipfile_entry(name, entry, &mut packages, &mut risky_specs, &mut warnings);
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages,
        risky_specs,
        warnings,
    })
}

pub fn parse_poetry_lock(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let lock: PoetryLock =
        toml::from_str(content).map_err(|e| LockfileError::Parse(e.to_string()))?;

    let mut packages = Vec::new();
    let mut risky_specs = Vec::new();
    let warnings = Vec::new();

    for pkg in &lock.package {
        let version = match semver::Version::parse(&pkg.version) {
            Ok(v) => v,
            Err(_) => continue,
        };

        packages.push(InstalledPackage {
            name: pkg.name.clone(),
            version,
            ecosystem: Ecosystem::PyPI,
        });

        if let Some(source) = &pkg.source {
            if source.source_type == "git" {
                risky_specs.push(RiskySpec {
                    package: pkg.name.clone(),
                    specifier: source.url.clone().unwrap_or_default(),
                    reason: "resolves from git source — bypasses registry integrity".to_string(),
                });
            }
            if source.source_type == "file" || source.source_type == "directory" {
                risky_specs.push(RiskySpec {
                    package: pkg.name.clone(),
                    specifier: source.url.clone().unwrap_or_default(),
                    reason: "resolves from local source — not reproducible".to_string(),
                });
            }
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages,
        risky_specs,
        warnings,
    })
}

#[derive(Deserialize)]
struct PipfileLock {
    #[serde(default)]
    default: BTreeMap<String, PipfileEntry>,
    #[serde(default)]
    develop: BTreeMap<String, PipfileEntry>,
}

#[derive(Deserialize)]
struct PipfileEntry {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    hashes: Option<Vec<String>>,
    #[serde(default)]
    git: Option<String>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    editable: Option<bool>,
}

fn process_pipfile_entry(
    name: &str,
    entry: &PipfileEntry,
    packages: &mut Vec<InstalledPackage>,
    risky_specs: &mut Vec<RiskySpec>,
    warnings: &mut Vec<LockfileWarning>,
) {
    if let Some(git_url) = &entry.git {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: git_url.clone(),
            reason: "resolves from git source — bypasses registry integrity".to_string(),
        });
    }

    if let Some(file_url) = &entry.file {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: file_url.clone(),
            reason: "resolves from local file — not reproducible".to_string(),
        });
    }

    if entry.editable == Some(true) {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: "editable = true".to_string(),
            reason: "editable install — not pinned to a specific version".to_string(),
        });
    }

    let version_str = match &entry.version {
        Some(v) => v.trim_start_matches("=="),
        None => return,
    };

    let version = match semver::Version::parse(version_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    packages.push(InstalledPackage {
        name: name.to_string(),
        version,
        ecosystem: Ecosystem::PyPI,
    });

    if entry.hashes.as_ref().is_none_or(|h| h.is_empty()) && entry.git.is_none() {
        warnings.push(LockfileWarning {
            package: name.to_string(),
            field: "hashes".to_string(),
            message: "missing integrity hashes".to_string(),
            severity: Severity::Medium,
        });
    }
}

#[derive(Deserialize)]
struct PoetryLock {
    #[serde(default)]
    package: Vec<PoetryPackage>,
}

#[derive(Deserialize)]
struct PoetryPackage {
    name: String,
    version: String,
    #[serde(default)]
    source: Option<PoetrySource>,
}

#[derive(Deserialize)]
struct PoetrySource {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(default)]
    url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pipfile_lock() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Pipfile.lock").expect("read fixture");
        let parsed = parse_pipfile_lock(&content).expect("parse");

        assert!(
            parsed.packages.len() >= 15,
            "expected at least 15 packages, got {}",
            parsed.packages.len()
        );

        let flask = parsed
            .packages
            .iter()
            .find(|p| p.name == "flask")
            .expect("flask should be present");
        assert_eq!(flask.version, semver::Version::new(3, 0, 3));
        assert_eq!(flask.ecosystem, Ecosystem::PyPI);

        let pytest = parsed.packages.iter().find(|p| p.name == "pytest");
        assert!(pytest.is_some(), "dev dependency should be included");
    }

    #[test]
    fn test_parse_poetry_lock() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/poetry.lock").expect("read fixture");
        let parsed = parse_poetry_lock(&content).expect("parse");

        assert!(
            parsed.packages.len() >= 15,
            "expected at least 15 packages, got {}",
            parsed.packages.len()
        );

        let flask = parsed
            .packages
            .iter()
            .find(|p| p.name == "flask")
            .expect("flask should be present");
        assert_eq!(flask.version, semver::Version::new(3, 0, 3));
        assert_eq!(flask.ecosystem, Ecosystem::PyPI);
    }

    #[test]
    fn test_risky_specs() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Pipfile.lock").expect("read fixture");
        let parsed = parse_pipfile_lock(&content).expect("parse");

        let git_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("git"))
            .collect();
        assert!(!git_risks.is_empty(), "should flag git sources");

        let file_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("file"))
            .collect();
        assert!(!file_risks.is_empty(), "should flag file: sources");

        let editable_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("editable"))
            .collect();
        assert!(!editable_risks.is_empty(), "should flag editable installs");

        let poetry_content =
            std::fs::read_to_string("../../tests/fixtures/poetry.lock").expect("read fixture");
        let poetry_parsed = parse_poetry_lock(&poetry_content).expect("parse");

        let poetry_git: Vec<_> = poetry_parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("git"))
            .collect();
        assert!(!poetry_git.is_empty(), "poetry should flag git sources too");
    }

    #[test]
    fn test_pipfile_snapshot() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Pipfile.lock").expect("read fixture");
        let parsed = parse_pipfile_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }

    #[test]
    fn test_poetry_snapshot() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/poetry.lock").expect("read fixture");
        let parsed = parse_poetry_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }
}
