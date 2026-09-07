//! Git staging and conventional commit creation primitives.
//!
//! Enforces ADR 0003: strictly local operations, no remote push, no destructive clean/resets.

use std::path::Path;
use std::process::Command;

use super::models::{CheckpointError, CheckpointPhase};

/// Extracts the short task identifier (e.g. "014" from "014-phase-checkpoint-engine").
pub fn extract_short_task_id(task_id: &str) -> &str {
    if let Some((prefix, _)) = task_id.split_once('-') {
        if !prefix.is_empty() {
            return prefix;
        }
    }
    task_id
}

/// Formats a conventional git commit message adhering to xGauntlet phase checkpoint standards.
pub fn compose_commit_message(
    phase: CheckpointPhase,
    task_id: &str,
    user_message: Option<&str>,
) -> String {
    let short_id = extract_short_task_id(task_id);

    let raw_msg = match user_message {
        Some(m) if !m.trim().is_empty() => m.trim(),
        _ => return phase.default_message(short_id),
    };

    // If caller provided a fully formed conventional message, reuse it
    let prefix = phase.conventional_prefix();
    let tag = phase.phase_tag();

    let clean_body = if let Some(stripped) = raw_msg.strip_prefix(&format!("{prefix}({short_id}):"))
    {
        stripped.trim()
    } else if let Some(stripped) = raw_msg.strip_prefix(&format!("{prefix}:")) {
        stripped.trim()
    } else {
        raw_msg
    };

    let body_without_tag = if let Some(t) = tag {
        clean_body.strip_suffix(t).unwrap_or(clean_body).trim()
    } else {
        clean_body
    };

    match tag {
        Some(t) => format!("{prefix}({short_id}): {body_without_tag} {t}"),
        None => format!("{prefix}({short_id}): {body_without_tag}"),
    }
}

/// Stages all workspace modifications into the Git index.
///
/// Returns list of staged files.
pub fn stage_workspace_changes(workspace: &Path) -> Result<Vec<String>, CheckpointError> {
    let add_status = Command::new("git")
        .args(["add", "-A"])
        .current_dir(workspace)
        .status()
        .map_err(|e| CheckpointError::GitError(format!("Failed to execute 'git add -A': {e}")))?;

    if !add_status.success() {
        return Err(CheckpointError::GitError(
            "'git add -A' failed with non-zero exit code".to_string(),
        ));
    }

    let diff_output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(workspace)
        .output()
        .map_err(|e| {
            CheckpointError::GitError(format!("Failed to inspect cached diff: {e}"))
        })?;

    if !diff_output.status.success() {
        return Err(CheckpointError::GitError(
            "'git diff --cached' failed with non-zero exit code".to_string(),
        ));
    }

    let staged_files: Vec<String> = String::from_utf8_lossy(&diff_output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(staged_files)
}

/// Executes a local git commit and returns the generated commit OID hash.
///
/// Under ADR 0003, this NEVER pushes to any remote.
pub fn execute_git_commit(
    workspace: &Path,
    commit_message: &str,
    allow_empty: bool,
) -> Result<String, CheckpointError> {
    let mut commit_cmd = Command::new("git");
    commit_cmd.arg("commit");
    commit_cmd.args(["-m", commit_message]);
    if allow_empty {
        commit_cmd.arg("--allow-empty");
    }
    commit_cmd.current_dir(workspace);

    let output = commit_cmd.output().map_err(|e| {
        CheckpointError::GitError(format!("Failed to execute 'git commit': {e}"))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(CheckpointError::GitError(format!(
            "git commit failed (exit code {:?}): {}\n{}",
            output.status.code(),
            stderr.trim(),
            stdout.trim()
        )));
    }

    let oid_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace)
        .output()
        .map_err(|e| CheckpointError::GitError(format!("Failed to rev-parse HEAD: {e}")))?;

    if !oid_output.status.success() {
        return Err(CheckpointError::GitError(
            "Failed to resolve HEAD commit OID after commit".to_string(),
        ));
    }

    let oid = String::from_utf8_lossy(&oid_output.stdout).trim().to_string();
    Ok(oid)
}
