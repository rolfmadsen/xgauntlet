//! Domain models and schema definitions for verification reports and workspace manifests (Schema v2).

use crate::features::diagnostics::DiagnosticFinding;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// High-level verification outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationVerdict {
    Passed,
    Failed,
    Error,
    Incomplete,
    Skipped,
}

impl VerificationVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "PASSED",
            Self::Failed => "FAILED",
            Self::Error => "ERROR",
            Self::Incomplete => "INCOMPLETE",
            Self::Skipped => "SKIPPED",
        }
    }
}

/// Execution status of an individual verification check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    Passed,
    Failed,
    Error,
    Skipped,
}

impl CheckStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "PASSED",
            Self::Failed => "FAILED",
            Self::Error => "ERROR",
            Self::Skipped => "SKIPPED",
        }
    }
}

/// Execution environment where verification occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionOrigin {
    Local,
    CiUnprivileged,
    CiProtected,
}

impl ExecutionOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "LOCAL",
            Self::CiUnprivileged => "CI_UNPRIVILEGED",
            Self::CiProtected => "CI_PROTECTED",
        }
    }
}

/// Attestation status classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttestationStatus {
    Absent,
    Valid,
    Invalid,
}

/// Consumer trust decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrustDecision {
    Accepted,
    PolicyRejected,
}

/// Summary of an executed verification check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckSummary {
    pub name: String,
    pub status: String,
    pub exit_code: i32,
    pub duration_seconds: f64,
    #[serde(default)]
    pub optional: bool,
    pub passed: bool,
    #[serde(default)]
    pub log_digest: String,
}

/// VCS metadata captured during verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VcsMetadata {
    #[serde(rename = "type")]
    pub vcs_type: String,
    pub head: String,
    pub commit: String,
    #[serde(default)]
    pub is_dirty: bool,
}

impl Default for VcsMetadata {
    fn default() -> Self {
        Self {
            vcs_type: "git".to_string(),
            head: "(no git)".to_string(),
            commit: "(no git)".to_string(),
            is_dirty: false,
        }
    }
}

/// Multi-digest state of the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceState {
    #[serde(default = "default_manifest_version")]
    pub manifest_version: String,
    #[serde(default)]
    pub source_content_digest: String,
    #[serde(default)]
    pub source_manifest_digest_pre: String,
    #[serde(default)]
    pub source_manifest_digest_post: String,
    #[serde(default)]
    pub config_digest: String,
    #[serde(default)]
    pub task_digest: String,
    #[serde(default)]
    pub policy_digest: String,
    #[serde(default)]
    pub check_definitions_digest: String,
    #[serde(default)]
    pub included_files_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcs: Option<VcsMetadata>,
}

fn default_manifest_version() -> String {
    "1.0".to_string()
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            manifest_version: default_manifest_version(),
            source_content_digest: String::new(),
            source_manifest_digest_pre: String::new(),
            source_manifest_digest_post: String::new(),
            config_digest: String::new(),
            task_digest: String::new(),
            policy_digest: String::new(),
            check_definitions_digest: String::new(),
            included_files_count: 0,
            vcs: None,
        }
    }
}

/// Execution metadata including timestamps and duration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub finished_at: String,
    #[serde(default)]
    pub total_duration_seconds: f64,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

impl Default for ExecutionMetadata {
    fn default() -> Self {
        Self {
            started_at: String::new(),
            finished_at: String::new(),
            total_duration_seconds: 0.0,
            environment: BTreeMap::new(),
        }
    }
}

/// Active task contract summary bound to the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TaskContractSummary {
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub task_title: String,
    #[serde(default)]
    pub task_digest: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub unresolved_criteria: Vec<String>,
}

/// Canonical Schema v2 Verification Report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationReport {
    #[serde(rename = "$schema", default = "default_schema_url")]
    pub schema: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default = "default_execution_origin")]
    pub execution_origin: String,
    #[serde(default = "default_verdict")]
    pub verdict: String,
    #[serde(default)]
    pub task_contract: TaskContractSummary,
    #[serde(default)]
    pub workspace_state: WorkspaceState,
    #[serde(default)]
    pub execution_metadata: ExecutionMetadata,
    #[serde(default)]
    pub checks: Vec<CheckSummary>,
    #[serde(default)]
    pub diagnostics: Vec<DiagnosticFinding>,
}

fn default_schema_url() -> String {
    "https://agent-gauntlet.dev/schemas/v2/verification-report.json".to_string()
}

fn default_schema_version() -> String {
    "2.0.0".to_string()
}

fn default_execution_origin() -> String {
    "LOCAL".to_string()
}

fn default_verdict() -> String {
    "PASSED".to_string()
}

impl VerificationReport {
    /// Creates a new default local verification report for a task.
    pub fn new_local(task_id: &str) -> Self {
        Self {
            schema: default_schema_url(),
            schema_version: default_schema_version(),
            execution_origin: default_execution_origin(),
            verdict: default_verdict(),
            task_contract: TaskContractSummary {
                task_id: task_id.to_string(),
                task_title: String::new(),
                task_digest: String::new(),
                acceptance_criteria: Vec::new(),
                unresolved_criteria: Vec::new(),
            },
            workspace_state: WorkspaceState::default(),
            execution_metadata: ExecutionMetadata::default(),
            checks: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}
