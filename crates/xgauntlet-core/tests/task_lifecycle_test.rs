//! Integration tests for Task Lifecycle Engine, Intent Scaffolding,
//! and Task Telemetry (Task 013).

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::tasks::{
    check_task_specification, collect_git_telemetry, inspect_task_telemetry, list_workspace_tasks,
    parse_criteria_progress, ScaffoldTaskOptions, TaskScaffolder, TaskStatus,
};

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let unique = format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            count,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(format!("xgauntlet_test_{}", unique));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn setup_mock_glossary(workspace: &Path) {
    let glossary = r#"---
type: Knowledge Bundle Index
title: "xGauntlet Context & Domain Glossary"
description: "Test glossary"
status: stable
---

# xGauntlet Context & Domain Glossary

**Task**:
An executable unit of engineering work.
_Avoid_: Ticket, issue.

**Gauntlet**:
A sequential verification pipeline.
_Avoid_: Test runner.
"#;
    fs::write(workspace.join("CONTEXT.md"), glossary).unwrap();
}

#[test]
fn test_detect_next_task_number() {
    let temp = TempDir::new("task_num_detect");
    let tasks_dir = temp.path().join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();

    // 1. Empty tasks dir -> should detect 1
    assert_eq!(TaskScaffolder::detect_next_task_number(&tasks_dir), 1);

    // 2. Add some task files
    fs::write(tasks_dir.join("001-core-policy.md"), "# Task 1").unwrap();
    fs::write(tasks_dir.join("002-manifest.md"), "# Task 2").unwrap();
    fs::write(tasks_dir.join("README.md"), "# Readme").unwrap(); // ignore non-numeric

    assert_eq!(TaskScaffolder::detect_next_task_number(&tasks_dir), 3);

    // 3. Add higher task number
    fs::write(tasks_dir.join("014-dx-enhancements.md"), "# Task 14").unwrap();
    assert_eq!(TaskScaffolder::detect_next_task_number(&tasks_dir), 15);
}

#[test]
fn test_slugify_and_formatting() {
    assert_eq!(TaskScaffolder::format_task_number(1), "001");
    assert_eq!(TaskScaffolder::format_task_number(14), "014");
    assert_eq!(TaskScaffolder::format_task_number(125), "125");

    assert_eq!(
        TaskScaffolder::slugify("Dynamic HUD & Telemetry!"),
        "dynamic-hud-telemetry"
    );
    assert_eq!(
        TaskScaffolder::slugify("task_lifecycle_engine"),
        "task-lifecycle-engine"
    );

    assert_eq!(
        TaskScaffolder::format_intent(Some("feature")),
        "🚀 NEW FEATURE"
    );
    assert_eq!(TaskScaffolder::format_intent(Some("bug")), "🐛 BUG FIX");
    assert_eq!(
        TaskScaffolder::format_intent(Some("refactor")),
        "🔄 REFACTOR"
    );
    assert_eq!(TaskScaffolder::format_intent(None), "🚀 NEW FEATURE");
}

#[test]
fn test_scaffold_task_package_creates_valid_spec() {
    let temp = TempDir::new("task_scaffold_valid");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let options = ScaffoldTaskOptions {
        name: "intent-telemetry-engine".to_string(),
        title: Some("Intent Telemetry Engine".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Etablere telemetri- og intent-scaffolding motor jf. spec.md".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };

    let result = TaskScaffolder::scaffold(&options).expect("scaffold must succeed");
    assert_eq!(result.task_number, 1);
    assert_eq!(result.task_id, "001-intent-telemetry-engine");
    assert!(result.created);

    let created_file = ws.join("tasks/001-intent-telemetry-engine.md");
    assert!(created_file.is_file(), "Task file must exist on disk");

    let content = fs::read_to_string(&created_file).unwrap();
    assert!(content.contains("type: Task Package"));
    assert!(content.contains("Task 001: Intent Telemetry Engine"));
    assert!(content.contains("**Intent**: `🚀 NEW FEATURE`"));
    assert!(content.contains("## 🎯 Formål"));
    assert!(content.contains("## 📋 Acceptance Criteria"));
    assert!(content.contains("## 🚫 Must NOT"));
    assert!(content.contains("## 📝 Revisions"));
    assert!(content.contains("## 🧪 Verifikation"));

    // Check that check-spec validates this scaffolded task with 0 errors!
    let report = check_task_specification(&created_file, ws);
    assert!(
        report.is_valid,
        "Scaffolded task must pass check_task_specification: {:?}",
        report.diagnostics
    );
}

#[test]
fn test_scaffold_collision_fail_closed_without_force() {
    let temp = TempDir::new("task_scaffold_collision");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let options = ScaffoldTaskOptions {
        name: "duplicate-task".to_string(),
        title: None,
        intent: None,
        purpose: None,
        workspace: ws.to_path_buf(),
        force: false,
    };

    // First scaffold succeeds
    TaskScaffolder::scaffold(&options).expect("first scaffold must succeed");

    // Second scaffold for existing file with force: false must fail closed
    let err = TaskScaffolder::scaffold(&options);
    assert!(
        err.is_err(),
        "Must fail closed when target task already exists and force is false"
    );

    // With force: true, it overwrites
    let force_options = ScaffoldTaskOptions {
        force: true,
        ..options
    };
    let res = TaskScaffolder::scaffold(&force_options);
    assert!(res.is_ok(), "Must succeed with force: true");
}

#[test]
fn test_parse_criteria_progress() {
    let mixed_content = r#"
# Task 099: Test
## 📋 Acceptance Criteria
- [x] First completed criteria
- [X] Second completed criteria
- [ ] Third pending criteria
- [x] Fourth completed criteria
- [ ] Fifth pending criteria
"#;

    let progress = parse_criteria_progress(mixed_content);
    assert_eq!(progress.total, 5);
    assert_eq!(progress.completed, 3);
    assert_eq!(progress.pending, 2);
    assert_eq!(progress.percentage, 60);
    assert_eq!(progress.bar, "[■■■□□]");

    let empty_content = "# Task without criteria\n";
    let empty_progress = parse_criteria_progress(empty_content);
    assert_eq!(empty_progress.total, 0);
    assert_eq!(empty_progress.completed, 0);
    assert_eq!(empty_progress.pending, 0);
    assert_eq!(empty_progress.percentage, 0);
    assert_eq!(empty_progress.bar, "[□□□□□]");

    let full_content = r#"
- [x] A
- [x] B
- [x] C
- [x] D
"#;
    let full_progress = parse_criteria_progress(full_content);
    assert_eq!(full_progress.total, 4);
    assert_eq!(full_progress.completed, 4);
    assert_eq!(full_progress.pending, 0);
    assert_eq!(full_progress.percentage, 100);
    assert_eq!(full_progress.bar, "[■■■■■]");
}

#[test]
fn test_git_telemetry_graceful_fallback() {
    let temp = TempDir::new("non_git_ws");
    let ws = temp.path();

    // Ensure directory is definitely not a git repository
    let telemetry = collect_git_telemetry(ws);
    assert_eq!(telemetry.branch, "unknown");
    assert_eq!(telemetry.head_oid, "unknown");
    assert_eq!(telemetry.dirty_count, 0);
    assert!(telemetry.is_clean);
}

#[test]
fn test_inspect_and_list_workspace_tasks() {
    let temp = TempDir::new("task_inspect_list");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let opt1 = ScaffoldTaskOptions {
        name: "alpha-feature".to_string(),
        title: Some("Alpha Feature".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Alpha purpose".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    let opt2 = ScaffoldTaskOptions {
        name: "beta-fix".to_string(),
        title: Some("Beta Fix".to_string()),
        intent: Some("bug".to_string()),
        purpose: Some("Beta purpose".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };

    TaskScaffolder::scaffold(&opt1).unwrap();
    TaskScaffolder::scaffold(&opt2).unwrap();

    let tasks = list_workspace_tasks(ws).expect("list tasks must succeed");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].task_id, "001-alpha-feature");
    assert_eq!(tasks[0].task_number, Some(1));
    assert_eq!(tasks[0].title, "Task 001: Alpha Feature");
    assert_eq!(tasks[0].status, TaskStatus::Active);

    assert_eq!(tasks[1].task_id, "002-beta-fix");
    assert_eq!(tasks[1].task_number, Some(2));
    assert_eq!(tasks[1].status, TaskStatus::Active);

    // Test inspect_task_telemetry
    let telemetry = inspect_task_telemetry(ws, Some("001")).expect("inspect must succeed");
    assert_eq!(telemetry.task_id, "001-alpha-feature");
    assert_eq!(telemetry.status, TaskStatus::Active);
}
