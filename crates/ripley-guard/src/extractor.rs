use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub struct Script {
    pub name: String,
    pub content: String,
    pub source_file: PathBuf,
}

const LIFECYCLE_HOOKS: &[&str] = &["preinstall", "install", "postinstall", "prepare", "prepack"];

#[derive(Deserialize)]
struct PackageJson {
    #[serde(default)]
    scripts: HashMap<String, String>,
}

pub fn extract_lifecycle_scripts(package_dir: &Path) -> anyhow::Result<Vec<Script>> {
    let package_json_path = package_dir.join("package.json");

    let content = match std::fs::read_to_string(&package_json_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "could not read {}: {e}",
                package_json_path.display()
            ));
        }
    };

    let pkg: PackageJson = serde_json::from_str(&content)?;

    let scripts = LIFECYCLE_HOOKS
        .iter()
        .filter_map(|&hook| {
            pkg.scripts.get(hook).map(|script_content| Script {
                name: hook.to_string(),
                content: script_content.clone(),
                source_file: package_json_path.clone(),
            })
        })
        .collect();

    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_lifecycle_scripts() {
        let dir = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            dir.path().join("package.json"),
            r#"{
                "name": "test-pkg",
                "scripts": {
                    "postinstall": "node setup.js",
                    "prepare": "tsc",
                    "test": "jest",
                    "build": "webpack"
                }
            }"#,
        )
        .expect("write package.json");

        let scripts = extract_lifecycle_scripts(dir.path()).expect("extract");
        assert_eq!(scripts.len(), 2);

        let postinstall = scripts.iter().find(|s| s.name == "postinstall");
        assert!(postinstall.is_some());
        assert_eq!(postinstall.expect("checked").content, "node setup.js");

        let prepare = scripts.iter().find(|s| s.name == "prepare");
        assert!(prepare.is_some());
        assert_eq!(prepare.expect("checked").content, "tsc");
    }

    #[test]
    fn test_no_lifecycle_scripts() {
        let dir = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            dir.path().join("package.json"),
            r#"{
                "name": "clean-pkg",
                "scripts": {
                    "test": "jest",
                    "build": "webpack"
                }
            }"#,
        )
        .expect("write package.json");

        let scripts = extract_lifecycle_scripts(dir.path()).expect("extract");
        assert!(scripts.is_empty());
    }

    #[test]
    fn test_no_package_json() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let scripts = extract_lifecycle_scripts(dir.path()).expect("extract");
        assert!(scripts.is_empty());
    }

    #[test]
    fn test_no_scripts_field() {
        let dir = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "bare-pkg", "version": "1.0.0"}"#,
        )
        .expect("write package.json");

        let scripts = extract_lifecycle_scripts(dir.path()).expect("extract");
        assert!(scripts.is_empty());
    }

    #[test]
    fn test_all_lifecycle_hooks() {
        let dir = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            dir.path().join("package.json"),
            r#"{
                "name": "all-hooks",
                "scripts": {
                    "preinstall": "echo pre",
                    "install": "echo install",
                    "postinstall": "echo post",
                    "prepare": "echo prepare",
                    "prepack": "echo prepack"
                }
            }"#,
        )
        .expect("write package.json");

        let scripts = extract_lifecycle_scripts(dir.path()).expect("extract");
        assert_eq!(scripts.len(), 5);
    }
}
