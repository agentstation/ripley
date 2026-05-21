use serde::Deserialize;

use crate::types::Severity;

#[derive(Debug, Clone, Deserialize)]
pub struct RemediationTemplate {
    pub id: String,
    pub name: String,
    pub attack_type: String,
    pub description: String,
    pub ioc_checks: Vec<String>,
    pub credential_rotation: Vec<RotationStep>,
    pub verification_steps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RotationStep {
    pub credential: String,
    pub command: String,
    pub priority: Severity,
}

static TEMPLATES: std::sync::LazyLock<Vec<RemediationTemplate>> = std::sync::LazyLock::new(|| {
    let sources: &[&str] = &[
        include_str!("../../../../templates/npm_worm.toml"),
        include_str!("../../../../templates/pypi_pth_injection.toml"),
        include_str!("../../../../templates/credential_exfil.toml"),
        include_str!("../../../../templates/ide_config_poison.toml"),
    ];

    sources
        .iter()
        .map(|s| toml::from_str(s).expect("compiled template must parse"))
        .collect()
});

pub fn load_templates() -> &'static [RemediationTemplate] {
    &TEMPLATES
}

pub fn find_template(attack_type: &str) -> Option<&'static RemediationTemplate> {
    TEMPLATES.iter().find(|t| t.attack_type == attack_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_templates_all_present() {
        let templates = load_templates();
        assert_eq!(templates.len(), 4);
    }

    #[test]
    fn test_find_template_npm_worm() {
        let template = find_template("npm_worm");
        assert!(template.is_some());
        let t = template.unwrap();
        assert_eq!(t.id, "npm-worm");
        assert!(!t.ioc_checks.is_empty());
        assert!(!t.credential_rotation.is_empty());
        assert!(!t.verification_steps.is_empty());
    }

    #[test]
    fn test_find_template_pypi() {
        let template = find_template("pypi_pth_injection");
        assert!(template.is_some());
    }

    #[test]
    fn test_find_template_credential_exfil() {
        let template = find_template("credential_exfil");
        assert!(template.is_some());
    }

    #[test]
    fn test_find_template_ide_config() {
        let template = find_template("ide_config_poison");
        assert!(template.is_some());
    }

    #[test]
    fn test_find_template_nonexistent() {
        let template = find_template("nonexistent_attack");
        assert!(template.is_none());
    }

    #[test]
    fn test_template_has_required_fields() {
        for t in load_templates() {
            assert!(!t.id.is_empty());
            assert!(!t.name.is_empty());
            assert!(!t.attack_type.is_empty());
            assert!(!t.description.is_empty());
            assert!(!t.ioc_checks.is_empty());
            assert!(!t.credential_rotation.is_empty());
            assert!(!t.verification_steps.is_empty());
        }
    }
}
