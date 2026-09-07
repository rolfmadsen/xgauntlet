//! Parsing and resolving task packages and contracts from markdown files.

use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::features::tasks::models::{TaskContract, TaskPackageInfo, TaskStatus};
use crate::features::tasks::okf::{parse_frontmatter, OkfError};

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task file not found: '{0}'")]
    TaskNotFound(String),

    #[error("OKF error: {0}")]
    Okf(#[from] OkfError),

    #[error("Invalid task specification: {0}")]
    InvalidSpecification(String),
}

/// Parses normalized task status from markdown content.
pub fn parse_task_status(content: &str) -> TaskStatus {
    // 1. Primary: **Status**: `STATUS` in markdown body
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("**Status**:") {
            let status = TaskStatus::parse(rest);
            if status != TaskStatus::Unknown {
                return status;
            }
        }
    }

    // 2. Secondary: Status: `STATUS`
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Status:") {
            let status = TaskStatus::parse(rest);
            if status != TaskStatus::Unknown {
                return status;
            }
        }
    }

    // 3. Fallback: YAML status: in frontmatter or headers
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("status:") {
            let status = TaskStatus::parse(rest);
            if status != TaskStatus::Unknown {
                return status;
            }
        }
    }

    TaskStatus::Unknown
}

/// Determines whether the task content represents an active approved task.
pub fn is_task_active(content: &str) -> bool {
    let status = parse_task_status(content);
    if !status.is_active() {
        return false;
    }

    let lower = content.to_ascii_lowercase();
    lower.contains("acceptance criteria") || content.contains("- [")
}

/// Determines whether the workspace has at least one active task in `tasks/`.
pub fn has_active_task(workspace: &Path) -> bool {
    let tasks_dir = workspace.join("tasks");
    if !tasks_dir.is_dir() {
        return false;
    }

    let entries = match fs::read_dir(tasks_dir) {
        Ok(e) => e,
        Err(_) => return false,
    };

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok().map(|d| d.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();

    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            if is_task_active(&content) {
                return true;
            }
        }
    }

    false
}

/// Resolves the task identifier of the first active task found in `tasks/`.
pub fn resolve_active_task_id(workspace: &Path) -> Option<String> {
    let tasks_dir = workspace.join("tasks");
    if !tasks_dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(tasks_dir).ok()?;
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok().map(|d| d.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();

    for path in paths {
        if let Ok(content) = fs::read_to_string(&path) {
            if is_task_active(&content) {
                return path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
            }
        }
    }

    None
}

/// Parses complete `TaskPackageInfo` from raw markdown content and a designated task identifier.
pub fn parse_task_content(content: &str, task_id: &str) -> Result<TaskPackageInfo, TaskError> {
    let (metadata, body) = match parse_frontmatter(content) {
        Ok((meta, body)) => (meta, body),
        Err(_) => (None, content),
    };

    let status = parse_task_status(content);
    let mut title = String::new();
    let mut intent = None;
    let mut purpose = String::new();
    let mut acceptance_criteria = Vec::new();
    let mut unresolved_criteria = Vec::new();
    let mut must_not = Vec::new();

    let mut in_purpose = false;
    let mut in_must_not = false;

    for line in body.lines() {
        let trimmed = line.trim();

        if title.is_empty() && trimmed.starts_with("# ") {
            title = trimmed[2..].trim().to_string();
            continue;
        }

        if intent.is_none() {
            if let Some(rest) = trimmed.strip_prefix("**Intent**:") {
                intent = Some(rest.trim().trim_matches('`').to_string());
            } else if let Some(rest) = trimmed.strip_prefix("intent:") {
                intent = Some(rest.trim().trim_matches('`').to_string());
            }
        }

        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("## 🎯 formål")
            || lower.starts_with("## formål")
            || lower.starts_with("## purpose")
        {
            in_purpose = true;
            in_must_not = false;
            continue;
        } else if lower.starts_with("## 🚫 must not") || lower.starts_with("## must not") {
            in_must_not = true;
            in_purpose = false;
            continue;
        } else if trimmed.starts_with("## ") {
            in_purpose = false;
            in_must_not = false;
        }

        if in_purpose && !trimmed.is_empty() {
            if !purpose.is_empty() {
                purpose.push('\n');
            }
            purpose.push_str(trimmed);
        }

        if in_must_not {
            if let Some(stripped) = trimmed.strip_prefix("- ") {
                let rule = stripped.trim();
                if !rule.is_empty() {
                    must_not.push(rule.to_string());
                }
            }
        }

        if let Some(stripped) = trimmed.strip_prefix("- [x] ") {
            let crit = stripped.trim();
            if !crit.is_empty() {
                acceptance_criteria.push(crit.to_string());
            }
        } else if let Some(stripped) = trimmed.strip_prefix("- [X] ") {
            let crit = stripped.trim();
            if !crit.is_empty() {
                acceptance_criteria.push(crit.to_string());
            }
        } else if let Some(stripped) = trimmed.strip_prefix("- [ ] ") {
            let crit = stripped.trim();
            if !crit.is_empty() {
                acceptance_criteria.push(crit.to_string());
                unresolved_criteria.push(crit.to_string());
            }
        }
    }

    Ok(TaskPackageInfo {
        task_id: task_id.to_string(),
        title,
        status,
        intent,
        purpose,
        acceptance_criteria,
        unresolved_criteria,
        must_not,
        metadata,
    })
}

/// Parses complete `TaskPackageInfo` from a file on disk.
pub fn parse_task_file(path: &Path) -> Result<TaskPackageInfo, TaskError> {
    if !path.is_file() {
        return Err(TaskError::TaskNotFound(path.display().to_string()));
    }
    let content = fs::read_to_string(path)?;
    let task_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    parse_task_content(&content, task_id)
}

/// Resolves an executable `TaskContract` from workspace tasks.
pub fn resolve_task_contract(
    workspace: &Path,
    explicit_task_id: Option<&str>,
) -> Result<TaskContract, TaskError> {
    let tasks_dir = workspace.join("tasks");
    if !tasks_dir.is_dir() {
        return Err(TaskError::TaskNotFound(format!(
            "tasks directory not found in '{}'",
            workspace.display()
        )));
    }

    let mut target_file = None;

    if let Some(task_id) = explicit_task_id {
        let entries = fs::read_dir(&tasks_dir)?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|ext| ext == "md") {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if stem == task_id
                    || name == task_id
                    || stem.starts_with(&format!("{task_id}-"))
                    || name.starts_with(&format!("{task_id}-"))
                    || stem.contains(task_id)
                {
                    target_file = Some(p);
                    break;
                }
            }
        }
        if target_file.is_none() {
            return Err(TaskError::TaskNotFound(task_id.to_string()));
        }
    } else if let Some(active_id) = resolve_active_task_id(workspace) {
        target_file = Some(tasks_dir.join(format!("{active_id}.md")));
    }

    let file_to_read = target_file.ok_or_else(|| {
        TaskError::TaskNotFound("no active task found in tasks/".to_string())
    })?;

    let info = parse_task_file(&file_to_read)?;
    Ok(TaskContract {
        task_id: info.task_id,
        title: info.title,
        acceptance_criteria: info.acceptance_criteria,
        unresolved_criteria: info.unresolved_criteria,
    })
}
