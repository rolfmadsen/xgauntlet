//! Data models for project scaffolding and non-destructive initialization.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Action taken or planned for a scaffolded target file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldAction {
    /// File was newly created on disk.
    Created,
    /// File already existed and was preserved without modification.
    Skipped,
    /// File already existed and was overwritten due to --force.
    Overwritten,
    /// File would be created (dry-run mode).
    WouldCreate,
    /// File would be skipped because it already exists (dry-run mode).
    WouldSkip,
    /// File would be overwritten due to --force (dry-run mode).
    WouldOverwrite,
}

impl ScaffoldAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Skipped => "skipped",
            Self::Overwritten => "overwritten",
            Self::WouldCreate => "would_create",
            Self::WouldSkip => "would_skip",
            Self::WouldOverwrite => "would_overwrite",
        }
    }
}

/// Report for an individual file handled during scaffolding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldFileReport {
    pub path: String,
    pub action: ScaffoldAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Runtime configuration options for the scaffolding engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldOptions {
    /// Target repository workspace directory.
    pub workspace: PathBuf,
    /// Explicit stack profile (if None, auto-detected from workspace files).
    pub stack: Option<String>,
    /// Whether to force overwriting existing template files.
    pub force: bool,
    /// Preview mode: evaluate actions without writing to disk.
    pub dry_run: bool,
    /// Optional project name used in documentation templates.
    pub project_name: Option<String>,
}

/// Aggregated outcome of a scaffolding execution run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldResult {
    pub workspace: PathBuf,
    pub stack: String,
    pub files: Vec<ScaffoldFileReport>,
    pub created_count: usize,
    pub skipped_count: usize,
    pub overwritten_count: usize,
    pub is_success: bool,
}

/// Domain errors encountered during scaffolding.
#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported stack profile '{0}'.")]
    InvalidStack(String),

    #[error("Invalid workspace path: '{0}'.")]
    InvalidWorkspace(String),
}
