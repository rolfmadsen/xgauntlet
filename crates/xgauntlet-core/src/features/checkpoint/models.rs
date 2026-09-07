//! Data structures, options, and error models for Phase-Bound TDD Checkpoint Engine.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::features::config::ConfigError;
use crate::features::tasks::TaskError;

/// TDD lifecycle phases bound to deterministic checkpoint invariant checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointPhase {
    Spec,
    Red,
    Green,
    Refactor,
    Done,
}

impl CheckpointPhase {
    /// Returns canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Red => "red",
            Self::Green => "green",
            Self::Refactor => "refactor",
            Self::Done => "done",
        }
    }

    /// Conventional commit prefix (e.g. `feat`, `test`, `refactor`, `chore`, `task`).
    pub fn conventional_prefix(&self) -> &'static str {
        match self {
            Self::Spec => "task",
            Self::Red => "test",
            Self::Green => "feat",
            Self::Refactor => "refactor",
            Self::Done => "chore",
        }
    }

    /// Associated uppercase phase tag (e.g. `[RED]`, `[GREEN]`).
    pub fn phase_tag(&self) -> Option<&'static str> {
        match self {
            Self::Spec => None,
            Self::Red => Some("[RED]"),
            Self::Green => Some("[GREEN]"),
            Self::Refactor => Some("[REFACTOR]"),
            Self::Done => Some("[DONE]"),
        }
    }

    /// Canonical default conventional commit message for a given task identifier.
    pub fn default_message(&self, task_id: &str) -> String {
        match self {
            Self::Spec => format!("task({task_id}): initialize task specification and criteria"),
            Self::Red => format!("test({task_id}): add failing acceptance test [RED]"),
            Self::Green => {
                format!("feat({task_id}): implement minimal logic to satisfy test [GREEN]")
            }
            Self::Refactor => {
                format!("refactor({task_id}): clean up module boundaries and types [REFACTOR]")
            }
            Self::Done => format!("chore({task_id}): seal evidence and mark task DONE"),
        }
    }
}

impl fmt::Display for CheckpointPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CheckpointPhase {
    type Err = CheckpointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "spec" => Ok(Self::Spec),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "refactor" => Ok(Self::Refactor),
            "done" => Ok(Self::Done),
            other => Err(CheckpointError::InvalidPhase(other.to_string())),
        }
    }
}

/// Options controlling checkpoint invariant evaluation and git commit execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointOptions {
    /// Designated TDD phase.
    pub phase: CheckpointPhase,
    /// Custom commit message (conventional prefix will be added if missing).
    pub message: Option<String>,
    /// Repository workspace root.
    pub workspace: PathBuf,
    /// Skip pre-flight phase verification checks (emergency override).
    pub skip_verify: bool,
    /// Allow creating empty commit when no files changed.
    pub allow_empty: bool,
}

impl CheckpointOptions {
    pub fn new(phase: CheckpointPhase, workspace: impl Into<PathBuf>) -> Self {
        Self {
            phase,
            message: None,
            workspace: workspace.into(),
            skip_verify: false,
            allow_empty: false,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_skip_verify(mut self, skip: bool) -> Self {
        self.skip_verify = skip;
        self
    }

    pub fn with_allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }
}

/// Result returned after executing a phase checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointResult {
    /// Phase executed.
    pub phase: CheckpointPhase,
    /// Active task identifier.
    pub task_id: String,
    /// Git commit OID (40-char SHA-1).
    pub commit_oid: Option<String>,
    /// Full conventional commit message used.
    pub commit_message: String,
    /// List of staged files included in the commit.
    pub staged_files: Vec<String>,
    /// Whether pre-flight checks passed (or were skipped).
    pub preflight_passed: bool,
    /// Optional contextual details or warnings.
    pub details: Option<String>,
}

/// Structured errors emitted during phase checkpoint operations.
#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("Unknown or unsupported checkpoint phase: '{0}' (expected: spec, red, green, refactor, done)")]
    InvalidPhase(String),

    #[error("No active task found in tasks/")]
    NoActiveTask,

    #[error("RED phase unmet: expected failing test assertions in RED phase, but tests passed")]
    RedPhaseUnmet { details: String },

    #[error("GREEN phase unmet: expected passing tests in GREEN phase, but tests failed")]
    GreenPhaseUnmet { details: String },

    #[error("SPEC phase unmet: task specification has validation errors: {0}")]
    SpecPhaseUnmet(String),

    #[error("REFACTOR phase unmet: {details}")]
    RefactorPhaseUnmet { details: String },

    #[error("DONE phase unmet: gauntlet verification pipeline failed: {0}")]
    DonePhaseUnmet(String),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("Nothing to commit: working tree and staging index are clean")]
    NothingToCommit,

    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Task error: {0}")]
    TaskError(#[from] TaskError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
