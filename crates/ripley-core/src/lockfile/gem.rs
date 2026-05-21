use super::{InstalledPackage, LockfileError, ParsedLockfile, RiskySpec};
use crate::types::Ecosystem;

pub fn parse_gemfile_lock(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let mut packages = Vec::new();
    let mut risky_specs = Vec::new();
    let warnings = Vec::new();

    let mut current_section = Section::None;
    let mut in_specs = false;
    let mut current_git_remote: Option<String> = None;
    let mut current_path_remote: Option<String> = None;

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') {
            let section_name = line.trim();
            match section_name {
                "GIT" => {
                    current_section = Section::Git;
                    in_specs = false;
                    current_git_remote = None;
                }
                "PATH" => {
                    current_section = Section::Path;
                    in_specs = false;
                    current_path_remote = None;
                }
                "GEM" => {
                    current_section = Section::Gem;
                    in_specs = false;
                }
                "PLATFORMS" | "DEPENDENCIES" | "BUNDLED WITH" => {
                    current_section = Section::Other;
                    in_specs = false;
                }
                _ => {}
            }
            continue;
        }

        let trimmed = line.trim();

        match current_section {
            Section::Git => {
                if let Some(remote) = trimmed.strip_prefix("remote: ") {
                    current_git_remote = Some(remote.to_string());
                } else if trimmed == "specs:" {
                    in_specs = true;
                } else if in_specs && let Some((name, version)) = parse_spec_line(trimmed) {
                    risky_specs.push(RiskySpec {
                        package: name.clone(),
                        specifier: current_git_remote
                            .clone()
                            .unwrap_or_else(|| "git".to_string()),
                        reason: "resolves from git source — bypasses registry integrity"
                            .to_string(),
                    });

                    if let Some(v) = version
                        && let Ok(ver) = semver::Version::parse(&v)
                    {
                        packages.push(InstalledPackage {
                            name,
                            version: ver,
                            ecosystem: Ecosystem::Gem,
                        });
                    }
                }
            }
            Section::Path => {
                if let Some(remote) = trimmed.strip_prefix("remote: ") {
                    current_path_remote = Some(remote.to_string());
                } else if trimmed == "specs:" {
                    in_specs = true;
                } else if in_specs && let Some((name, version)) = parse_spec_line(trimmed) {
                    risky_specs.push(RiskySpec {
                        package: name.clone(),
                        specifier: current_path_remote
                            .clone()
                            .unwrap_or_else(|| "path".to_string()),
                        reason: "resolves from local path — not reproducible".to_string(),
                    });

                    if let Some(v) = version
                        && let Ok(ver) = semver::Version::parse(&v)
                    {
                        packages.push(InstalledPackage {
                            name,
                            version: ver,
                            ecosystem: Ecosystem::Gem,
                        });
                    }
                }
            }
            Section::Gem => {
                if trimmed == "specs:" {
                    in_specs = true;
                } else if in_specs
                    && let Some((name, Some(v))) = parse_spec_line(trimmed)
                    && let Ok(ver) = semver::Version::parse(&v)
                {
                    packages.push(InstalledPackage {
                        name,
                        version: ver,
                        ecosystem: Ecosystem::Gem,
                    });
                }
            }
            _ => {}
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages,
        risky_specs,
        warnings,
    })
}

#[derive(Clone, Copy)]
enum Section {
    None,
    Git,
    Path,
    Gem,
    Other,
}

fn parse_spec_line(line: &str) -> Option<(String, Option<String>)> {
    let line = line.trim();

    if line.starts_with(' ') || line.starts_with('\t') {
        return None;
    }

    if let Some(paren_pos) = line.find('(') {
        let name = line[..paren_pos].trim().to_string();
        let version = line[paren_pos + 1..]
            .trim_end_matches(')')
            .trim()
            .to_string();
        Some((name, Some(version)))
    } else {
        Some((line.to_string(), None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Gemfile.lock").expect("read fixture");
        let parsed = parse_gemfile_lock(&content).expect("parse");

        assert!(
            parsed.packages.len() >= 15,
            "expected at least 15 packages, got {}",
            parsed.packages.len()
        );

        let rails = parsed
            .packages
            .iter()
            .find(|p| p.name == "rails")
            .expect("rails should be present");
        assert_eq!(rails.version, semver::Version::new(7, 1, 3));
        assert_eq!(rails.ecosystem, Ecosystem::Gem);

        let puma = parsed.packages.iter().find(|p| p.name == "puma");
        assert!(puma.is_some(), "puma should be present");
    }

    #[test]
    fn test_risky_sources() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Gemfile.lock").expect("read fixture");
        let parsed = parse_gemfile_lock(&content).expect("parse");

        let git_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("git"))
            .collect();
        assert!(!git_risks.is_empty(), "should flag GIT source gems");

        let path_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("path"))
            .collect();
        assert!(!path_risks.is_empty(), "should flag PATH source gems");
    }

    #[test]
    fn test_parse_snapshot() {
        let content =
            std::fs::read_to_string("../../tests/fixtures/Gemfile.lock").expect("read fixture");
        let parsed = parse_gemfile_lock(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }
}
