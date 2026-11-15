//! Automated per-task git worktree provisioning inside `.worktrees/<task-id>/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::features::adapters::workspace::resolve_primary_repo_root;
use crate::features::checkpoint::extract_short_task_id;
use crate::features::policy::has_path_traversal;
use crate::features::tasks::parser::{
    bind_active_task, is_task_active, resolve_task_file_path, TaskError,
};

/// Summary returned after creating and binding an in-repo task worktree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorktreeCreateResult {
    pub task_id: String,
    pub short_id: String,
    pub branch: String,
    pub worktree_path: PathBuf,
    pub primary_repo_root: PathBuf,
    pub bound_active_task: String,
}

fn validate_worktree_segment(raw: &str, label: &str) -> Result<String, TaskError> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || has_path_traversal(trimmed)
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.starts_with('-')
    {
        return Err(TaskError::InvalidSpecification(format!(
            "Invalid {label} '{raw}': must be a safe single path segment"
        )));
    }
    Ok(trimmed.to_string())
}

fn branch_exists(repo_root: &Path, branch: &str) -> Result<bool, TaskError> {
    let ref_name = format!("refs/heads/{branch}");
    let status = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &ref_name])
        .current_dir(repo_root)
        .status()?;
    Ok(status.success())
}

fn task_exists_in_git_ref(repo_root: &Path, git_ref: &str, task_path: &Path) -> bool {
    let canon_root = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    let canon_task = task_path
        .canonicalize()
        .unwrap_or_else(|_| task_path.to_path_buf());
    let Ok(rel) = canon_task.strip_prefix(&canon_root) else {
        return false;
    };
    let rel_unix = rel.to_string_lossy().replace('\\', "/");
    let spec = format!("{git_ref}:{rel_unix}");
    Command::new("git")
        .args(["cat-file", "-e", &spec])
        .current_dir(repo_root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn cleanup_failed_worktree(
    primary_root: &Path,
    worktree_path: &Path,
    branch: &str,
    created_new_branch: bool,
) {
    let wt_str = worktree_path.to_string_lossy().to_string();
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force", &wt_str])
        .current_dir(primary_root)
        .status();
    if worktree_path.exists() {
        let _ = fs::remove_dir_all(worktree_path);
    }
    let _ = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(primary_root)
        .status();
    if created_new_branch {
        let _ = Command::new("git")
            .args(["branch", "-D", branch])
            .current_dir(primary_root)
            .status();
    }
}

/// Creates `.worktrees/<short_id>/` inside the primary repository root for `task_id`
/// and binds `.xgauntlet/active_task` inside the new worktree.
pub fn create_task_worktree(
    workspace: &Path,
    task_id: &str,
    branch: Option<&str>,
) -> Result<WorktreeCreateResult, TaskError> {
    let primary_root = resolve_primary_repo_root(workspace).unwrap_or_else(|| {
        workspace
            .canonicalize()
            .unwrap_or_else(|_| workspace.to_path_buf())
    });

    let task_path = resolve_task_file_path(&primary_root, Some(task_id))?;
    let task_content = fs::read_to_string(&task_path)?;
    if !is_task_active(&task_content) {
        return Err(TaskError::InvalidSpecification(format!(
            "Task '{task_id}' is not ACTIVE and cannot be bound to a worktree"
        )));
    }

    let canonical_id = task_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| TaskError::TaskNotFound(task_id.to_string()))?
        .to_string();

    let short_id = validate_worktree_segment(extract_short_task_id(&canonical_id), "task id")?;
    let target_branch = match branch {
        Some(custom) => {
            let trimmed = custom.trim();
            if trimmed.is_empty() || has_path_traversal(trimmed) || trimmed.starts_with('-') {
                return Err(TaskError::InvalidSpecification(format!(
                    "Invalid branch name '{custom}'"
                )));
            }
            trimmed.to_string()
        }
        None => format!("task/{short_id}"),
    };

    let exists = branch_exists(&primary_root, &target_branch)?;
    let source_ref = if exists {
        target_branch.as_str()
    } else {
        "HEAD"
    };
    if !task_exists_in_git_ref(&primary_root, source_ref, &task_path) {
        return Err(TaskError::InvalidSpecification(format!(
            "Task file '{}' must be committed to '{source_ref}' before creating a worktree",
            task_path.display()
        )));
    }

    let worktrees_dir = primary_root.join(".worktrees");
    let worktree_path = worktrees_dir.join(&short_id);
    if worktree_path.exists() {
        return Err(TaskError::InvalidSpecification(format!(
            "Worktree path '{}' already exists",
            worktree_path.display()
        )));
    }
    fs::create_dir_all(&worktrees_dir)?;

    let wt_str = worktree_path.to_string_lossy().to_string();
    let created_new_branch = !exists;
    let mut cmd = Command::new("git");
    cmd.current_dir(&primary_root).arg("worktree").arg("add");
    if exists {
        cmd.arg(&wt_str).arg(&target_branch);
    } else {
        cmd.arg("-b").arg(&target_branch).arg(&wt_str);
    }

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        cleanup_failed_worktree(&primary_root, &worktree_path, &target_branch, false);
        return Err(TaskError::InvalidSpecification(format!(
            "git worktree add failed: {}",
            stderr.trim()
        )));
    }

    let bound_active_task = match bind_active_task(&worktree_path, &canonical_id) {
        Ok(id) => id,
        Err(err) => {
            cleanup_failed_worktree(
                &primary_root,
                &worktree_path,
                &target_branch,
                created_new_branch,
            );
            return Err(err);
        }
    };

    Ok(WorktreeCreateResult {
        task_id: canonical_id,
        short_id,
        branch: target_branch,
        worktree_path,
        primary_repo_root: primary_root,
        bound_active_task,
    })
}
