//! Task telemetry, criteria progress calculation, and VCS inspection engine.

use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::features::tasks::models::TaskStatus;
use crate::features::tasks::parser::TaskError;

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
pub fn parse_criteria_progress(_content: &str) -> CriteriaProgress {
    todo!("parse_criteria_progress not yet implemented")
}

/// Collects local git telemetry with graceful fallback if outside a git repository.
pub fn collect_git_telemetry(_workspace: &Path) -> GitTelemetry {
    todo!("collect_git_telemetry not yet implemented")
}

/// Inspects telemetry for an active or explicitly designated task.
pub fn inspect_task_telemetry(
    _workspace: &Path,
    _task_id_or_path: Option<&str>,
) -> Result<TaskTelemetry, TaskError> {
    todo!("inspect_task_telemetry not yet implemented")
}

/// Lists all tasks discovered in `tasks/` directory with progress telemetry.
pub fn list_workspace_tasks(_workspace: &Path) -> Result<Vec<TaskSummaryItem>, TaskError> {
    todo!("list_workspace_tasks not yet implemented")
}
