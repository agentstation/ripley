use std::path::Path;

use crate::audit::TrafficLight;

use super::HardenFinding;

pub fn check_provenance(project_path: &Path) -> Vec<HardenFinding> {
    let mut findings = Vec::new();

    findings.push(check_trusted_publishing(project_path));

    findings
}

pub fn check_trusted_publishing(project_path: &Path) -> HardenFinding {
    let workflows_dir = project_path.join(".github/workflows");

    if !workflows_dir.exists() {
        return HardenFinding {
            name: "Trusted Publishing".to_string(),
            status: TrafficLight::Yellow,
            detail: "No GitHub workflows found — cannot check OIDC publishing".to_string(),
            fix_command: None,
            pm: String::new(),
        };
    }

    let has_provenance = std::fs::read_dir(&workflows_dir)
        .ok()
        .map(|entries| {
            entries.flatten().any(|entry| {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                content.contains("provenance") || content.contains("id-token: write")
            })
        })
        .unwrap_or(false);

    if has_provenance {
        HardenFinding {
            name: "Trusted Publishing".to_string(),
            status: TrafficLight::Green,
            detail: "OIDC/provenance configuration detected in CI workflows".to_string(),
            fix_command: None,
            pm: String::new(),
        }
    } else {
        HardenFinding {
            name: "Trusted Publishing".to_string(),
            status: TrafficLight::Yellow,
            detail: "No provenance/OIDC configuration found in workflows".to_string(),
            fix_command: Some(
                "Add provenance: true to your npm publish workflow or configure OIDC".to_string(),
            ),
            pm: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_workflows_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let finding = check_trusted_publishing(dir.path());
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_workflows_with_provenance() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workflows = dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows).expect("mkdir");
        std::fs::write(
            workflows.join("publish.yml"),
            "name: publish\npermissions:\n  id-token: write\n",
        )
        .expect("write");

        let finding = check_trusted_publishing(dir.path());
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_workflows_without_provenance() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workflows = dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows).expect("mkdir");
        std::fs::write(
            workflows.join("ci.yml"),
            "name: ci\non: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n",
        )
        .expect("write");

        let finding = check_trusted_publishing(dir.path());
        assert_eq!(finding.status, TrafficLight::Yellow);
    }
}
