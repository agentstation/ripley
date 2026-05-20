pub mod osv;

use serde::{Deserialize, Serialize};

use crate::types::{Ecosystem, Severity};

#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to parse feed response: {0}")]
    Parse(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    pub ecosystem: Ecosystem,
    pub package: String,
    pub affected_ranges: Vec<AffectedRange>,
    pub severity: Option<Severity>,
    pub summary: String,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedRange {
    pub introduced: semver::Version,
    pub fixed: Option<semver::Version>,
}
