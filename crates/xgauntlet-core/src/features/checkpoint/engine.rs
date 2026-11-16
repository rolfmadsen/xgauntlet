//! Orchestrator for Phase-Bound TDD Checkpoint operations.

use super::git::{
    compose_commit_message, ensure_git_dir_writable, execute_git_commit,
    seal_tracked_evidence_files, stage_workspace_changes,
};
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

/// Resolves and parses the TaskPackageInfo for a given task identifier.
pub fn load_task_package_info(
    workspace: &std::path::Path,
    task_id: &str,
) -> Option<crate::features::tasks::TaskPackageInfo> {
    let task_path =
        crate::features::tasks::resolve_task_file_path(workspace, Some(task_id)).ok()?;
    crate::features::tasks::parse_task_file(&task_path).ok()
}

pub fn validate_staged_files_scope(
    workspace: &std::path::Path,
    task_id: &str,
    staged_files: &[String],
) -> Result<(), CheckpointError> {
    let task_info = match load_task_package_info(workspace, task_id) {
        Some(info) => info,
        None => return Ok(()),
    };

    if task_info.scope.is_empty() && task_info.test_refs.is_empty() {
        return Ok(());
    }

    let mut violations = Vec::new();
    for file in staged_files {
        let norm_file = file.replace('\\', "/");

        if norm_file.starts_with("tasks/")
            || norm_file.starts_with("specs/")
            || norm_file.starts_with("docs/")
            || norm_file == "spec.md"
            || norm_file == "CONTEXT.md"
            || norm_file == "CODING_STANDARDS.md"
            || norm_file == "README.md"
            || norm_file == "evidence.md"
            || norm_file == "evidence.json"
            || norm_file == "verification-report.json"
            || norm_file == "architecture.mermaid"
            || norm_file == "architecture.md"
            || norm_file == "gauntlet.toml"
            || norm_file.starts_with(".agents/")
            || norm_file.starts_with(".github/")
            || norm_file.starts_with(".xgauntlet/")
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

    // AC3: Fail fast before running verification if .git/ is read-only
    ensure_git_dir_writable(&options.workspace)?;

    if !options.skip_verify {
        run_preflight_check(options).await?;
    }

    // AC2: Transition task file to DONE after preflight/verify passes and before staging/committing
    let task_backup = if options.phase == super::models::CheckpointPhase::Done {
        Some(
            crate::features::tasks::lifecycle::mark_task_done(&options.workspace, &task_id)
                .map_err(|e| {
                    CheckpointError::DonePhaseUnmet(format!("Failed to mark task as DONE: {e}"))
                })?,
        )
    } else {
        None
    };

    let commit_outcome: Result<(Vec<String>, String, String), CheckpointError> = (|| {
        let staged_files = stage_workspace_changes(&options.workspace)?;
        if staged_files.is_empty() && !options.allow_empty {
            return Err(CheckpointError::NothingToCommit);
        }

        validate_staged_files_scope(&options.workspace, &task_id, &staged_files)?;

        let commit_message =
            compose_commit_message(options.phase, &task_id, options.message.as_deref());

        let commit_oid =
            execute_git_commit(&options.workspace, &commit_message, options.allow_empty)?;

        Ok((staged_files, commit_message, commit_oid))
    })();

    let (staged_files, commit_message, commit_oid) = match commit_outcome {
        Ok(res) => res,
        Err(err) => {
            if let Some((path, original)) = task_backup {
                let _ = std::fs::write(path, original);
            }
            return Err(err);
        }
    };

    let handoff = if options.phase == super::models::CheckpointPhase::Done {
        let h = super::models::SessionHandoff::generate(&options.workspace, &task_id, None).ok();
        let _ = crate::features::telemetry::reset_session_health(&options.workspace, &task_id);
        h
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

    if let Ok(provenance_list) =
        super::provenance::collect_task_test_provenance(&options.workspace, &task_id)
    {
        let mut combined = report.test_provenance.take().unwrap_or_default();
        for p in provenance_list {
            if !combined
                .iter()
                .any(|c| c.test_identifier == p.test_identifier)
            {
                combined.push(p);
            }
        }
        if !combined.is_empty() {
            report.test_provenance = Some(combined);
        }
    }

    if options.phase == super::models::CheckpointPhase::Done && !options.skip_verify {
        if let Ok(post_manifest) =
            crate::features::evidence::compute_workspace_manifest(&options.workspace, None)
        {
            report.task_contract.task_digest = post_manifest.task_digest.clone();
            report.workspace_state.source_content_digest = post_manifest.source_content_digest;
            report.workspace_state.source_manifest_digest_post =
                post_manifest.source_manifest_digest;
            report.workspace_state.task_digest = post_manifest.task_digest;
            report.workspace_state.config_digest = post_manifest.config_digest;
            report.workspace_state.policy_digest = post_manifest.policy_digest;
            report.workspace_state.included_files_count = post_manifest.included_files_count;
            if let Some(ref mut vcs) = report.workspace_state.vcs {
                let short_oid = if commit_oid.len() >= 7 {
                    &commit_oid[..7]
                } else {
                    &commit_oid
                };
                vcs.head = short_oid.to_string();
                vcs.commit = short_oid.to_string();
                vcs.is_dirty = false;
            }
        }
    }

    let _ = crate::features::evidence::save_verification_report(&options.workspace, &report);

    if options.phase == super::models::CheckpointPhase::Done {
        seal_tracked_evidence_files(&options.workspace, &task_id)?;
        crate::features::tasks::parser::unbind_active_task_if_matching(
            &options.workspace,
            &task_id,
        )?;
    }

    let (next_phase, autonomous_transition) = match options.phase {
        super::models::CheckpointPhase::Red => (Some("green".to_string()), true),
        _ => (None, false),
    };

    Ok(CheckpointResult {
        phase: options.phase,
        task_id,
        commit_oid: Some(commit_oid),
        commit_message,
        staged_files,
        preflight_passed: !options.skip_verify,
        details: None,
        handoff,
        next_phase,
        autonomous_transition,
    })
}
