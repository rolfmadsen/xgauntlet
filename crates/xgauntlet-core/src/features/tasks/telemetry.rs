//! Task telemetry, criteria progress calculation, and VCS inspection engine.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::features::tasks::models::TaskStatus;
use crate::features::tasks::parser::{parse_task_file, resolve_active_task_id, TaskError};

/// Lightweight local Git status telemetry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitTelemetry {
    pub branch: String,
    pub head_oid: String,
    pub dirty_count: usize,
    pub is_clean: bool,
}

/// Acceptance criteria progress breakdown and visual bar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriteriaProgress {
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
    pub percentage: u8,
    pub bar: String,
}

/// Full runtime telemetry for an individual task package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskTelemetry {
    pub task_id: String,
    pub title: String,
    pub status: TaskStatus,
    pub intent: Option<String>,
    pub criteria: CriteriaProgress,
    pub git: GitTelemetry,
    pub file_path: String,
}

/// Summary item representing a task package in task list overviews.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSummaryItem {
    pub task_id: String,
    pub task_number: Option<u32>,
    pub title: String,
    pub status: TaskStatus,
    pub intent: Option<String>,
    pub criteria: CriteriaProgress,
    pub file_path: String,
}

/// Parses criteria progress from task markdown content.
pub fn parse_criteria_progress(content: &str) -> CriteriaProgress {
    let mut completed: usize = 0;
    let mut pending: usize = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            completed += 1;
        } else if trimmed.starts_with("- [ ] ") {
            pending += 1;
        }
    }

    let total = completed + pending;
    let percentage = (completed * 100).checked_div(total).unwrap_or(0) as u8;

    let filled = (completed * 5 + total / 2)
        .checked_div(total)
        .map(|f| std::cmp::min(5, f))
        .unwrap_or(0);
    let empty = 5 - filled;
    let bar = format!("[{}{}]", "■".repeat(filled), "□".repeat(empty));

    CriteriaProgress {
        total,
        completed,
        pending,
        percentage,
        bar,
    }
}

/// Collects local git telemetry with graceful fallback if outside a git repository.
pub fn collect_git_telemetry(workspace: &Path) -> GitTelemetry {
    let branch_cmd = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output();

    let (branch, is_repo) = match branch_cmd {
        Ok(out) if out.status.success() => {
            let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let branch_name = if b.is_empty() { "HEAD".to_string() } else { b };
            (branch_name, true)
        }
        _ => ("unknown".to_string(), false),
    };

    if !is_repo {
        return GitTelemetry {
            branch: "unknown".to_string(),
            head_oid: "unknown".to_string(),
            dirty_count: 0,
            is_clean: true,
        };
    }

    let oid_cmd = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output();

    let head_oid = match oid_cmd {
        Ok(out) if out.status.success() => {
            let o = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if o.is_empty() {
                "unknown".to_string()
            } else {
                o
            }
        }
        _ => "unknown".to_string(),
    };

    let status_cmd = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("status")
        .arg("--porcelain")
        .output();

    let dirty_count = match status_cmd {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count(),
        _ => 0,
    };

    GitTelemetry {
        branch,
        head_oid,
        dirty_count,
        is_clean: dirty_count == 0,
    }
}

/// Resolves task file path from explicit identifier or active task.
fn resolve_task_path(
    workspace: &Path,
    task_id_or_path: Option<&str>,
) -> Result<PathBuf, TaskError> {
    let tasks_dir = workspace.join("tasks");

    if let Some(id_or_path) = task_id_or_path {
        // 1. Direct path check
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

        // 2. Scan tasks dir for matching prefix or substring
        if tasks_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&tasks_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().is_some_and(|ext| ext == "md") {
                        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        if stem == id_or_path
                            || stem.starts_with(&format!("{id_or_path}-"))
                            || stem.contains(id_or_path)
                        {
                            return Ok(p);
                        }
                    }
                }
            }
        }

        return Err(TaskError::TaskNotFound(id_or_path.to_string()));
    }

    // 3. Fallback to active task
    if let Some(active_id) = resolve_active_task_id(workspace) {
        let p = tasks_dir.join(format!("{active_id}.md"));
        if p.is_file() {
            return Ok(p);
        }
    }

    Err(TaskError::TaskNotFound(
        "no active task found in tasks/".to_string(),
    ))
}

/// Inspects telemetry for an active or explicitly designated task.
pub fn inspect_task_telemetry(
    workspace: &Path,
    task_id_or_path: Option<&str>,
) -> Result<TaskTelemetry, TaskError> {
    let task_path = resolve_task_path(workspace, task_id_or_path)?;
    let content = fs::read_to_string(&task_path)?;
    let info = parse_task_file(&task_path)?;

    let criteria = parse_criteria_progress(&content);
    let git = collect_git_telemetry(workspace);

    let rel_path = match task_path.strip_prefix(workspace) {
        Ok(p) => p.display().to_string(),
        Err(_) => task_path.display().to_string(),
    };

    Ok(TaskTelemetry {
        task_id: info.task_id,
        title: info.title,
        status: info.status,
        intent: info.intent,
        criteria,
        git,
        file_path: rel_path,
    })
}

/// Lists all tasks discovered in `tasks/` directory with progress telemetry.
pub fn list_workspace_tasks(workspace: &Path) -> Result<Vec<TaskSummaryItem>, TaskError> {
    let tasks_dir = workspace.join("tasks");
    if !tasks_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&tasks_dir)?;
    let mut items = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            if let Ok(info) = parse_task_file(&path) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let criteria = parse_criteria_progress(&content);
                    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
                    let task_number = digits.parse::<u32>().ok();

                    let rel_path = match path.strip_prefix(workspace) {
                        Ok(p) => p.display().to_string(),
                        Err(_) => path.display().to_string(),
                    };

                    items.push(TaskSummaryItem {
                        task_id: info.task_id,
                        task_number,
                        title: info.title,
                        status: info.status,
                        intent: info.intent,
                        criteria,
                        file_path: rel_path,
                    });
                }
            }
        }
    }

    // Sort items by task_number (ascending) then task_id
    items.sort_by(|a, b| match (a.task_number, b.task_number) {
        (Some(num_a), Some(num_b)) => num_a.cmp(&num_b).then_with(|| a.task_id.cmp(&b.task_id)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.task_id.cmp(&b.task_id),
    });

    Ok(items)
}
