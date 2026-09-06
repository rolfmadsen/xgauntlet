//! Domain models for the xGauntlet Release Readiness Gatekeeper & Manifest Harmony Engine.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

/// Structured categories for release readiness findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseFindingCategory {
    /// Version mismatch detected across project manifest files.
    ConfigVersionMismatch,
    /// CHANGELOG.md file is missing in the workspace root.
    MissingChangelog,
    /// CHANGELOG.md does not contain an entry for declared release version.
    ChangelogVersionMismatch,
    /// Architecture Decision Record in docs/adr/ is not referenced in README.md or spec.md.
    UnreferencedAdr,
    /// No supported manifest file found to determine project version.
    MissingManifest,
    /// General error encountered during release evaluation.
    GeneralError,
}

impl ReleaseFindingCategory {
    /// String slice identifier for the finding category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConfigVersionMismatch => "CONFIG_VERSION_MISMATCH",
            Self::MissingChangelog => "MISSING_CHANGELOG",
            Self::ChangelogVersionMismatch => "CHANGELOG_VERSION_MISMATCH",
            Self::UnreferencedAdr => "UNREFERENCED_ADR",
            Self::MissingManifest => "MISSING_MANIFEST",
            Self::GeneralError => "GENERAL_ERROR",
        }
    }
}

impl fmt::Display for ReleaseFindingCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An individual release readiness diagnostic finding with actionable remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseFinding {
    /// Relative or absolute path to the problematic file.
    pub file_path: String,
    /// Categorization of the finding.
    pub category: ReleaseFindingCategory,
    /// Human-readable description of the violation or discrepancy.
    pub message: String,
    /// Actionable remediation suggestion to resolve the issue.
    pub remediation_hint: String,
}

/// Options configuring the release readiness check.
#[derive(Debug, Clone)]
pub struct ReleaseReadinessOptions {
    /// Root directory of the repository workspace.
    pub workspace: PathBuf,
    /// If true, allows an `[Unreleased]` section in CHANGELOG.md instead of an exact version match.
    pub allow_unreleased: bool,
    /// If true, enforces strict validation where any warning blocks release.
    pub strict: bool,
}

impl Default for ReleaseReadinessOptions {
    fn default() -> Self {
        Self {
            workspace: PathBuf::from("."),
            allow_unreleased: false,
            strict: false,
        }
    }
}

/// Consolidated assessment of repository release readiness and documentation synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseReadinessReport {
    /// Overall readiness verdict: true if all invariants pass with zero blocking errors.
    pub is_ready: bool,
    /// Target release version inferred or declared across project manifests.
    pub declared_version: String,
    /// Map of manifest file relative paths to declared version strings.
    pub versions_by_source: BTreeMap<String, String>,
    /// List of all inspected files during the evaluation.
    pub inspected_files: Vec<String>,
    /// List of version headers parsed from CHANGELOG.md.
    pub changelog_versions: Vec<String>,
    /// List of ADR files in docs/adr/ that are not referenced in README.md or spec.md.
    pub unreferenced_adrs: Vec<String>,
    /// Diagnostic findings detailing any defects or mismatches.
    pub diagnostics: Vec<ReleaseFinding>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Structured error hierarchy for the release subsystem.
#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("I/O error during release evaluation: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization or parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid workspace path: {0}")]
    InvalidWorkspace(String),

    #[error("General release engine error: {0}")]
    General(String),
}
