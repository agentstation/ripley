use super::{InstalledPackage, LockfileError, LockfileWarning, ParsedLockfile, RiskySpec};
use crate::types::{Ecosystem, Severity};

pub fn parse_yarn_lock(content: &str) -> Result<ParsedLockfile, LockfileError> {
    if content.contains("__metadata:") {
        parse_v2(content)
    } else {
        parse_v1(content)
    }
}

fn parse_v1(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let mut packages = Vec::new();
    let mut risky_specs = Vec::new();
    let mut warnings = Vec::new();

    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;
    let mut current_resolved: Option<String> = None;
    let mut current_integrity: Option<String> = None;

    for line in content.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(ref name) = current_name {
                flush_v1_entry(
                    name,
                    &current_version,
                    &current_resolved,
                    &current_integrity,
                    &mut packages,
                    &mut risky_specs,
                    &mut warnings,
                );
            }

            current_name = extract_v1_package_name(line);
            current_version = None;
            current_resolved = None;
            current_integrity = None;
            continue;
        }

        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("version ") {
            current_version = Some(unquote(rest));
        } else if let Some(rest) = trimmed.strip_prefix("resolved ") {
            current_resolved = Some(unquote(rest));
        } else if let Some(rest) = trimmed.strip_prefix("integrity ") {
            current_integrity = Some(unquote(rest));
        }
    }

    if let Some(ref name) = current_name {
        flush_v1_entry(
            name,
            &current_version,
            &current_resolved,
            &current_integrity,
            &mut packages,
            &mut risky_specs,
            &mut warnings,
        );
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages,
        risky_specs,
        warnings,
    })
}

fn extract_v1_package_name(header: &str) -> Option<String> {
    let header = header.trim_end_matches(':');
    let first_spec = header.split(',').next()?;
    let spec = unquote(first_spec.trim());

    if let Some(at_pos) = spec.rfind('@')
        && at_pos > 0
    {
        return Some(spec[..at_pos].to_string());
    }
    None
}

fn flush_v1_entry(
    name: &str,
    version: &Option<String>,
    resolved: &Option<String>,
    integrity: &Option<String>,
    packages: &mut Vec<InstalledPackage>,
    risky_specs: &mut Vec<RiskySpec>,
    warnings: &mut Vec<LockfileWarning>,
) {
    let version_str = match version {
        Some(v) => v,
        None => return,
    };

    let version = match semver::Version::parse(version_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    packages.push(InstalledPackage {
        name: name.to_string(),
        version,
        ecosystem: Ecosystem::Npm,
    });

    if let Some(resolved) = resolved {
        check_resolved(name, resolved, risky_specs, warnings);
    }

    check_integrity(name, integrity, warnings);
}

fn parse_v2(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let mut packages = Vec::new();
    let mut risky_specs = Vec::new();
    let mut warnings = Vec::new();

    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;
    let mut current_resolution: Option<String> = None;
    let mut current_checksum: Option<String> = None;

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') && line.ends_with(':') {
            if let Some(ref name) = current_name {
                flush_v2_entry(
                    name,
                    &current_version,
                    &current_resolution,
                    &current_checksum,
                    &mut packages,
                    &mut risky_specs,
                    &mut warnings,
                );
            }

            current_name = extract_v2_package_name(line);
            current_version = None;
            current_resolution = None;
            current_checksum = None;
            continue;
        }

        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("version: ") {
            current_version = Some(rest.to_string());
        } else if let Some(rest) = trimmed.strip_prefix("resolution: ") {
            current_resolution = Some(unquote(rest));
        } else if let Some(rest) = trimmed.strip_prefix("checksum: ") {
            current_checksum = Some(rest.to_string());
        }
    }

    if let Some(ref name) = current_name {
        flush_v2_entry(
            name,
            &current_version,
            &current_resolution,
            &current_checksum,
            &mut packages,
            &mut risky_specs,
            &mut warnings,
        );
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages,
        risky_specs,
        warnings,
    })
}

fn extract_v2_package_name(header: &str) -> Option<String> {
    let header = header.trim().trim_end_matches(':');
    let spec = unquote(header);

    if spec == "__metadata" {
        return None;
    }

    let without_protocol = spec.strip_prefix("npm:").unwrap_or(&spec);

    let without_protocol = if let Some(pkg_part) = without_protocol.strip_prefix("patch:") {
        let before_hash = pkg_part.split('#').next()?;
        before_hash.strip_prefix("npm:").unwrap_or(before_hash)
    } else {
        without_protocol
    };

    if let Some(at_pos) = without_protocol.rfind('@')
        && at_pos > 0
    {
        let name = &without_protocol[..at_pos];
        let name = name.strip_prefix("npm:").unwrap_or(name);
        return Some(name.to_string());
    }
    None
}

fn flush_v2_entry(
    name: &str,
    version: &Option<String>,
    resolution: &Option<String>,
    checksum: &Option<String>,
    packages: &mut Vec<InstalledPackage>,
    risky_specs: &mut Vec<RiskySpec>,
    warnings: &mut Vec<LockfileWarning>,
) {
    let version_str = match version {
        Some(v) => v,
        None => return,
    };

    let version = match semver::Version::parse(version_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    packages.push(InstalledPackage {
        name: name.to_string(),
        version,
        ecosystem: Ecosystem::Npm,
    });

    if let Some(resolution) = resolution {
        check_v2_resolution(name, resolution, &mut *risky_specs);
    }

    if checksum.is_none() {
        warnings.push(LockfileWarning {
            package: name.to_string(),
            field: "checksum".to_string(),
            message: "missing checksum".to_string(),
            severity: Severity::Medium,
        });
    }
}

fn check_resolved(
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

fn check_v2_resolution(name: &str, resolution: &str, risky_specs: &mut Vec<RiskySpec>) {
    if resolution.contains("patch:") || resolution.contains("#~/.yarn/patches/") {
        risky_specs.push(RiskySpec {
            package: name.to_string(),
            specifier: resolution.to_string(),
            reason: "uses patch protocol — modified from upstream".to_string(),
        });
    }
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_v1_fixture() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/yarn-v1.lock").expect("read fixture");
        let parsed = parse_yarn_lock(&content).expect("parse");

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

        let tanstack = parsed
            .packages
            .iter()
            .find(|p| p.name == "@tanstack/react-query");
        assert!(tanstack.is_some(), "scoped package should be present");
    }

    #[test]
    fn test_parse_v2_fixture() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/yarn-v2.lock").expect("read fixture");
        let parsed = parse_yarn_lock(&content).expect("parse");

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
    fn test_scoped_packages() {
        assert_eq!(
            extract_v1_package_name("\"@babel/core@^7.24.0\":"),
            Some("@babel/core".to_string())
        );
        assert_eq!(
            extract_v1_package_name("\"@tanstack/react-query@^5.32.0\":"),
            Some("@tanstack/react-query".to_string())
        );
        assert_eq!(
            extract_v1_package_name("lodash@^4.17.21:"),
            Some("lodash".to_string())
        );

        assert_eq!(
            extract_v2_package_name("\"@babel/core@npm:^7.24.0\":"),
            Some("@babel/core".to_string())
        );
        assert_eq!(
            extract_v2_package_name("\"express@npm:^4.18.0\":"),
            Some("express".to_string())
        );
    }

    #[test]
    fn test_risky_specs_v1() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/yarn-v1.lock").expect("read fixture");
        let parsed = parse_yarn_lock(&content).expect("parse");

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
    }

    #[test]
    fn test_risky_specs_v2() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/yarn-v2.lock").expect("read fixture");
        let parsed = parse_yarn_lock(&content).expect("parse");

        let patch_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("patch"))
            .collect();
        assert!(!patch_risks.is_empty(), "should flag patch: protocol");
    }

    #[test]
    fn test_parse_v1_snapshot() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/yarn-v1.lock").expect("read fixture");
        let parsed = parse_yarn_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }

    #[test]
    fn test_parse_v2_snapshot() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/yarn-v2.lock").expect("read fixture");
        let parsed = parse_yarn_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }
}
