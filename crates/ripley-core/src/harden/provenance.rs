use std::path::Path;

use crate::audit::TrafficLight;

use super::HardenFinding;

pub fn check_provenance(project_path: &Path) -> Vec<HardenFinding> {
    vec![check_trusted_publishing(project_path)]
}

pub fn check_trusted_publishing(project_path: &Path) -> HardenFinding {
    let workflows_dir = project_path.join(".github/workflows");

    if !workflows_dir.exists() {
        return evaluate_trusted_publishing(None);
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

    evaluate_trusted_publishing(Some(has_provenance))
}

pub fn evaluate_trusted_publishing(has_provenance: Option<bool>) -> HardenFinding {
    match has_provenance {
        None => HardenFinding {
            name: "Trusted Publishing".to_string(),
            status: TrafficLight::Yellow,
            detail: "No GitHub workflows found — cannot check OIDC publishing".to_string(),
            fix_command: None,
            pm: "all".to_string(),
        },
        Some(true) => HardenFinding {
            name: "Trusted Publishing".to_string(),
            status: TrafficLight::Green,
            detail: "OIDC/provenance configuration detected in CI workflows".to_string(),
            fix_command: None,
            pm: "all".to_string(),
        },
        Some(false) => HardenFinding {
            name: "Trusted Publishing".to_string(),
            status: TrafficLight::Yellow,
            detail: "No provenance/OIDC configuration found in workflows".to_string(),
            fix_command: Some(
                "Add provenance: true to your npm publish workflow or configure OIDC".to_string(),
            ),
            pm: "all".to_string(),
        },
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
        assert_eq!(finding.pm, "all");
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
        assert_eq!(finding.pm, "all");
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
        assert_eq!(finding.pm, "all");
    }

    // --- evaluate tests ---

    #[test]
    fn test_evaluate_no_workflows() {
        let f = evaluate_trusted_publishing(None);
        assert_eq!(f.status, TrafficLight::Yellow);
        assert_eq!(f.pm, "all");
    }

    #[test]
    fn test_evaluate_has_provenance() {
        let f = evaluate_trusted_publishing(Some(true));
        assert_eq!(f.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_no_provenance() {
        let f = evaluate_trusted_publishing(Some(false));
        assert_eq!(f.status, TrafficLight::Yellow);
        assert!(f.fix_command.is_some());
    }
}
