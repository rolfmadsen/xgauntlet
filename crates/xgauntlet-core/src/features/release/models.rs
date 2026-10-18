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
    /// Working tree contains uncommitted modifications or untracked files.
    DirtyWorktree,
    /// Code formatting check failed according to gauntlet.toml format layer.
    UnformattedCode,
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
            Self::DirtyWorktree => "DIRTY_WORKTREE",
            Self::UnformattedCode => "UNFORMATTED_CODE",
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

    #[error("Invalid SemVer version string: {0}")]
    InvalidVersion(String),

    #[error("General release engine error: {0}")]
    General(String),
}

/// Semantic versioning bump increment type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemverBump {
    Patch,
    Minor,
    Major,
}

impl std::str::FromStr for SemverBump {
    type Err = ReleaseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().trim() {
            "patch" => Ok(Self::Patch),
            "minor" => Ok(Self::Minor),
            "major" => Ok(Self::Major),
            other => Err(ReleaseError::General(format!(
                "Invalid SemVer bump increment: '{other}' (expected: patch, minor, major)"
            ))),
        }
    }
}

/// Calculates the next semantic version given a current version string and bump increment.
pub fn bump_semver(current: &str, bump: SemverBump) -> Result<String, ReleaseError> {
    let clean = current.trim().trim_start_matches(['v', 'V']);
    let main_part = clean.split(['-', '+']).next().unwrap_or(clean);
    let parts: Vec<&str> = main_part.split('.').collect();
    if parts.len() < 3 {
        return Err(ReleaseError::InvalidVersion(format!(
            "Cannot parse SemVer from '{current}': expected 3 numeric components (MAJOR.MINOR.PATCH)"
        )));
    }

    let major: u64 = parts[0].parse().map_err(|_| {
        ReleaseError::InvalidVersion(format!("Non-numeric MAJOR in SemVer '{current}'"))
    })?;
    let minor: u64 = parts[1].parse().map_err(|_| {
        ReleaseError::InvalidVersion(format!("Non-numeric MINOR in SemVer '{current}'"))
    })?;
    let patch: u64 = parts[2].parse().map_err(|_| {
        ReleaseError::InvalidVersion(format!("Non-numeric PATCH in SemVer '{current}'"))
    })?;

    match bump {
        SemverBump::Major => Ok(format!("{}.0.0", major + 1)),
        SemverBump::Minor => Ok(format!("{}.{}.0", major, minor + 1)),
        SemverBump::Patch => Ok(format!("{}.{}.{}", major, minor, patch + 1)),
    }
}

/// Generates standard conventional Git publication commands for a release version, branch, and explicit files.
pub fn generate_release_git_instructions_with_files(
    new_version: &str,
    branch: &str,
    files: &[String],
) -> String {
    let ver = new_version.trim().trim_start_matches(['v', 'V']);
    let add_cmd = if files.is_empty() {
        "git add -u verification-report.json evidence.md".to_string()
    } else {
        let mut clean_files = Vec::new();
        for f in files {
            if !clean_files.contains(f) {
                clean_files.push(f.clone());
            }
        }
        format!("git add {}", clean_files.join(" "))
    };
    format!(
        "{add_cmd}\n\
         git commit -m \"chore(release): prepare v{ver}\"\n\
         git tag -a v{ver} -m \"Release v{ver}\"\n\
         git push origin {branch} v{ver}"
    )
}

/// Generates standard conventional Git publication commands for a release version and branch.
pub fn generate_release_git_instructions(new_version: &str, branch: &str) -> String {
    generate_release_git_instructions_with_files(new_version, branch, &[])
}
