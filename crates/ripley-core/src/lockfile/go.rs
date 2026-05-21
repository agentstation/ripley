use super::{InstalledPackage, LockfileError, ParsedLockfile, RiskySpec};
use crate::types::Ecosystem;

pub fn parse_go_sum(content: &str) -> Result<ParsedLockfile, LockfileError> {
    let mut packages = Vec::new();
    let warnings = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            continue;
        }

        let module = parts[0];
        let version_raw = parts[1];

        let version_str = version_raw.strip_suffix("/go.mod").unwrap_or(version_raw);

        let key = format!("{module}@{version_str}");
        if !seen.insert(key) {
            continue;
        }

        let semver_str = version_str.strip_prefix('v').unwrap_or(version_str);

        let version = match semver::Version::parse(semver_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        packages.push(InstalledPackage {
            name: module.to_string(),
            version,
            ecosystem: Ecosystem::Go,
        });
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedLockfile {
        packages,
        risky_specs: Vec::new(),
        warnings,
    })
}

pub fn parse_go_mod_replace(content: &str) -> Vec<RiskySpec> {
    let mut risky = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("replace ")
            && let Some((original, replacement)) = rest.split_once("=>")
        {
            let replacement = replacement.trim();
            if replacement.starts_with('.')
                || replacement.starts_with('/')
                || replacement.starts_with("..")
            {
                risky.push(RiskySpec {
                    package: original.trim().to_string(),
                    specifier: replacement.to_string(),
                    reason: "replace directive points to local path — not reproducible".to_string(),
                });
            }
        }
    }

    risky
}

pub fn parse_go_sum_with_mod(
    sum_content: &str,
    mod_content: Option<&str>,
) -> Result<ParsedLockfile, LockfileError> {
    let mut parsed = parse_go_sum(sum_content)?;

    if let Some(mod_content) = mod_content {
        let replace_risks = parse_go_mod_replace(mod_content);
        parsed.risky_specs.extend(replace_risks);
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_go_sum() {
        let content = std::fs::read_to_string("../../tests/fixtures/go.sum").expect("read fixture");
        let parsed = parse_go_sum(&content).expect("parse");

        assert!(
            parsed.packages.len() >= 10,
            "expected at least 10 packages, got {}",
            parsed.packages.len()
        );

        let gin = parsed
            .packages
            .iter()
            .find(|p| p.name == "github.com/gin-gonic/gin")
            .expect("gin should be present");
        assert_eq!(gin.version, semver::Version::new(1, 9, 1));
        assert_eq!(gin.ecosystem, Ecosystem::Go);

        let crypto = parsed
            .packages
            .iter()
            .find(|p| p.name == "golang.org/x/crypto");
        assert!(crypto.is_some(), "stdlib extensions should be present");
    }

    #[test]
    fn test_risky_replace() {
        let mod_content =
            std::fs::read_to_string("../../tests/fixtures/go.mod").expect("read fixture");
        let sum_content =
            std::fs::read_to_string("../../tests/fixtures/go.sum").expect("read fixture");

        let parsed = parse_go_sum_with_mod(&sum_content, Some(&mod_content)).expect("parse");

        let local_risks: Vec<_> = parsed
            .risky_specs
            .iter()
            .filter(|r| r.reason.contains("local path"))
            .collect();
        assert!(
            !local_risks.is_empty(),
            "should flag local path replace directives"
        );
    }

    #[test]
    fn test_deduplicates_go_mod_entries() {
        let content = std::fs::read_to_string("../../tests/fixtures/go.sum").expect("read fixture");
        let parsed = parse_go_sum(&content).expect("parse");

        let gin_count = parsed
            .packages
            .iter()
            .filter(|p| p.name == "github.com/gin-gonic/gin")
            .count();
        assert_eq!(gin_count, 1, "should deduplicate h1: and go.mod entries");
    }

    #[test]
    fn test_parse_snapshot() {
        let content = std::fs::read_to_string("../../tests/fixtures/go.sum").expect("read fixture");
        let parsed = parse_go_sum(&content).expect("parse");
        insta::assert_json_snapshot!(parsed);
    }
}
