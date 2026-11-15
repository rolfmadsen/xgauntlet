//! Empirical Cockpit telemetry collection for active and idle workspaces.
//!
//! Computes fast (<5ms) Verification Freshness, Worktree Task Binding,
//! Scope Hygiene, and Git upstream ahead counts without spawning builds or
//! hashing entire source trees.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::features::adapters::workspace::resolve_primary_repo_root;
use crate::features::checkpoint::extract_short_task_id;
use crate::features::evidence::{load_verification_report, CheckStatus, VerificationVerdict};
use crate::features::tasks::models::TaskStatus;
use crate::features::tasks::parser::ACTIVE_TASK_FILE;
use crate::features::tasks::telemetry::{
    inspect_task_telemetry_for_session, list_workspace_tasks, GitTelemetry, TaskTelemetry,
};
use crate::features::tasks::TaskError;

/// Empirical freshness of `verification-report.json` against active task and current HEAD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationFreshness {
    None,
    Pass,
    Fail,
    Stale,
}

impl VerificationFreshness {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Stale => "STALE",
        }
    }
}

/// Summary of the latest Verification Report and check layers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub freshness: VerificationFreshness,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub passed_checks: usize,
    pub total_checks: usize,
    pub failed_checks: usize,
    pub skipped_checks: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_status: Option<String>,
}

/// Checkout topology (primary vs. linked worktree) and local task binding state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeSummary {
    pub is_worktree: bool,
    pub checkout_label: String,
    pub active_task_bound: bool,
}

/// Fast working-tree dirty file breakdown relative to declared task scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeHygieneSummary {
    pub declared_scope: Vec<String>,
    pub dirty_in_scope: usize,
    pub dirty_out_of_scope: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub out_of_scope_sample: Vec<String>,
}

/// Truthful cockpit enrichment attached to `TaskTelemetry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CockpitTelemetry {
    pub verification: VerificationSummary,
    pub worktree: WorktreeSummary,
    pub scope_hygiene: ScopeHygieneSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commits_ahead: Option<usize>,
}

/// Workspace telemetry rendered when no active task exists (`Task: NONE`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdleWorkspaceTelemetry {
    pub git: GitTelemetry,
    pub worktree: WorktreeSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commits_ahead: Option<usize>,
    pub verification: VerificationSummary,
    pub draft_tasks: usize,
    pub done_tasks: usize,
    pub policy_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_draft_id: Option<String>,
}

/// Unified cockpit snapshot supporting both active tasks and idle Read-Only workspaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CockpitSnapshot {
    Active(Box<TaskTelemetry>),
    Idle(Box<IdleWorkspaceTelemetry>),
}

fn parse_porcelain_path(line: &str) -> Option<String> {
    if line.len() < 3 {
        return None;
    }
    let raw = line[3..].trim();
    let candidate = raw.rsplit(" -> ").next().unwrap_or(raw).trim();
    let unquoted = candidate.trim_matches('"').replace('\\', "/");
    if unquoted.is_empty() {
        None
    } else {
        Some(unquoted)
    }
}

/// Collects Git branch, OID, dirty file list, and upstream ahead count in one pass.
pub fn collect_git_details(workspace: &Path) -> (GitTelemetry, Vec<String>, Option<usize>) {
    let branch_cmd = Command::new("git")
        .args([
            "-C",
            &workspace.to_string_lossy(),
            "rev-parse",
            "--abbrev-ref",
            "HEAD",
        ])
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
        return (
            GitTelemetry {
                branch: "unknown".to_string(),
                head_oid: "unknown".to_string(),
                dirty_count: 0,
                is_clean: true,
            },
            Vec::new(),
            None,
        );
    }

    let head_oid = Command::new("git")
        .args([
            "-C",
            &workspace.to_string_lossy(),
            "rev-parse",
            "--short",
            "HEAD",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let dirty_paths: Vec<String> = Command::new("git")
        .args(["-C", &workspace.to_string_lossy(), "status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .filter_map(parse_porcelain_path)
                .collect()
        })
        .unwrap_or_default();

    let dirty_count = dirty_paths.len();

    let commits_ahead = Command::new("git")
        .args([
            "-C",
            &workspace.to_string_lossy(),
            "rev-list",
            "--count",
            "@{upstream}..HEAD",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<usize>()
                .ok()
        });

    (
        GitTelemetry {
            branch,
            head_oid,
            dirty_count,
            is_clean: dirty_count == 0,
        },
        dirty_paths,
        commits_ahead,
    )
}

fn task_ids_equivalent(a: &str, b: &str) -> bool {
    let sa = extract_short_task_id(a);
    let sb = extract_short_task_id(b);
    a.eq_ignore_ascii_case(b) || (!sa.is_empty() && sa.eq_ignore_ascii_case(sb))
}

/// Evaluates `verification-report.json` against task identity, cleanliness, and HEAD OID.
pub fn evaluate_verification_summary(
    workspace: &Path,
    active_task_id: Option<&str>,
    git: &GitTelemetry,
    dirty_paths: &[String],
) -> VerificationSummary {
    let Ok(report) = load_verification_report(workspace) else {
        return VerificationSummary {
            freshness: VerificationFreshness::None,
            reason: Some("no report".to_string()),
            passed_checks: 0,
            total_checks: 0,
            failed_checks: 0,
            skipped_checks: 0,
            mutation_status: None,
        };
    };

    let total_checks = report.checks.len();
    let passed_checks = report
        .checks
        .iter()
        .filter(|c| c.status == CheckStatus::Passed)
        .count();
    let failed_checks = report
        .checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Failed | CheckStatus::Error))
        .count();
    let skipped_checks = report
        .checks
        .iter()
        .filter(|c| c.status == CheckStatus::Skipped)
        .count();
    let mutation_status = report
        .checks
        .iter()
        .find(|c| c.name.contains("mutation"))
        .map(|c| match c.status {
            CheckStatus::Passed => "PASS".to_string(),
            CheckStatus::Failed | CheckStatus::Error => "FAIL".to_string(),
            CheckStatus::Skipped => "skipped".to_string(),
        });

    if let Some(tid) = active_task_id {
        if !report.task_contract.task_id.is_empty()
            && !task_ids_equivalent(&report.task_contract.task_id, tid)
        {
            return VerificationSummary {
                freshness: VerificationFreshness::Stale,
                reason: Some(format!(
                    "task {} != {}",
                    extract_short_task_id(&report.task_contract.task_id),
                    extract_short_task_id(tid)
                )),
                passed_checks,
                total_checks,
                failed_checks,
                skipped_checks,
                mutation_status,
            };
        }
    }

    let has_non_evidence_dirty = if dirty_paths.is_empty() {
        !git.is_clean
    } else {
        dirty_paths
            .iter()
            .any(|p| !is_always_allowed_runtime_file(p))
    };

    if has_non_evidence_dirty {
        return VerificationSummary {
            freshness: VerificationFreshness::Stale,
            reason: Some("dirty tree".to_string()),
            passed_checks,
            total_checks,
            failed_checks,
            skipped_checks,
            mutation_status,
        };
    }

    if let Some(ref vcs) = report.workspace_state.vcs {
        let rep_commit = vcs.commit.trim();
        if !rep_commit.is_empty()
            && rep_commit != "(no git)"
            && git.head_oid != "unknown"
            && !rep_commit.starts_with(&git.head_oid)
            && !git.head_oid.starts_with(rep_commit)
        {
            return VerificationSummary {
                freshness: VerificationFreshness::Stale,
                reason: Some("commit drifted".to_string()),
                passed_checks,
                total_checks,
                failed_checks,
                skipped_checks,
                mutation_status,
            };
        }
    }

    let (freshness, reason) = match report.verdict {
        VerificationVerdict::Passed => (VerificationFreshness::Pass, None),
        VerificationVerdict::Failed
        | VerificationVerdict::Error
        | VerificationVerdict::Partial
        | VerificationVerdict::Incomplete => (
            VerificationFreshness::Fail,
            Some(report.verdict.as_str().to_ascii_lowercase()),
        ),
        VerificationVerdict::Skipped => (VerificationFreshness::Stale, Some("skipped".to_string())),
    };

    VerificationSummary {
        freshness,
        reason,
        passed_checks,
        total_checks,
        failed_checks,
        skipped_checks,
        mutation_status,
    }
}

/// Evaluates whether the workspace is the primary checkout or a linked worktree and checks binding.
pub fn evaluate_worktree_summary(
    workspace: &Path,
    active_task_id: Option<&str>,
) -> WorktreeSummary {
    let canon_ws = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    let is_worktree = resolve_primary_repo_root(workspace)
        .map(|root| root.canonicalize().unwrap_or(root) != canon_ws)
        .unwrap_or(false);

    let checkout_label = if is_worktree {
        let name = workspace
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("worktree");
        format!("wt:{name}")
    } else {
        "primary".to_string()
    };

    let bound_raw = fs::read_to_string(workspace.join(ACTIVE_TASK_FILE))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let active_task_bound = match (bound_raw, active_task_id) {
        (Some(ref bound), Some(tid)) => task_ids_equivalent(bound, tid),
        (Some(_), None) => true,
        _ => false,
    };

    WorktreeSummary {
        is_worktree,
        checkout_label,
        active_task_bound,
    }
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    let p = pattern.trim().replace('\\', "/");
    let path = path.trim().trim_start_matches("./").replace('\\', "/");
    if p == "*" || p == "**" || p == path {
        return true;
    }
    if let Some(prefix) = p.strip_suffix("/**") {
        let pref = prefix.trim_start_matches("./");
        return path == pref || path.starts_with(&format!("{pref}/"));
    }
    if let Some(prefix) = p.strip_suffix("/*") {
        let pref = prefix.trim_start_matches("./");
        if let Some(rest) = path.strip_prefix(&format!("{pref}/")) {
            return !rest.contains('/');
        }
        return false;
    }
    if let Some(suffix) = p.strip_prefix('*') {
        return path.ends_with(suffix);
    }
    path == p.trim_start_matches("./")
}

fn is_always_allowed_runtime_file(path: &str) -> bool {
    let norm = path.trim_start_matches("./");
    norm == "verification-report.json" || norm == "evidence.md" || norm.starts_with(".xgauntlet/")
}

/// Evaluates dirty working-tree files against the task's declared `scope` and `test_refs`.
pub fn evaluate_scope_hygiene(
    scope: &[String],
    test_refs: &[String],
    dirty_paths: &[String],
) -> ScopeHygieneSummary {
    if scope.is_empty() {
        return ScopeHygieneSummary {
            declared_scope: Vec::new(),
            dirty_in_scope: dirty_paths.len(),
            dirty_out_of_scope: 0,
            out_of_scope_sample: Vec::new(),
        };
    }

    let mut dirty_in_scope = 0usize;
    let mut dirty_out_of_scope = 0usize;
    let mut out_of_scope_sample = Vec::new();

    for path in dirty_paths {
        let matches = is_always_allowed_runtime_file(path)
            || scope.iter().any(|pat| path_matches_pattern(path, pat))
            || test_refs.iter().any(|pat| path_matches_pattern(path, pat));
        if matches {
            dirty_in_scope += 1;
        } else {
            dirty_out_of_scope += 1;
            if out_of_scope_sample.len() < 3 {
                out_of_scope_sample.push(path.clone());
            }
        }
    }

    ScopeHygieneSummary {
        declared_scope: scope.to_vec(),
        dirty_in_scope,
        dirty_out_of_scope,
        out_of_scope_sample,
    }
}

/// Formats declared task scope patterns compactly for single-line / box HUD rendering.
pub fn format_compact_scope(scope: &[String]) -> Option<String> {
    if scope.is_empty() {
        return None;
    }
    if scope.len() == 1 {
        return Some(scope[0].clone());
    }
    let joined = scope.join(", ");
    if joined.chars().count() <= 36 {
        Some(joined)
    } else {
        Some(format!("{} (+{})", scope[0], scope.len() - 1))
    }
}

/// Collects honest Read-Only telemetry when no active task exists.
pub fn inspect_idle_workspace_telemetry(workspace: &Path) -> IdleWorkspaceTelemetry {
    let (git, dirty_paths, commits_ahead) = collect_git_details(workspace);
    let verification = evaluate_verification_summary(workspace, None, &git, &dirty_paths);
    let worktree = evaluate_worktree_summary(workspace, None);

    let mut draft_tasks = 0usize;
    let mut done_tasks = 0usize;
    let mut latest_draft_id = None;

    if let Ok(items) = list_workspace_tasks(workspace) {
        for item in &items {
            if matches!(item.status, TaskStatus::Done | TaskStatus::Completed) {
                done_tasks += 1;
            } else if matches!(item.status, TaskStatus::Draft | TaskStatus::Todo) {
                draft_tasks += 1;
                latest_draft_id = Some(extract_short_task_id(&item.task_id).to_string());
            }
        }
    }

    IdleWorkspaceTelemetry {
        git,
        worktree,
        commits_ahead,
        verification,
        draft_tasks,
        done_tasks,
        policy_mode: "Read-Only / Planning".to_string(),
        latest_draft_id,
    }
}

/// Unified workspace cockpit inspection supporting both active tasks and `Task: NONE`.
pub fn inspect_workspace_cockpit(
    workspace: &Path,
    task_id_or_path: Option<&str>,
    session_id: Option<&str>,
) -> Result<CockpitSnapshot, TaskError> {
    match inspect_task_telemetry_for_session(workspace, task_id_or_path, session_id) {
        Ok(telemetry) => Ok(CockpitSnapshot::Active(Box::new(telemetry))),
        Err(TaskError::TaskNotFound(_)) if task_id_or_path.is_none() => Ok(CockpitSnapshot::Idle(
            Box::new(inspect_idle_workspace_telemetry(workspace)),
        )),
        Err(e) => Err(e),
    }
}
