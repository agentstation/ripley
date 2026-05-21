pub mod cargo_lock;
pub mod gem;
pub mod go;
pub mod npm;
pub mod pip;
pub mod pnpm;
pub mod yarn;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::types::{Ecosystem, Severity};

#[derive(Debug, thiserror::Error)]
pub enum LockfileError {
    #[error("could not read lockfile {path}: {source}")]
    ReadFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse lockfile: {0}")]
    Parse(String),
    #[error("unsupported lockfile: {0}")]
    Unsupported(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub name: String,
    pub version: semver::Version,
    pub ecosystem: Ecosystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskySpec {
    pub package: String,
    pub specifier: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileWarning {
    pub package: String,
    pub field: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLockfile {
    pub packages: Vec<InstalledPackage>,
    pub risky_specs: Vec<RiskySpec>,
    pub warnings: Vec<LockfileWarning>,
}

pub fn parse_lockfile(path: &Path) -> Result<ParsedLockfile, LockfileError> {
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| LockfileError::Unsupported("no filename".to_string()))?;

    let content = std::fs::read_to_string(path).map_err(|e| LockfileError::ReadFile {
        path: path.to_path_buf(),
        source: e,
    })?;

    match filename {
        "package-lock.json" => npm::parse_package_lock(&content),
        "yarn.lock" => yarn::parse_yarn_lock(&content),
        "pnpm-lock.yaml" => pnpm::parse_pnpm_lock(&content),
        "Pipfile.lock" => pip::parse_pipfile_lock(&content),
        "poetry.lock" => pip::parse_poetry_lock(&content),
        "Cargo.lock" => cargo_lock::parse_cargo_lock(&content),
        "go.sum" => {
            let mod_path = path.with_file_name("go.mod");
            let mod_content = std::fs::read_to_string(&mod_path).ok();
            go::parse_go_sum_with_mod(&content, mod_content.as_deref())
        }
        "Gemfile.lock" => gem::parse_gemfile_lock(&content),
        _ => Err(LockfileError::Unsupported(filename.to_string())),
    }
}

const KNOWN_LOCKFILES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Pipfile.lock",
    "poetry.lock",
    "Cargo.lock",
    "go.sum",
    "Gemfile.lock",
];
const MAX_WALK_DEPTH: usize = 10;

pub fn find_lockfiles(dir: &Path) -> Vec<PathBuf> {
    let mut lockfiles = Vec::new();
    find_lockfiles_recursive(dir, &mut lockfiles, 0);
    lockfiles
}

fn find_lockfiles_recursive(dir: &Path, lockfiles: &mut Vec<PathBuf>, depth: usize) {
    if depth > MAX_WALK_DEPTH {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && KNOWN_LOCKFILES.contains(&name)
            {
                lockfiles.push(path);
            }
        } else if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && (name == "node_modules" || name.starts_with('.'))
            {
                continue;
            }
            find_lockfiles_recursive(&path, lockfiles, depth + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_lockfiles_in_fixtures() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("fixtures");
        let found = find_lockfiles(&fixtures);
        assert!(
            !found.is_empty(),
            "expected at least 1 lockfile in fixtures"
        );
        assert!(found.iter().all(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| KNOWN_LOCKFILES.contains(&n))
        }));
    }
}
