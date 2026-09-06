//! Domain models for the xGauntlet Fast Environment, Git, and Toolchain Diagnostics Engine.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

/// Categories for diagnostic checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorCategory {
    /// Host platform, OS, architecture, and core engine metadata.
    Host,
    /// Git repository presence, configuration, branch status, and safety invariants.
    Git,
    /// In-repo governance files (spec, context glossary, config, tasks, ADRs).
    Governance,
    /// Toolchains, compilers, linters, and testing runners for detected stacks.
    Toolchains,
    /// Embedded WebAssembly policy engine readiness and sandbox invariants.
    Engine,
}

impl DoctorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Git => "git",
            Self::Governance => "governance",
            Self::Toolchains => "toolchains",
            Self::Engine => "engine",
        }
    }

    pub fn parse_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "host" | "env" | "platform" => Some(Self::Host),
            "git" | "vcs" => Some(Self::Git),
            "gov" | "governance" | "spec" | "config" => Some(Self::Governance),
            "tool" | "tools" | "toolchain" | "toolchains" | "stack" => Some(Self::Toolchains),
            "engine" | "wasm" | "policy" => Some(Self::Engine),
            _ => None,
        }
    }
}

impl fmt::Display for DoctorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Execution status of an individual diagnostic check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DoctorCheckStatus {
    /// Check passed successfully with no issues.
    Pass,
    /// Check encountered a non-critical warning or sub-optimal configuration.
    Warn,
    /// Check failed a required precondition or invariant.
    Fail,
    /// Informational note with no negative impact.
    Info,
}

impl DoctorCheckStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
            Self::Info => "INFO",
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Pass | Self::Info)
    }
}

/// Consolidated verdict for the workspace diagnostic check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DoctorVerdict {
    /// All mandatory checks passed, zero warnings or acceptable minor infos.
    Healthy,
    /// All mandatory checks passed, but warnings or sub-optimal configurations exist.
    Degraded,
    /// One or more required checks failed. Remediation required.
    Critical,
}

impl DoctorVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Degraded => "DEGRADED",
            Self::Critical => "CRITICAL",
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }
}

/// Result of an individual diagnostic check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorCheckItem {
    /// Identifier name of the check (e.g. "git_binary", "governance_spec").
    pub name: String,
    /// Category grouping for the check.
    pub category: DoctorCategory,
    /// Final status of this check.
    pub status: DoctorCheckStatus,
    /// Short summary message.
    pub message: String,
    /// Optional technical details or command output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Optional actionable remediation instruction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    /// Execution duration for this check in milliseconds.
    pub duration_ms: u64,
}

impl DoctorCheckItem {
    pub fn pass(
        name: impl Into<String>,
        category: DoctorCategory,
        message: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            status: DoctorCheckStatus::Pass,
            message: message.into(),
            detail: None,
            remediation: None,
            duration_ms,
        }
    }

    pub fn warn(
        name: impl Into<String>,
        category: DoctorCategory,
        message: impl Into<String>,
        detail: Option<String>,
        remediation: Option<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            status: DoctorCheckStatus::Warn,
            message: message.into(),
            detail,
            remediation,
            duration_ms,
        }
    }

    pub fn fail(
        name: impl Into<String>,
        category: DoctorCategory,
        message: impl Into<String>,
        detail: Option<String>,
        remediation: Option<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            status: DoctorCheckStatus::Fail,
            message: message.into(),
            detail,
            remediation,
            duration_ms,
        }
    }

    pub fn info(
        name: impl Into<String>,
        category: DoctorCategory,
        message: impl Into<String>,
        detail: Option<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            status: DoctorCheckStatus::Info,
            message: message.into(),
            detail,
            remediation: None,
            duration_ms,
        }
    }
}

/// Runtime configuration options for the doctor diagnostics engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorOptions {
    /// Target repository workspace directory.
    pub workspace: PathBuf,
    /// Verbose output showing extra technical detail for each check.
    pub verbose: bool,
    /// Strict mode: warnings will be treated as failures in the overall verdict.
    pub strict: bool,
    /// Optional category filter to run only a subset of checks.
    pub category: Option<DoctorCategory>,
}

impl Default for DoctorOptions {
    fn default() -> Self {
        Self {
            workspace: PathBuf::from("."),
            verbose: false,
            strict: false,
            category: None,
        }
    }
}

/// Comprehensive report generated by the doctor diagnostics engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    /// Workspace root inspected.
    pub workspace: PathBuf,
    /// ISO-8601 UTC timestamp of execution.
    pub timestamp: String,
    /// Core engine version.
    pub engine_version: String,
    /// Host target platform string (e.g. "linux-x86_64").
    pub target_platform: String,
    /// Evaluated diagnostic check items.
    pub checks: Vec<DoctorCheckItem>,
    /// Number of checks that passed.
    pub passed_count: usize,
    /// Number of checks with warnings.
    pub warn_count: usize,
    /// Number of checks that failed.
    pub failed_count: usize,
    /// Number of informational notes.
    pub info_count: usize,
    /// Total execution duration in milliseconds.
    pub duration_total_ms: u64,
    /// Overall workspace health verdict.
    pub verdict: DoctorVerdict,
}

impl DoctorReport {
    /// Whether the workspace passed without critical failures.
    pub fn is_healthy(&self) -> bool {
        self.verdict != DoctorVerdict::Critical && self.failed_count == 0
    }

    /// Whether the workspace passed strictly with zero warnings and zero failures.
    pub fn is_strictly_healthy(&self) -> bool {
        self.is_healthy() && self.warn_count == 0 && self.verdict == DoctorVerdict::Healthy
    }

    /// Whether the workspace has critical failures.
    pub fn has_failures(&self) -> bool {
        self.failed_count > 0 || self.verdict == DoctorVerdict::Critical
    }

    /// Filters checks belonging to a specific category.
    pub fn filter_by_category(&self, cat: DoctorCategory) -> Vec<&DoctorCheckItem> {
        self.checks.iter().filter(|c| c.category == cat).collect()
    }
}

/// Domain errors encountered during doctor execution.
#[derive(Debug, Error)]
pub enum DoctorError {
    #[error("IO error during diagnostics: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration diagnostics error: {0}")]
    Config(String),

    #[error("Specification diagnostics error: {0}")]
    Spec(String),

    #[error("Git diagnostics error: {0}")]
    Git(String),

    #[error("Diagnostic engine error: {0}")]
    Other(String),
}
