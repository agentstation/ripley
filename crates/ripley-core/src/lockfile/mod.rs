pub mod npm;

use std::path::Path;

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
        _ => Err(LockfileError::Unsupported(filename.to_string())),
    }
}
