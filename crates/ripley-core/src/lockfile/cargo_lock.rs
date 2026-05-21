use serde::Deserialize;

use super::{InstalledPackage, LockfileError, LockfileWarning, ParsedLockfile, RiskySpec};
use crate::types::{Ecosystem, Severity};

#[derive(Deserialize)]
struct CargoLock {
    #[serde(default)]
    package: Vec<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    checksum: Option<String>,
}

pub fn parse_cargo_lock(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let lock: CargoLock =
        toml::from_str(content).map_err(|e| LockfileError::Parse(e.to_string()))?;

    let mut packages = Vec::new();
    let mut risky_specs = Vec::new();
    let mut warnings = Vec::new();

    for pkg in &lock.package {
        let version = match semver::Version::parse(&pkg.version) {
            Ok(v) => v,
            Err(_) => continue,
        };

        packages.push(InstalledPackage {
            name: pkg.name.clone(),
            version,
            ecosystem: Ecosystem::Cargo,
        });

        if let Some(source) = &pkg.source {
            check_source(&pkg.name, source, &mut risky_specs);
        }

        check_checksum(&pkg.name, &pkg.source, &pkg.checksum, &mut warnings);
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages,
        risky_specs,
        warnings,
    })
}

fn check_source(name: &str, source: &str, risky_specs: &mut Vec<RiskySpec>) {
    if source.starts_with("git+") {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: source.to_string(),
            reason: "resolves from git source — bypasses registry integrity".to_string(),
        });
    }

    if source.starts_with("path+") {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: source.to_string(),
            reason: "resolves from local path — not reproducible".to_string(),
        });
    }

    if source.starts_with("registry+")
        && !source.starts_with("registry+https://github.com/rust-lang/crates.io-index")
    {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: source.to_string(),
            reason: "resolves from non-standard registry".to_string(),
        });
    }
}

fn check_checksum(
    name: &str,
    source: &Option<String>,
    checksum: &Option<String>,
    warnings: &mut Vec<LockfileWarning>,
) {
    let is_registry = source.as_ref().is_some_and(|s| s.starts_with("registry+"));

    if is_registry && checksum.is_none() {
        warnings.push(LockfileWarning {
            package: name.to_string(),
            field: "checksum".to_string(),
            message: "missing checksum for registry package".to_string(),
            severity: Severity::Medium,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Cargo.lock").expect("read fixture");
        let parsed = parse_cargo_lock(&content).expect("parse");

        assert!(
            parsed.packages.len() >= 15,
            "expected at least 15 packages, got {}",
            parsed.packages.len()
        );

        let serde = parsed
            .packages
            .iter()
            .find(|p| p.name == "serde")
            .expect("serde should be present");
        assert_eq!(serde.ecosystem, Ecosystem::Cargo);

        let tokio = parsed.packages.iter().find(|p| p.name == "tokio");
        assert!(tokio.is_some(), "tokio should be present");
    }

    #[test]
    fn test_risky_sources() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Cargo.lock").expect("read fixture");
        let parsed = parse_cargo_lock(&content).expect("parse");

        let git_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("git"))
            .collect();
        assert!(!git_risks.is_empty(), "should flag git+ sources");

        let path_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("path"))
            .collect();
        assert!(!path_risks.is_empty(), "should flag path+ sources");

        let registry_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("non-standard registry"))
            .collect();
        assert!(
            !registry_risks.is_empty(),
            "should flag non-crates.io registries"
        );
    }

    #[test]
    fn test_parse_snapshot() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Cargo.lock").expect("read fixture");
        let parsed = parse_cargo_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }
}
