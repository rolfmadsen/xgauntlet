//! Domain models and status enums for tasks and specifications.

use serde::{Deserialize, Serialize};

use crate::features::diagnostics::DiagnosticFinding;
use crate::features::tasks::okf::OkfMetadata;

/// Task lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Draft,
    Active,
    InProgress,
    Wip,
    Todo,
    Reopened,
    Done,
    Completed,
    Deprecated,
    Superseded,
    Rejected,
    Unknown,
}

impl TaskStatus {
    /// Normalizes and parses status from string.
    pub fn parse(raw: &str) -> Self {
        let clean = raw
            .trim()
            .trim_matches('`')
            .to_ascii_uppercase()
            .replace('-', "_");
        match clean.as_str() {
            "DRAFT" => Self::Draft,
            "ACTIVE" => Self::Active,
            "IN_PROGRESS" => Self::InProgress,
            "WIP" => Self::Wip,
            "TODO" => Self::Todo,
            "REOPENED" => Self::Reopened,
            "DONE" => Self::Done,
            "COMPLETED" => Self::Completed,
            "DEPRECATED" => Self::Deprecated,
            "SUPERSEDED" => Self::Superseded,
            "REJECTED" => Self::Rejected,
            _ => Self::Unknown,
        }
    }

    /// Checks if this status grants active work permission.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Active | Self::InProgress | Self::Wip | Self::Reopened
        )
    }

    /// Returns string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Active => "ACTIVE",
            Self::InProgress => "IN_PROGRESS",
            Self::Wip => "WIP",
            Self::Todo => "TODO",
            Self::Reopened => "REOPENED",
            Self::Done => "DONE",
            Self::Completed => "COMPLETED",
            Self::Deprecated => "DEPRECATED",
            Self::Superseded => "SUPERSEDED",
            Self::Rejected => "REJECTED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Parsed metadata and criteria for a task package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskPackageInfo {
    pub task_id: String,
    pub title: String,
    pub status: TaskStatus,
    pub intent: Option<String>,
    pub purpose: String,
    pub acceptance_criteria: Vec<String>,
    pub unresolved_criteria: Vec<String>,
    pub must_not: Vec<String>,
    pub metadata: Option<OkfMetadata>,
}

/// Concise contract representation for execution and verification binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskContract {
    pub task_id: String,
    pub title: String,
    pub acceptance_criteria: Vec<String>,
    pub unresolved_criteria: Vec<String>,
}

/// Consolidated assessment of task specification completeness and business rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecReadinessReport {
    pub is_valid: bool,
    pub task_id: String,
    pub diagnostics: Vec<DiagnosticFinding>,
    pub inspected_files: Vec<String>,
    pub must_not_rules: Vec<String>,
    pub acceptance_criteria: Vec<String>,
}

impl SpecReadinessReport {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            is_valid: true,
            task_id: task_id.into(),
            diagnostics: Vec::new(),
            inspected_files: Vec::new(),
            must_not_rules: Vec::new(),
            acceptance_criteria: Vec::new(),
        }
    }
}
