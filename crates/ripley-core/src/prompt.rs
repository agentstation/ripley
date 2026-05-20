use crate::matcher::Match;

pub fn generate_remediation_prompt(m: &Match) -> String {
    let fixed_version = m
        .advisory
        .affected_ranges
        .iter()
        .filter_map(|r| r.fixed.as_ref())
        .max()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "the latest clean version".to_string());

    let mut prompt = format!(
        "In the project at {project}, the package {pkg} \
         is installed at version {version}, which is compromised ({id}).\n\n",
        project = m.project_path.display(),
        pkg = m.advisory.package,
        version = m.package.version,
        id = m.advisory.id,
    );

    prompt.push_str(&format!("1. Update to {fixed_version}.\n"));

    prompt.push_str("2. Check for these IOC files and remove them if present:\n");
    let ioc_paths = default_ioc_paths();
    for path in &ioc_paths {
        prompt.push_str(&format!("   - {path}\n"));
    }

    prompt.push_str("3. Check for credential exposure:\n");
    prompt.push_str("   - Inspect .npmrc for leaked tokens\n");
    prompt.push_str("   - Check ~/.ssh/ for unauthorized keys\n");
    prompt.push_str("   - Check .env files for exposed secrets\n");

    prompt.push_str("4. Run the test suite and verify the build passes.\n");
    prompt.push_str("5. Report what you found and what you changed.\n");

    prompt
}

pub fn generate_batch_prompt(matches: &[Match]) -> String {
    if matches.len() == 1 {
        return generate_remediation_prompt(&matches[0]);
    }

    let mut prompt = format!(
        "The following {} packages have known vulnerabilities:\n\n",
        matches.len()
    );

    for m in matches {
        prompt.push_str(&format!(
            "- {}@{} ({}) in {}\n",
            m.advisory.package,
            m.package.version,
            m.advisory.id,
            m.project_path.display()
        ));
    }

    prompt.push_str("\nFor each:\n");
    prompt.push_str("1. Update to the latest clean version.\n");
    prompt.push_str("2. Check for IOC files and remove them if present.\n");
    prompt.push_str("3. Run the test suite and verify the build passes.\n");
    prompt.push_str("4. Report what you found and what you changed.\n");

    prompt
}

fn default_ioc_paths() -> Vec<&'static str> {
    vec![
        ".claude/execution.js",
        ".claude/setup.mjs",
        ".claude/settings.json",
        ".vscode/tasks.json",
        ".mcp.json",
        ".cursor/mcp.json",
        "node_modules/.cache/ (unexpected executables)",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{Advisory, AffectedRange};
    use crate::lockfile::InstalledPackage;
    use crate::types::{Ecosystem, Severity};
    use std::path::PathBuf;

    fn test_match() -> Match {
        Match {
            advisory: Advisory {
                id: "GHSA-xxxx-yyyy-zzzz".into(),
                ecosystem: Ecosystem::Npm,
                package: "@tanstack/react-router".into(),
                affected_ranges: vec![AffectedRange {
                    introduced: "1.0.0".parse().unwrap(),
                    fixed: Some("1.170.0".parse().unwrap()),
                }],
                severity: Some(Severity::Critical),
                summary: "Prototype Pollution leading to RCE".into(),
                references: vec![],
            },
            package: InstalledPackage {
                name: "@tanstack/react-router".into(),
                version: "1.169.5".parse().unwrap(),
                ecosystem: Ecosystem::Npm,
            },
            project_path: PathBuf::from("/Users/jack/src/myapp"),
        }
    }

    #[test]
    fn test_generate_remediation_prompt() {
        let m = test_match();
        let prompt = generate_remediation_prompt(&m);

        assert!(prompt.contains("/Users/jack/src/myapp"));
        assert!(prompt.contains("@tanstack/react-router"));
        assert!(prompt.contains("1.169.5"));
        assert!(prompt.contains("GHSA-xxxx-yyyy-zzzz"));
        assert!(prompt.contains("1.170.0"));
        assert!(prompt.contains(".claude/execution.js"));
        assert!(prompt.contains("test suite"));
    }

    #[test]
    fn test_generate_remediation_prompt_snapshot() {
        let m = test_match();
        let prompt = generate_remediation_prompt(&m);
        insta::assert_snapshot!(prompt);
    }

    #[test]
    fn test_batch_prompt() {
        let m = test_match();
        let prompt = generate_batch_prompt(&[m.clone(), m]);
        assert!(prompt.contains("2 packages"));
    }

    #[test]
    fn test_single_match_batch_delegates() {
        let m = test_match();
        let single = generate_remediation_prompt(&m);
        let batch = generate_batch_prompt(&[m]);
        assert_eq!(single, batch);
    }
}
