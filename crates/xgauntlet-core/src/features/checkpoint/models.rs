//! Data structures, options, and error models for Phase-Bound TDD Checkpoint Engine.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
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
    /// Explicit task identifier (optional, auto-resolved from workspace if omitted).
    pub task_id: Option<String>,
    /// Custom commit message (conventional prefix will be added if missing).
    pub message: Option<String>,
    /// Repository workspace root.
    pub workspace: PathBuf,
    /// Skip pre-flight phase verification checks (emergency override).
    pub skip_verify: bool,
    /// Allow creating empty commit when no files changed.
    pub allow_empty: bool,
    /// Strictly require tests to be committed in RED phase; reject any in-flight test additions.
    pub strict_provenance: bool,
    /// Allow born-green tests to pass without inverse preflight verification (emergency override).
    pub allow_born_green: bool,
}

impl CheckpointOptions {
    pub fn new(phase: CheckpointPhase, workspace: impl Into<PathBuf>) -> Self {
        Self {
            phase,
            task_id: None,
            message: None,
            workspace: workspace.into(),
            skip_verify: false,
            allow_empty: false,
            strict_provenance: false,
            allow_born_green: false,
        }
    }

    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
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

    pub fn with_strict_provenance(mut self, strict: bool) -> Self {
        self.strict_provenance = strict;
        self
    }

    pub fn with_allow_born_green(mut self, allow: bool) -> Self {
        self.allow_born_green = allow;
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
    /// Pre-rendered session handoff card (populated upon Done phase).
    #[serde(default)]
    pub handoff: Option<SessionHandoff>,
}

/// Deterministic Session Handoff metadata and pre-rendered presentation card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHandoff {
    pub task_id: String,
    pub branch: String,
    pub commit_oid: String,
    pub commits_ahead: usize,
    pub is_clean: bool,
    pub next_role: String,
    pub formatted_card: String,
}

impl SessionHandoff {
    /// Deterministically generates session handoff card by inspecting the local git workspace.
    pub fn generate(
        workspace: &std::path::Path,
        task_id: &str,
        next_role: Option<&str>,
    ) -> Result<Self, CheckpointError> {
        let branch =
            super::git::get_current_branch(workspace).unwrap_or_else(|_| "main".to_string());
        let commit_oid =
            super::git::get_head_commit_oid(workspace).unwrap_or_else(|_| "unknown".to_string());
        let short_oid = if commit_oid.len() >= 7 {
            &commit_oid[..7]
        } else {
            &commit_oid
        };
        let commits_ahead = super::git::get_commits_ahead(workspace).unwrap_or(0);
        let is_clean = super::git::is_worktree_clean(workspace).unwrap_or(true);
        let role = next_role.unwrap_or("Software Engineer").to_string();
        let short_task = super::git::extract_short_task_id(task_id);

        let clean_str = if is_clean { "clean" } else { "dirty" };
        let ahead_note = if commits_ahead > 0 {
            format!(" ({commits_ahead} commits ahead of remote)")
        } else {
            String::new()
        };

        let starter_prompt = format!(
            "Hej! Vi skal i denne session fortsætte arbejdet efter afslutning af task {short_task}.\n\
             Læs venligst CONTEXT.md og spec.md samt opgavelisten i tasks/ for at afklare næste opgave."
        );

        let formatted_card = format!(
            "> ### 🏁 SESSION HANDOFF • {short_task}\n\
             > **Status**: `TASK: DONE` | **Evidens**: `FORSEGLET (Two-Tier Model)` | **Næste Rolle**: `{role}`\n\
             > **Git State**: `{branch}@{short_oid}` • `{clean_str}{ahead_note}`\n\
             > 💡 *Start venligst en frisk chat-session for at bevare et skarpt kontekstvindue uden context rot.*\n\
             >\n\
             > 📋 **Kopiér og indsæt følgende starter-prompt i en ny chat:**\n\
             > ```text\n\
             > {starter_prompt}\n\
             > ```\n\
             >\n\
             > 🚀 **Synkroniser dine commits til remote:**\n\
             > ```bash\n\
             > git push origin {branch}\n\
             > ```\n"
        );

        Ok(Self {
            task_id: task_id.to_string(),
            branch,
            commit_oid,
            commits_ahead,
            is_clean,
            next_role: role,
            formatted_card,
        })
    }
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

    #[error("Scope violation during checkpoint: staged files outside active task '{task_id}' scope: {violations:?}")]
    ScopeViolation {
        task_id: String,
        violations: Vec<String>,
    },

    #[error("Born-green tests detected in GREEN/REFACTOR checkpoint without prior RED failure: {tests:?}")]
    BornGreenTestDetected { tests: Vec<String> },

    #[error("Tautological test detected: '{test_name}' passed against baseline code before production changes")]
    TautologicalTestDetected { test_name: String },

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
