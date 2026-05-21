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

pub fn generate_audit_prompt(report: &crate::audit::AuditReport) -> String {
    let mut prompt = String::from(
        "Review the following environment security audit findings and generate \
         environment-specific fix commands for each issue. Prioritize by actual \
         risk — consider the combination of findings (e.g., disabled disk \
         encryption + exposed tokens is critical).\n\n",
    );

    for cat in &report.categories {
        prompt.push_str(&format!("## {} [{}]\n\n", cat.category, cat.overall));
        for finding in &cat.findings {
            prompt.push_str(&format!(
                "- [{}] {}: {}\n",
                finding.status, finding.name, finding.detail
            ));
            if let Some(fix) = &finding.fix_command {
                prompt.push_str(&format!("  Suggested fix: {fix}\n"));
            }
        }
        prompt.push('\n');
    }

    prompt.push_str(
        "For each finding, generate the exact command to run on this machine. \
         If a finding is already green, skip it. Group related fixes together \
         and explain why each matters in the context of supply chain defense.\n",
    );

    prompt
}

pub fn generate_harden_prompt(report: &crate::harden::HardenReport) -> String {
    let mut prompt = String::from(
        "Review the following package manager hardening findings and generate \
         fix commands for each issue. These settings protect against supply chain \
         attacks by controlling how packages are installed and verified.\n\n",
    );

    if !report.detected_pms.is_empty() {
        prompt.push_str("Detected package managers: ");
        let pms: Vec<String> = report
            .detected_pms
            .iter()
            .map(|pm| {
                let ver = pm.version.as_deref().unwrap_or("unknown");
                format!("{} v{ver}", pm.name)
            })
            .collect();
        prompt.push_str(&pms.join(", "));
        prompt.push_str("\n\n");
    }

    for cat in &report.categories {
        prompt.push_str(&format!("## {} [{}]\n\n", cat.category, cat.overall));
        for finding in &cat.findings {
            prompt.push_str(&format!(
                "- [{}] {}: {}\n",
                finding.status, finding.name, finding.detail
            ));
            if let Some(fix) = &finding.fix_command {
                prompt.push_str(&format!("  Suggested fix: {fix}\n"));
            }
        }
        prompt.push('\n');
    }

    prompt.push_str(
        "For each non-green finding, generate the exact command to run. \
         Explain which supply chain attack each setting mitigates.\n",
    );

    prompt
}

pub fn generate_cve_fix_prompt(
    cve_id: &str,
    matches: &[crate::matcher::Match],
    ioc_profiles: Option<&[crate::forensic::ioc::IocProfile]>,
) -> String {
    let mut prompt = format!(
        "Remediate {cve_id} across all affected projects. This CVE has been \
         matched against installed packages — update each to a clean version, \
         verify the build, and check for indicators of compromise.\n\n"
    );

    prompt.push_str(&format!("## Affected packages ({cve_id})\n\n"));

    for m in matches {
        let fixed_version = m
            .advisory
            .affected_ranges
            .iter()
            .filter_map(|r| r.fixed.as_ref())
            .max()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "latest clean version".to_string());

        prompt.push_str(&format!(
            "- **{}@{}** in `{}`\n  Update to: {fixed_version}\n",
            m.advisory.package,
            m.package.version,
            m.project_path.display(),
        ));
    }

    prompt.push_str("\n## Steps\n\n");
    prompt.push_str("1. Update each package to the clean version listed above.\n");
    prompt.push_str("2. Run the test suite in each project and verify the build passes.\n");

    if let Some(profiles) = ioc_profiles {
        let relevant: Vec<&crate::forensic::ioc::IocProfile> = profiles
            .iter()
            .filter(|p| p.references.iter().any(|r| r.contains(cve_id)))
            .collect();

        if !relevant.is_empty() {
            prompt.push_str("3. Check for these IOC files and remove if present:\n");
            for profile in &relevant {
                for file in &profile.indicators.files {
                    prompt.push_str(&format!("   - {file}\n"));
                }
                for path in &profile.indicators.persistence_paths {
                    prompt.push_str(&format!("   - {path}\n"));
                }
            }
            prompt.push_str("4. Check credential stores for exfiltration:\n");
            for profile in &relevant {
                for target in &profile.credentials.targeted {
                    prompt.push_str(&format!("   - {target}\n"));
                }
            }
            prompt.push_str("5. Report what you found and what you changed.\n");
        } else {
            prompt.push_str("3. Check for common IOC files:\n");
            for path in default_ioc_paths() {
                prompt.push_str(&format!("   - {path}\n"));
            }
            prompt.push_str("4. Check credentials for exposure (.npmrc, .ssh/, .env)\n");
            prompt.push_str("5. Report what you found and what you changed.\n");
        }
    } else {
        prompt.push_str("3. Check for common IOC files:\n");
        for path in default_ioc_paths() {
            prompt.push_str(&format!("   - {path}\n"));
        }
        prompt.push_str("4. Check credentials for exposure (.npmrc, .ssh/, .env)\n");
        prompt.push_str("5. Report what you found and what you changed.\n");
    }

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
    use crate::feed::{Advisory, AffectedRange, FeedSource};
    use crate::lockfile::InstalledPackage;
    use crate::types::{Ecosystem, Severity};
    use std::path::PathBuf;

    fn test_match() -> Match {
        Match {
            advisory: Advisory {
                id: "GHSA-xxxx-yyyy-zzzz".into(),
                source: FeedSource::Osv,
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

    #[test]
    fn test_generate_cve_fix_prompt_single_project() {
        let m = test_match();
        let prompt = generate_cve_fix_prompt("CVE-2024-001", &[m], None);
        assert!(prompt.contains("CVE-2024-001"));
        assert!(prompt.contains("@tanstack/react-router"));
        assert!(prompt.contains("1.170.0"));
        assert!(prompt.contains("/Users/jack/src/myapp"));
        assert!(prompt.contains("test suite"));
    }

    #[test]
    fn test_generate_cve_fix_prompt_multi_project() {
        let m1 = test_match();
        let mut m2 = test_match();
        m2.project_path = PathBuf::from("/Users/jack/src/other-app");
        let prompt = generate_cve_fix_prompt("CVE-2024-001", &[m1, m2], None);
        assert!(prompt.contains("/Users/jack/src/myapp"));
        assert!(prompt.contains("/Users/jack/src/other-app"));
    }

    #[test]
    fn test_generate_cve_fix_prompt_with_iocs() {
        use crate::forensic::ioc::{IocCredentials, IocIndicators, IocProfile};
        let m = test_match();
        let profile = IocProfile {
            id: "test-profile".into(),
            name: "Test Attack".into(),
            description: "test".into(),
            date: "2026-01-01".into(),
            references: vec!["https://example.com/CVE-2024-001".into()],
            packages: None,
            indicators: IocIndicators {
                files: vec![".malware/payload.js".into()],
                persistence_paths: vec!["~/.config/evil/".into()],
                ..Default::default()
            },
            credentials: IocCredentials {
                targeted: vec!["~/.npmrc".into(), "~/.ssh/id_rsa".into()],
                ..Default::default()
            },
        };
        let prompt = generate_cve_fix_prompt("CVE-2024-001", &[m], Some(&[profile]));
        assert!(prompt.contains(".malware/payload.js"));
        assert!(prompt.contains("~/.npmrc"));
        assert!(prompt.contains("~/.ssh/id_rsa"));
    }
}
