//! Phase-Bound TDD Checkpoint Engine (ADR 0001, ADR 0003).
//!
//! Provides mechanical, fail-closed pre-flight validation for each phase
//! of the TDD lifecycle and generates clean conventional Git commits.

pub mod engine;
pub mod git;
pub mod models;
pub mod preflight;

pub use engine::run_checkpoint;
pub use git::{compose_commit_message, execute_git_commit, extract_short_task_id, stage_workspace_changes};
pub use models::{CheckpointError, CheckpointOptions, CheckpointPhase, CheckpointResult};
pub use preflight::run_preflight_check;
