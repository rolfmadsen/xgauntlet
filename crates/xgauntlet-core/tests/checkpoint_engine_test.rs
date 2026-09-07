//! Integration tests for Phase-Bound TDD Checkpoint Engine (Task 014 jf. ADR 0001, ADR 0003).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xgauntlet_core::features::checkpoint::{
    compose_commit_message, extract_short_task_id, run_checkpoint, CheckpointError,
    CheckpointOptions, CheckpointPhase,
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

fn init_git_repo(path: &Path) {
    let _ = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output()
        .or_else(|_| {
            Command::new("git")
                .args(["init"])
                .current_dir(path)
                .output()
        });

    let _ = Command::new("git")
        .args(["config", "user.name", "Test Committer"])
        .current_dir(path)
        .output();

    let _ = Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output();
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

**Phase Checkpoint**:
A verified local Git commit.
_Avoid_: Quick save, commit hook.
"#;
    fs::write(workspace.join("CONTEXT.md"), glossary).unwrap();
}

fn setup_mock_task(workspace: &Path, task_id: &str, status: &str) {
    let tasks_dir = workspace.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let content = format!(
        r#"---
type: Task Package
title: "Task {task_id}: Test Task"
description: "Test task description"
status: {status}
---

# Task {task_id}: Test Task

**Status**: `{status}`

## 🎯 Formål
Test task purpose.

## 📋 Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## 🚫 Must NOT
- Must not violate invariants.
"#
    );
    fs::write(tasks_dir.join(format!("{task_id}.md")), content).unwrap();
}

#[test]
fn test_models_and_conventions() {
    assert_eq!(CheckpointPhase::Spec.as_str(), "spec");
    assert_eq!(CheckpointPhase::Red.as_str(), "red");
    assert_eq!(CheckpointPhase::Green.as_str(), "green");
    assert_eq!(CheckpointPhase::Refactor.as_str(), "refactor");
    assert_eq!(CheckpointPhase::Done.as_str(), "done");

    assert_eq!(CheckpointPhase::Spec.conventional_prefix(), "task");
    assert_eq!(CheckpointPhase::Red.conventional_prefix(), "test");
    assert_eq!(CheckpointPhase::Green.conventional_prefix(), "feat");
    assert_eq!(CheckpointPhase::Refactor.conventional_prefix(), "refactor");
    assert_eq!(CheckpointPhase::Done.conventional_prefix(), "chore");

    assert_eq!(CheckpointPhase::Spec.phase_tag(), None);
    assert_eq!(CheckpointPhase::Red.phase_tag(), Some("[RED]"));
    assert_eq!(CheckpointPhase::Green.phase_tag(), Some("[GREEN]"));
    assert_eq!(CheckpointPhase::Refactor.phase_tag(), Some("[REFACTOR]"));
    assert_eq!(CheckpointPhase::Done.phase_tag(), Some("[DONE]"));

    assert_eq!(
        "red".parse::<CheckpointPhase>().unwrap(),
        CheckpointPhase::Red
    );
    assert_eq!(
        "GREEN".parse::<CheckpointPhase>().unwrap(),
        CheckpointPhase::Green
    );
    assert!("invalid_phase".parse::<CheckpointPhase>().is_err());
}

#[test]
fn test_compose_commit_message_variations() {
    let task_id = "014-phase-checkpoint-engine";
    assert_eq!(extract_short_task_id(task_id), "014");

    // Default messages
    let msg_red = compose_commit_message(CheckpointPhase::Red, task_id, None);
    assert_eq!(msg_red, "test(014): add failing acceptance test [RED]");

    let msg_green = compose_commit_message(CheckpointPhase::Green, task_id, None);
    assert_eq!(
        msg_green,
        "feat(014): implement minimal logic to satisfy test [GREEN]"
    );

    let msg_refactor = compose_commit_message(CheckpointPhase::Refactor, task_id, None);
    assert_eq!(
        msg_refactor,
        "refactor(014): clean up module boundaries and types [REFACTOR]"
    );

    let msg_done = compose_commit_message(CheckpointPhase::Done, task_id, None);
    assert_eq!(msg_done, "chore(014): seal evidence and mark task DONE");

    let msg_spec = compose_commit_message(CheckpointPhase::Spec, task_id, None);
    assert_eq!(
        msg_spec,
        "task(014): initialize task specification and criteria"
    );

    // Custom messages with automatic prefix/tag wrapping
    let custom_red = compose_commit_message(
        CheckpointPhase::Red,
        task_id,
        Some("verify red phase invariant"),
    );
    assert_eq!(custom_red, "test(014): verify red phase invariant [RED]");

    // Custom message already formatted - should not duplicate tag or prefix
    let already_tagged = compose_commit_message(
        CheckpointPhase::Green,
        task_id,
        Some("feat(014): add support for skip verify [GREEN]"),
    );
    assert_eq!(
        already_tagged,
        "feat(014): add support for skip verify [GREEN]"
    );
}

#[tokio::test]
async fn test_checkpoint_fails_without_active_task() {
    let temp = TempDir::new("no_task_ws");
    init_git_repo(temp.path());

    let opts = CheckpointOptions::new(CheckpointPhase::Spec, temp.path());
    let res = run_checkpoint(&opts).await;
    match res {
        Err(CheckpointError::NoActiveTask) => {}
        other => panic!("Expected NoActiveTask error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_spec_phase_checkpoint() {
    let temp = TempDir::new("spec_phase_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "014-phase-checkpoint-engine", "active");

    let opts = CheckpointOptions::new(CheckpointPhase::Spec, temp.path());
    let res = run_checkpoint(&opts)
        .await
        .expect("Spec checkpoint should succeed for valid task");

    assert_eq!(res.phase, CheckpointPhase::Spec);
    assert_eq!(res.task_id, "014-phase-checkpoint-engine");
    assert!(res.commit_oid.is_some());
    assert!(res.commit_message.contains("task(014):"));
}

#[tokio::test]
async fn test_red_phase_fails_when_test_passes() {
    let temp = TempDir::new("red_phase_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "014-phase-checkpoint-engine", "active");

    // Configure a gauntlet.toml whose test command succeeds (exit 0)
    let config_content = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["true"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), config_content).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Red, temp.path());
    let res = run_checkpoint(&opts).await;

    // Must fail because tests PASSED during RED phase!
    match res {
        Err(CheckpointError::RedPhaseUnmet { details }) => {
            assert!(
                details.contains("Expected failing test assertions in RED phase"),
                "Details should mention unexpected passing tests: {details}"
            );
        }
        other => panic!("Expected RedPhaseUnmet error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_red_phase_succeeds_when_test_fails() {
    let temp = TempDir::new("red_success_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "014-phase-checkpoint-engine", "active");

    // Configure a gauntlet.toml whose test command fails (exit 1)
    let config_content = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["false"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), config_content).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Red, temp.path())
        .with_message("acceptance tests fail as expected");
    let res = run_checkpoint(&opts)
        .await
        .expect("RED phase checkpoint must succeed when test assertions fail");

    assert_eq!(res.phase, CheckpointPhase::Red);
    assert!(res.commit_oid.is_some());
    assert!(res.commit_message.contains("test(014):"));
    assert!(res.commit_message.ends_with("[RED]"));
}

#[tokio::test]
async fn test_green_phase_fails_when_test_fails() {
    let temp = TempDir::new("green_fail_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "014-phase-checkpoint-engine", "active");

    // Test command fails (exit 1)
    let config_content = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["false"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), config_content).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Green, temp.path());
    let res = run_checkpoint(&opts).await;

    match res {
        Err(CheckpointError::GreenPhaseUnmet { .. }) => {}
        other => panic!("Expected GreenPhaseUnmet error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_green_phase_succeeds_when_test_passes() {
    let temp = TempDir::new("green_success_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "014-phase-checkpoint-engine", "active");

    // Test command passes (exit 0)
    let config_content = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["true"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), config_content).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Green, temp.path())
        .with_message("implement minimal logic to satisfy test");
    let res = run_checkpoint(&opts)
        .await
        .expect("GREEN phase checkpoint must succeed when tests pass");

    assert_eq!(res.phase, CheckpointPhase::Green);
    assert!(res.commit_oid.is_some());
    assert!(res.commit_message.contains("feat(014):"));
    assert!(res.commit_message.ends_with("[GREEN]"));
}

#[tokio::test]
async fn test_refactor_phase_succeeds_when_green() {
    let temp = TempDir::new("refactor_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "014-phase-checkpoint-engine", "active");

    let config_content = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["true"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), config_content).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Refactor, temp.path());
    let res = run_checkpoint(&opts)
        .await
        .expect("REFACTOR phase checkpoint must succeed when tests pass");

    assert_eq!(res.phase, CheckpointPhase::Refactor);
    assert!(res.commit_oid.is_some());
    assert!(res.commit_message.contains("refactor(014):"));
    assert!(res.commit_message.ends_with("[REFACTOR]"));
}

#[tokio::test]
async fn test_skip_verify_override_bypasses_preflight() {
    let temp = TempDir::new("skip_verify_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "014-phase-checkpoint-engine", "active");

    // In RED phase, a passing test would normally fail preflight.
    // With --skip-verify, it must proceed and commit!
    let config_content = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["true"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), config_content).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Red, temp.path())
        .with_skip_verify(true)
        .with_message("emergency override bypass");

    let res = run_checkpoint(&opts)
        .await
        .expect("Checkpoint with skip_verify must succeed without executing preflight");

    assert!(res.commit_oid.is_some());
    assert!(res.commit_message.contains("emergency override bypass"));
}

#[tokio::test]
async fn test_done_phase_succeeds_and_creates_chore_commit() {
    let temp = TempDir::new("done_phase_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());

    // Setup completed task with all criteria resolved
    let tasks_dir = temp.path().join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let content = r#"---
type: Task Package
title: "Task 014: Test Task"
description: "Test task description"
status: active
---

# Task 014: Test Task

**Status**: `ACTIVE`

## 🎯 Formål
Test task purpose.

## 📋 Acceptance Criteria
- [x] Criterion 1
- [x] Criterion 2

## 🚫 Must NOT
- Must not violate invariants.
"#;
    fs::write(tasks_dir.join("014-phase-checkpoint-engine.md"), content).unwrap();

    let config_content = r#"
stack = "rust"
save_evidence = true

[[layers]]
name = "unit"
command = ["true"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), config_content).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Done, temp.path());
    let res = run_checkpoint(&opts)
        .await
        .expect("DONE phase checkpoint must succeed when gauntlet passes");

    assert_eq!(res.phase, CheckpointPhase::Done);
    assert!(res.commit_oid.is_some());
    assert!(res.commit_message.contains("chore(014):"));
    assert!(res.commit_message.contains("DONE"));
    assert!(temp.path().join("verification-report.json").is_file());
    assert!(temp.path().join("evidence.md").is_file());
}

fn find_xgauntlet_binary() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../../target/debug/xgauntlet"),
        manifest_dir.join("../../target/release/xgauntlet"),
        manifest_dir.join("../../target/debug/xgauntlet.exe"),
        manifest_dir.join("../../target/release/xgauntlet.exe"),
    ];
    for c in candidates {
        if c.is_file() {
            return c;
        }
    }
    PathBuf::from("xgauntlet")
}

#[test]
fn test_cli_checkpoint_red_and_green_invariants() {
    let bin_path = find_xgauntlet_binary();
    if !bin_path.is_file() {
        return;
    }

    let temp = TempDir::new("cli_checkpoint_test");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "014-phase-checkpoint-engine", "active");

    // 1. Tests pass (exit 0) -> --phase red MUST FAIL with exit code 1
    let passing_config = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["true"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), passing_config).unwrap();

    let output_red_fail = Command::new(&bin_path)
        .args([
            "checkpoint",
            "--phase",
            "red",
            "--workspace",
            temp.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute xgauntlet checkpoint");

    assert!(!output_red_fail.status.success());
    let stderr_red = String::from_utf8_lossy(&output_red_fail.stderr);
    assert!(stderr_red.contains("RED phase unmet"));

    // 2. Tests pass -> --phase green MUST SUCCEED with exit code 0 and output JSON
    let output_green_ok = Command::new(&bin_path)
        .args([
            "checkpoint",
            "--phase",
            "green",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("failed to execute xgauntlet checkpoint");

    assert!(output_green_ok.status.success());
    let stdout_green = String::from_utf8_lossy(&output_green_ok.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout_green)
        .expect("output must be valid JSON");
    assert_eq!(parsed["phase"], "green");
    assert_eq!(parsed["task_id"], "014-phase-checkpoint-engine");
    assert!(parsed["commit_oid"].is_string());

    // 3. Tests fail (exit 1) -> --phase green MUST FAIL with exit code 1
    let failing_config = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["false"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), failing_config).unwrap();

    let output_green_fail = Command::new(&bin_path)
        .args([
            "checkpoint",
            "--phase",
            "green",
            "--workspace",
            temp.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute xgauntlet checkpoint");

    assert!(!output_green_fail.status.success());
    let stderr_green = String::from_utf8_lossy(&output_green_fail.stderr);
    assert!(stderr_green.contains("GREEN phase unmet"));
}
