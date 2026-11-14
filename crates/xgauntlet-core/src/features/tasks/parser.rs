//! Parsing and resolving task packages and contracts from markdown files.

use std::fs;
use std::path::{Path, PathBuf};
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

/// Extracts the leading numeric task prefix (e.g., `Some(42)` from `042-foo.md` or `Some(1000)` from `1000-bar.md`).
pub fn extract_task_number(path: &Path) -> Option<u32> {
    let stem = path.file_stem()?.to_str()?;
    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u32>().ok()
    }
}

fn is_ignored_task_markdown(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name == "readme.md" || name == "template.md"
}

fn collect_task_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if name_str.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_task_files_recursive(&path, out);
        } else if path.is_file()
            && path.extension().is_some_and(|ext| ext == "md")
            && !is_ignored_task_markdown(&path)
        {
            out.push(path);
        }
    }
}

/// Recursively collects all task markdown files under `tasks_dir`, excluding `README.md` and `TEMPLATE.md`,
/// sorted deterministically by numeric task prefix (supporting >999) and filename.
pub fn collect_task_files(tasks_dir: &Path) -> Vec<PathBuf> {
    if !tasks_dir.is_dir() {
        return Vec::new();
    }
    let mut paths = Vec::new();
    collect_task_files_recursive(tasks_dir, &mut paths);
    paths.sort_by(|a, b| {
        let num_a = extract_task_number(a);
        let num_b = extract_task_number(b);
        match (num_a, num_b) {
            (Some(na), Some(nb)) => na.cmp(&nb),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| {
            let stem_a = a.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let stem_b = b.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            stem_a.cmp(stem_b)
        })
        .then_with(|| a.cmp(b))
    });
    paths
}

/// Resolves the path of the highest-numbered active task across `tasks/**`.
pub fn resolve_active_task_path(workspace: &Path) -> Option<PathBuf> {
    let tasks_dir = workspace.join("tasks");
    let paths = collect_task_files(&tasks_dir);

    for path in paths.into_iter().rev() {
        if let Ok(content) = fs::read_to_string(&path) {
            if is_task_active(&content) {
                return Some(path);
            }
        }
    }

    None
}

/// Resolves the path of the highest-numbered done task across `tasks/**`.
pub fn resolve_latest_done_task_path(workspace: &Path) -> Option<PathBuf> {
    let tasks_dir = workspace.join("tasks");
    let paths = collect_task_files(&tasks_dir);

    for path in paths.into_iter().rev() {
        if let Ok(content) = fs::read_to_string(&path) {
            let status = parse_task_status(&content);
            if matches!(status, TaskStatus::Done) {
                return Some(path);
            }
        }
    }

    None
}

/// Resolves a task file path across `tasks/**` by explicit identifier/path or active task fallback.
///
/// Prioritizes exact stem/filename matches, then `{id}-` prefix matches, then substring matches,
/// preventing e.g. `001` from accidentally matching `1001-...md`.
pub fn resolve_task_file_path(
    workspace: &Path,
    explicit_task_id: Option<&str>,
) -> Result<PathBuf, TaskError> {
    let tasks_dir = workspace.join("tasks");

    if let Some(id_or_path) = explicit_task_id {
        let direct = PathBuf::from(id_or_path);
        if direct.is_file() {
            return Ok(direct);
        }
        let joined = workspace.join(id_or_path);
        if joined.is_file() {
            return Ok(joined);
        }
        let in_tasks = tasks_dir.join(id_or_path);
        if in_tasks.is_file() {
            return Ok(in_tasks);
        }
        let in_tasks_md = tasks_dir.join(format!("{id_or_path}.md"));
        if in_tasks_md.is_file() {
            return Ok(in_tasks_md);
        }

        if tasks_dir.is_dir() {
            let files = collect_task_files(&tasks_dir);
            let prefix = format!("{id_or_path}-");

            if let Some(found) = files.iter().find(|p| {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                stem == id_or_path || name == id_or_path
            }) {
                return Ok(found.clone());
            }

            if let Some(found) = files.iter().find(|p| {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                stem.starts_with(&prefix) || name.starts_with(&prefix)
            }) {
                return Ok(found.clone());
            }

            if let Some(found) = files.iter().find(|p| {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                stem.contains(id_or_path)
            }) {
                return Ok(found.clone());
            }
        }

        return Err(TaskError::TaskNotFound(id_or_path.to_string()));
    }

    if let Some(active_path) = resolve_active_task_path(workspace) {
        return Ok(active_path);
    }

    Err(TaskError::TaskNotFound(
        "no active task found in tasks/".to_string(),
    ))
}

/// Determines whether the workspace has at least one active task in `tasks/**`.
pub fn has_active_task(workspace: &Path) -> bool {
    resolve_active_task_path(workspace).is_some()
}

/// Resolves the task identifier of the highest-numbered active task found in `tasks/**`.
pub fn resolve_active_task_id(workspace: &Path) -> Option<String> {
    resolve_active_task_path(workspace).and_then(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    })
}

/// Resolves the task identifier of the latest done task found in `tasks/**` (highest numbered).
pub fn resolve_latest_done_task_id(workspace: &Path) -> Option<String> {
    resolve_latest_done_task_path(workspace).and_then(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    })
}

/// Parses complete `TaskPackageInfo` from raw markdown content and a designated task identifier.
pub fn parse_task_content(content: &str, task_id: &str) -> Result<TaskPackageInfo, TaskError> {
    let (metadata, body) = parse_frontmatter(content)?;

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

    if intent.is_none() {
        intent = metadata.as_ref().and_then(|m| m.intent.clone());
    }
    let scope = metadata
        .as_ref()
        .map(|m| m.scope.clone())
        .unwrap_or_default();
    let test_refs = metadata
        .as_ref()
        .map(|m| m.test_refs.clone())
        .unwrap_or_default();
    let domain = metadata.as_ref().and_then(|m| m.domain.clone());
    let slice = metadata.as_ref().and_then(|m| m.slice.clone());
    let spec_ref = metadata.as_ref().and_then(|m| m.spec_ref.clone());

    Ok(TaskPackageInfo {
        task_id: task_id.to_string(),
        title,
        status,
        intent,
        purpose,
        acceptance_criteria,
        unresolved_criteria,
        must_not,
        scope,
        test_refs,
        domain,
        slice,
        spec_ref,
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

    let file_to_read = match explicit_task_id {
        Some(task_id) => resolve_task_file_path(workspace, Some(task_id))?,
        None => resolve_active_task_path(workspace)
            .or_else(|| resolve_latest_done_task_path(workspace))
            .ok_or_else(|| TaskError::TaskNotFound("no active task found in tasks/".to_string()))?,
    };

    let info = parse_task_file(&file_to_read)?;
    Ok(TaskContract {
        task_id: info.task_id,
        task_title: info.title,
        task_digest: String::new(),
        acceptance_criteria: info.acceptance_criteria,
        unresolved_criteria: info.unresolved_criteria,
    })
}
