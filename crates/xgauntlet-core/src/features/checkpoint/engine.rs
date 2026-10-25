//! Orchestrator for Phase-Bound TDD Checkpoint operations.

use super::git::{compose_commit_message, execute_git_commit, stage_workspace_changes};
use super::models::{CheckpointError, CheckpointOptions, CheckpointResult};
use super::preflight::run_preflight_check;
use crate::features::tasks::{resolve_active_task_id, resolve_latest_done_task_id};

/// Resolves task ID for checkpoint operation: explicit option > active task > latest done task (if phase == Done).
pub fn resolve_checkpoint_task_id(options: &CheckpointOptions) -> Option<String> {
    if let Some(ref explicit) = options.task_id {
        return Some(explicit.clone());
    }
    if let Some(active) = resolve_active_task_id(&options.workspace) {
        return Some(active);
    }
    if options.phase == super::models::CheckpointPhase::Done {
        return resolve_latest_done_task_id(&options.workspace);
    }
    None
}

fn matches_glob_pattern(path: &str, pattern: &str) -> bool {
    let p = pattern.trim().replace('\\', "/");
    let path = path.trim().replace('\\', "/");
    if p == "*" || p == "**" || p == path {
        return true;
    }
    if let Some(prefix) = p.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    if let Some(prefix) = p.strip_suffix("/*") {
        if path.starts_with(&format!("{prefix}/")) {
            let rest = &path[prefix.len() + 1..];
            return !rest.contains('/');
        }
        return false;
    }
    if let Some(suffix) = p.strip_prefix('*') {
        return path.ends_with(suffix);
    }
    path == p
}

pub fn validate_staged_files_scope(
    workspace: &std::path::Path,
    task_id: &str,
    staged_files: &[String],
) -> Result<(), CheckpointError> {
    let tasks_dir = workspace.join("tasks");
    let mut task_path = tasks_dir.join(format!("{task_id}.md"));
    if !task_path.is_file() {
        if let Ok(entries) = std::fs::read_dir(&tasks_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|e| e == "md") {
                    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if stem == task_id || stem.starts_with(&format!("{task_id}-")) {
                        task_path = p;
                        break;
                    }
                }
            }
        }
    }

    if !task_path.is_file() {
        return Ok(());
    }

    let task_info = match crate::features::tasks::parse_task_file(&task_path) {
        Ok(info) => info,
        Err(_) => return Ok(()),
    };

    if task_info.scope.is_empty() && task_info.test_refs.is_empty() {
        return Ok(());
    }

    let mut violations = Vec::new();
    for file in staged_files {
        let norm_file = file.replace('\\', "/");

        if norm_file.starts_with("tasks/")
            || norm_file == "spec.md"
            || norm_file == "CONTEXT.md"
            || norm_file == "CODING_STANDARDS.md"
            || norm_file == "README.md"
            || norm_file == "evidence.md"
            || norm_file == "evidence.json"
            || norm_file == "verification-report.json"
            || norm_file == "gauntlet.toml"
            || norm_file.starts_with(".agents/")
            || norm_file.starts_with(".github/")
        {
            continue;
        }

        let in_scope = task_info
            .scope
            .iter()
            .any(|pat| matches_glob_pattern(&norm_file, pat));
        let in_test = task_info
            .test_refs
            .iter()
            .any(|pat| matches_glob_pattern(&norm_file, pat));

        if !in_scope && !in_test {
            violations.push(file.clone());
        }
    }

    if !violations.is_empty() {
        return Err(CheckpointError::ScopeViolation {
            task_id: task_id.to_string(),
            violations,
        });
    }

    Ok(())
}

/// Orchestrates pre-flight validation, staging, and atomic local Git commits.
pub async fn run_checkpoint(
    options: &CheckpointOptions,
) -> Result<CheckpointResult, CheckpointError> {
    let task_id = resolve_checkpoint_task_id(options).ok_or(CheckpointError::NoActiveTask)?;

    if !options.skip_verify {
        run_preflight_check(options).await?;
    }

    let staged_files = stage_workspace_changes(&options.workspace)?;
    if staged_files.is_empty() && !options.allow_empty {
        return Err(CheckpointError::NothingToCommit);
    }

    validate_staged_files_scope(&options.workspace, &task_id, &staged_files)?;

    let commit_message =
        compose_commit_message(options.phase, &task_id, options.message.as_deref());

    let commit_oid = execute_git_commit(&options.workspace, &commit_message, options.allow_empty)?;

    let handoff = if options.phase == super::models::CheckpointPhase::Done {
        super::models::SessionHandoff::generate(&options.workspace, &task_id, None).ok()
    } else {
        None
    };

    // Record SessionLineageEntry into verification report
    let mut report = crate::features::evidence::load_verification_report(&options.workspace)
        .unwrap_or_else(|_| crate::features::evidence::VerificationReport::new_local(&task_id));

    let role = match options.phase {
        super::models::CheckpointPhase::Spec => "System Architect",
        super::models::CheckpointPhase::Red
        | super::models::CheckpointPhase::Green
        | super::models::CheckpointPhase::Refactor => "TDD Craftsman",
        super::models::CheckpointPhase::Done => "Software Engineer",
    };

    let phase_str = match options.phase {
        super::models::CheckpointPhase::Spec => "SPEC",
        super::models::CheckpointPhase::Red => "RED",
        super::models::CheckpointPhase::Green => "GREEN",
        super::models::CheckpointPhase::Refactor => "REFACTOR",
        super::models::CheckpointPhase::Done => "DONE",
    };

    let session_nonce = std::env::var("XGAUNTLET_SESSION_NONCE").unwrap_or_else(|_| {
        let short_oid = if commit_oid.len() >= 8 {
            &commit_oid[..8]
        } else {
            &commit_oid
        };
        format!("sess-{short_oid}")
    });

    let lineage = report
        .lineage
        .get_or_insert_with(crate::features::evidence::SessionLineageTrail::new);
    lineage.append(session_nonce, role, phase_str, &commit_oid, None);

    let _ = crate::features::evidence::save_verification_report(&options.workspace, &report);

    Ok(CheckpointResult {
        phase: options.phase,
        task_id,
        commit_oid: Some(commit_oid),
        commit_message,
        staged_files,
        preflight_passed: !options.skip_verify,
        details: None,
        handoff,
    })
}
