//! Orchestrator for Phase-Bound TDD Checkpoint operations.

use super::git::{compose_commit_message, execute_git_commit, stage_workspace_changes};
use super::models::{CheckpointError, CheckpointOptions, CheckpointResult};
use super::preflight::run_preflight_check;
use crate::features::tasks::resolve_active_task_id;

/// Orchestrates pre-flight validation, staging, and atomic local Git commits.
pub async fn run_checkpoint(options: &CheckpointOptions) -> Result<CheckpointResult, CheckpointError> {
    let task_id = resolve_active_task_id(&options.workspace).ok_or(CheckpointError::NoActiveTask)?;

    if !options.skip_verify {
        run_preflight_check(options).await?;
    }

    let staged_files = stage_workspace_changes(&options.workspace)?;
    if staged_files.is_empty() && !options.allow_empty {
        return Err(CheckpointError::NothingToCommit);
    }

    let commit_message =
        compose_commit_message(options.phase, &task_id, options.message.as_deref());

    let commit_oid =
        execute_git_commit(&options.workspace, &commit_message, options.allow_empty)?;

    Ok(CheckpointResult {
        phase: options.phase,
        task_id,
        commit_oid: Some(commit_oid),
        commit_message,
        staged_files,
        preflight_passed: !options.skip_verify,
        details: None,
    })
}
