//! Task telemetry, criteria progress calculation, and VCS inspection engine.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::features::tasks::cockpit::{
    collect_git_details, evaluate_scope_hygiene, evaluate_verification_summary,
    evaluate_worktree_summary, format_compact_scope, CockpitTelemetry,
};
use crate::features::tasks::models::TaskStatus;
use crate::features::tasks::parser::{
    collect_task_files, extract_task_number, parse_task_file, resolve_task_file_path, TaskError,
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invariants: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_health: Option<crate::features::telemetry::SessionHealth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cockpit: Option<CockpitTelemetry>,
}

/// Truncates string to at most max_len characters, appending ellipsis if truncated.
fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    let count = s.chars().count();
    if count <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s.chars().take(max_len).collect()
    } else {
        let prefix: String = s.chars().take(max_len - 3).collect();
        format!("{prefix}...")
    }
}

impl TaskTelemetry {
    /// Renders high-density Unicode Box-Drawing Telemetry Card (Variant B).
    ///
    /// Guarantees exact 64 characters visible column width on all 6 lines,
    /// even when fields (branch, scope, task id) exceed standard boundaries.
    pub fn render_box_card(&self) -> String {
        // Line 0: Header border (fixed 64 visible chars)
        let short_id = crate::features::checkpoint::extract_short_task_id(&self.task_id);
        let task_label = if self
            .task_id
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            format!("Task {}", short_id)
        } else {
            self.task_id.clone()
        };
        let label_trunc = truncate_with_ellipsis(&task_label, 42);
        let header_prefix = format!("┌─── xgauntlet: {label_trunc} ");
        let header_dashes = 64usize.saturating_sub(header_prefix.chars().count() + 1);
        let line0 = format!("{header_prefix}{}┐", "─".repeat(header_dashes));

        // Line 1: Status & Scope (Status: ... Scope: ...)
        let status_val = self
            .phase
            .as_deref()
            .unwrap_or_else(|| self.status.as_str());
        let mut status_str = if let Some(ref sh) = self.session_health {
            format!("Status: {status_val} • {}", sh.status.badge())
        } else {
            format!("Status: {status_val}")
        };
        let scope_val = self.scope.as_deref().unwrap_or("unscoped");
        let mut scope_str = format!("Scope: {scope_val}");
        if status_str.chars().count() + scope_str.chars().count() > 59 {
            if scope_str.chars().count() > 28 {
                scope_str = truncate_with_ellipsis(&scope_str, 28);
            }
            let max_status = 59usize.saturating_sub(scope_str.chars().count());
            if status_str.chars().count() > max_status {
                status_str = truncate_with_ellipsis(&status_str, max_status);
            }
        }
        let spaces1 =
            60usize.saturating_sub(status_str.chars().count() + scope_str.chars().count());
        let line1 = format!("│ {status_str}{}{scope_str} │", " ".repeat(spaces1));

        // Line 2: Progress & Invariants
        let total = self.criteria.total;
        let completed = self.criteria.completed;
        let percentage = (completed * 100).checked_div(total).unwrap_or(0) as u8;
        let filled = (completed * 10 + total / 2)
            .checked_div(total)
            .map(|f| std::cmp::min(10, f))
            .unwrap_or(0);
        let empty = 10 - filled;
        let bar_str = format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(empty),
            percentage
        );
        let progress_str = format!("Progress: {bar_str}");
        let invariants_val = self.invariants.as_deref().unwrap_or("unverified");
        let mut invariants_str = format!("Invariants: {invariants_val}");
        if progress_str.chars().count() + invariants_str.chars().count() > 59 {
            let max_invariants = 59usize.saturating_sub(progress_str.chars().count());
            invariants_str = truncate_with_ellipsis(&invariants_str, max_invariants);
        }
        let spaces2 =
            60usize.saturating_sub(progress_str.chars().count() + invariants_str.chars().count());
        let line2 = format!("│ {progress_str}{}{invariants_str} │", " ".repeat(spaces2));

        // Line 3: Git & Evidence
        let git_state = if self.git.is_clean {
            "clean".to_string()
        } else if self.git.dirty_count > 0 {
            format!("dirty: {} files", self.git.dirty_count)
        } else {
            "dirty".to_string()
        };
        let evidence_val = self.evidence.as_deref().unwrap_or("none");
        let mut evidence_str = format!("Evidence: {evidence_val}");
        if evidence_str.chars().count() > 22 {
            evidence_str = truncate_with_ellipsis(&evidence_str, 22);
        }
        let ahead_suffix = self
            .cockpit
            .as_ref()
            .and_then(|c| c.commits_ahead)
            .filter(|&n| n > 0)
            .map(|n| format!(" +{n}"))
            .unwrap_or_default();
        let max_git = 59usize.saturating_sub(evidence_str.chars().count());
        let max_branch = max_git.saturating_sub(
            self.git.head_oid.chars().count()
                + git_state.chars().count()
                + ahead_suffix.chars().count()
                + 10,
        );
        let branch_trunc = truncate_with_ellipsis(&self.git.branch, std::cmp::max(8, max_branch));
        let mut git_str = format!(
            "Git: {}@{} ({git_state}{ahead_suffix})",
            branch_trunc, self.git.head_oid
        );
        if git_str.chars().count() + evidence_str.chars().count() > 59 {
            let avail = 59usize.saturating_sub(evidence_str.chars().count());
            git_str = truncate_with_ellipsis(&git_str, avail);
        }
        let spaces3 =
            60usize.saturating_sub(git_str.chars().count() + evidence_str.chars().count());
        let line3 = format!("│ {git_str}{}{evidence_str} │", " ".repeat(spaces3));

        // Line 4: Ref row with clean terminal-clickable relative paths (POSIX normalized)
        let norm_path = self.file_path.replace('\\', "/");
        let task_ref = if norm_path.len() <= 16 && norm_path.starts_with("tasks/") {
            norm_path
        } else {
            format!("tasks/{}.md", short_id)
        };
        let ref_str = format!("Ref: {} • spec.md • docs/adr/README.md", task_ref);
        let ref_str_trunc = truncate_with_ellipsis(&ref_str, 60);
        let spaces4 = 60usize.saturating_sub(ref_str_trunc.chars().count());
        let line4 = format!("│ {ref_str_trunc}{} │", " ".repeat(spaces4));

        // Line 5: Bottom border (fixed 64 chars)
        let line5 = format!("└{}┘", "─".repeat(62));

        format!("{line0}\n{line1}\n{line2}\n{line3}\n{line4}\n{line5}")
    }

    /// Renders ultra-compact single-line badge telemetry.
    pub fn render_box_compact(&self) -> String {
        let status_val = self
            .phase
            .as_deref()
            .unwrap_or_else(|| self.status.as_str());
        let git_state = if self.git.is_clean { "clean" } else { "dirty" };
        let session_part = if let Some(ref sh) = self.session_health {
            format!(" | Session: {}", sh.status.badge())
        } else {
            String::new()
        };
        format!(
            "[xgauntlet: {}] Status: {}{} | Progress: {}% | Git: {}@{} ({})",
            self.task_id,
            status_val,
            session_part,
            self.criteria.percentage,
            self.git.branch,
            self.git.head_oid,
            git_state
        )
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn with_invariants(mut self, invariants: impl Into<String>) -> Self {
        self.invariants = Some(invariants.into());
        self
    }

    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = Some(evidence.into());
        self
    }

    pub fn with_phase(mut self, phase: impl Into<String>) -> Self {
        self.phase = Some(phase.into());
        self
    }

    pub fn with_session_health(
        mut self,
        session_health: crate::features::telemetry::SessionHealth,
    ) -> Self {
        self.session_health = Some(session_health);
        self
    }

    pub fn with_cockpit(mut self, cockpit: CockpitTelemetry) -> Self {
        self.cockpit = Some(cockpit);
        self
    }
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
    collect_git_details(workspace).0
}

/// Resolves task file path from explicit identifier or active task.
fn resolve_task_path(
    workspace: &Path,
    task_id_or_path: Option<&str>,
) -> Result<PathBuf, TaskError> {
    resolve_task_file_path(workspace, task_id_or_path)
}

/// Infers the active TDD phase by inspecting conventional commit markers for this task.
pub fn infer_current_phase(workspace: &Path, task_id: &str) -> String {
    let short_id = task_id.split('-').next().unwrap_or(task_id);
    let output = std::process::Command::new("git")
        .args(["log", "-n", "20", "--format=%s"])
        .current_dir(workspace)
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let log = String::from_utf8_lossy(&out.stdout);
            for line in log.lines() {
                let trimmed = line.trim();
                if trimmed.contains(&format!("({short_id})"))
                    || trimmed.contains(&format!("({task_id})"))
                    || trimmed.starts_with(&format!("task({short_id})"))
                    || trimmed.starts_with(&format!("test({short_id})"))
                    || trimmed.starts_with(&format!("feat({short_id})"))
                    || trimmed.starts_with(&format!("refactor({short_id})"))
                    || trimmed.starts_with(&format!("chore({short_id})"))
                {
                    if trimmed.contains("[DONE]")
                        || trimmed.starts_with(&format!("chore({short_id})"))
                    {
                        return "DONE".to_string();
                    }
                    if trimmed.contains("[REFACTOR]")
                        || trimmed.starts_with(&format!("refactor({short_id})"))
                    {
                        return "GAUNTLET".to_string();
                    }
                    if trimmed.contains("[GREEN]")
                        || trimmed.starts_with(&format!("feat({short_id})"))
                    {
                        return "REFACTOR".to_string();
                    }
                    if trimmed.contains("[RED]")
                        || trimmed.starts_with(&format!("test({short_id})"))
                    {
                        return "GREEN".to_string();
                    }
                    if trimmed.starts_with(&format!("task({short_id})")) {
                        return "RED".to_string();
                    }
                }
            }
        }
    }

    "SPEC".to_string()
}

/// Inspects telemetry for an active or explicitly designated task, scoped to an optional session.
pub fn inspect_task_telemetry_for_session(
    workspace: &Path,
    task_id_or_path: Option<&str>,
    session_id: Option<&str>,
) -> Result<TaskTelemetry, TaskError> {
    let task_path = resolve_task_path(workspace, task_id_or_path)?;
    let content = fs::read_to_string(&task_path)?;
    let info = parse_task_file(&task_path)?;

    let criteria = parse_criteria_progress(&content);
    let (git, dirty_paths, commits_ahead) = collect_git_details(workspace);

    let rel_path = match task_path.strip_prefix(workspace) {
        Ok(p) => p.display().to_string().replace('\\', "/"),
        Err(_) => task_path.display().to_string().replace('\\', "/"),
    };

    let session_health = match session_id {
        Some(sid) => crate::features::telemetry::SessionTracker::load_for_session(
            workspace,
            &info.task_id,
            Some(sid),
        )
        .ok()
        .flatten(),
        None => crate::features::telemetry::SessionTracker::load(workspace, &info.task_id)
            .ok()
            .flatten()
            .or_else(|| {
                crate::features::telemetry::SessionTracker::load_latest_matching(
                    workspace,
                    &info.task_id,
                )
                .ok()
                .flatten()
            }),
    };

    let phase = infer_current_phase(workspace, &info.task_id);
    let verification =
        evaluate_verification_summary(workspace, Some(&info.task_id), &git, &dirty_paths);
    let worktree = evaluate_worktree_summary(workspace, Some(&info.task_id));
    let scope_hygiene = evaluate_scope_hygiene(&info.scope, &info.test_refs, &dirty_paths);

    let scope = format_compact_scope(&info.scope);
    let invariants = if verification.total_checks > 0 {
        Some(format!(
            "{}/{} {}",
            verification.passed_checks,
            verification.total_checks,
            verification.freshness.as_str()
        ))
    } else {
        None
    };
    let evidence = Some(
        match verification.freshness {
            crate::features::tasks::cockpit::VerificationFreshness::Pass => "sealed",
            crate::features::tasks::cockpit::VerificationFreshness::Fail => "failed",
            crate::features::tasks::cockpit::VerificationFreshness::Stale => "stale",
            crate::features::tasks::cockpit::VerificationFreshness::None => "none",
        }
        .to_string(),
    );

    let cockpit = CockpitTelemetry {
        verification,
        worktree,
        scope_hygiene,
        commits_ahead,
    };

    Ok(TaskTelemetry {
        task_id: info.task_id,
        title: info.title,
        status: info.status,
        intent: info.intent,
        criteria,
        git,
        file_path: rel_path,
        scope,
        invariants,
        evidence,
        phase: Some(phase),
        session_health,
        cockpit: Some(cockpit),
    })
}

/// Inspects telemetry for an active or explicitly designated task.
pub fn inspect_task_telemetry(
    workspace: &Path,
    task_id_or_path: Option<&str>,
) -> Result<TaskTelemetry, TaskError> {
    inspect_task_telemetry_for_session(workspace, task_id_or_path, None)
}

/// Lists all tasks discovered in `tasks/**` directory tree with progress telemetry.
pub fn list_workspace_tasks(workspace: &Path) -> Result<Vec<TaskSummaryItem>, TaskError> {
    let tasks_dir = workspace.join("tasks");
    if !tasks_dir.is_dir() {
        return Ok(Vec::new());
    }

    let paths = collect_task_files(&tasks_dir);
    let mut items = Vec::new();

    for path in paths {
        if let Ok(info) = parse_task_file(&path) {
            if let Ok(content) = fs::read_to_string(&path) {
                let criteria = parse_criteria_progress(&content);
                let task_number = extract_task_number(&path);

                let rel_path = match path.strip_prefix(workspace) {
                    Ok(p) => p.display().to_string().replace('\\', "/"),
                    Err(_) => path.display().to_string().replace('\\', "/"),
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

    // Sort items by task_number (ascending) then task_id
    items.sort_by(|a, b| match (a.task_number, b.task_number) {
        (Some(num_a), Some(num_b)) => num_a.cmp(&num_b).then_with(|| a.task_id.cmp(&b.task_id)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.task_id.cmp(&b.task_id),
    });

    Ok(items)
}
