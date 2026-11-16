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
scope:
  - "crates/**"
test_refs:
  - "tests/**"
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
async fn test_autonomous_red_to_green_transition_checkpoint() {
    let temp = TempDir::new("auto_red_to_green_ws");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(
        temp.path(),
        "030-autonomous-red-to-green-tdd-transition",
        "active",
    );

    let config_content = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["false"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), config_content).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Red, temp.path())
        .with_message("acceptance tests fail as expected [RED]");
    let res = run_checkpoint(&opts)
        .await
        .expect("RED phase checkpoint must succeed when test assertions fail");

    assert_eq!(res.phase, CheckpointPhase::Red);
    assert!(res.autonomous_transition);
    assert_eq!(res.next_phase.as_deref(), Some("green"));

    // Also verify non-RED phase does not trigger autonomous transition
    let green_opts =
        CheckpointOptions::new(CheckpointPhase::Green, temp.path()).with_skip_verify(true);
    let green_res = run_checkpoint(&green_opts)
        .await
        .expect("Green checkpoint with skip_verify must succeed");
    assert!(!green_res.autonomous_transition);
    assert_ne!(green_res.next_phase.as_deref(), Some("green"));
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
scope:
  - "crates/**"
test_refs:
  - "tests/**"
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

    let _ = Command::new("git")
        .args(["commit", "--allow-empty", "-m", "test(014): test [RED]"])
        .current_dir(temp.path())
        .output();

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
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_green).expect("output must be valid JSON");
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

#[tokio::test]
async fn test_task_084_ac2_done_checkpoint_marks_task_done_and_commits() {
    let temp = TempDir::new("task_084_done_transition");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    let tasks_dir = temp.path().join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let task_md = r#"---
type: Task Package
title: "Task 084: Done Transition"
description: "Test DONE transition"
status: active
scope:
  - "crates/**"
test_refs:
  - "tests/**"
---

# Task 084: Done Transition

**Status**: `ACTIVE`

## 🎯 Formål
Test DONE transition.

## 📋 Acceptance Criteria
- [x] Criterion 1
- [x] Criterion 2

## 🚫 Must NOT
- Must not violate invariants.
"#;
    fs::write(tasks_dir.join("084-done-transition.md"), task_md).unwrap();

    let passing_config = r#"
stack = "rust"
save_evidence = true
[[layers]]
name = "unit"
command = ["true"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), passing_config).unwrap();

    // Seed RED commit so DONE phase progression invariant is satisfied
    fs::create_dir_all(temp.path().join("crates")).unwrap();
    fs::write(temp.path().join("crates/seed.txt"), "red").unwrap();
    Command::new("git")
        .args(["add", "-A"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "test(084): add failing test [RED]"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    fs::write(temp.path().join("crates/seed.txt"), "green").unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Done, temp.path());
    let res = run_checkpoint(&opts)
        .await
        .expect("DONE checkpoint should succeed");
    assert!(res.commit_oid.is_some());

    let task_path = temp.path().join("tasks/084-done-transition.md");
    let task_content = fs::read_to_string(&task_path).unwrap();
    assert!(
        task_content.contains("status: done"),
        "Task frontmatter must be updated to status: done, got:\n{task_content}"
    );
    assert!(
        task_content.contains("**Status**: `DONE`"),
        "Task body must be updated to **Status**: `DONE`, got:\n{task_content}"
    );

    // Ensure the task status update was included inside the DONE commit (no uncommitted diff on task file)
    let diff_out = Command::new("git")
        .args([
            "diff",
            "--name-only",
            "HEAD",
            "--",
            "tasks/084-done-transition.md",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&diff_out.stdout).trim().is_empty(),
        "Updated task file must be committed as part of DONE checkpoint"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn test_task_084_ac3_checkpoint_fails_fast_on_readonly_git_dir() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new("task_084_readonly_git");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "084-readonly-git", "active");

    let marker = temp.path().join("verify_ran_marker.txt");
    let marker_str = marker.to_string_lossy().to_string();
    let marker_config = format!(
        r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["touch", "{marker_str}"]
optional = false
"#
    );
    fs::write(temp.path().join("gauntlet.toml"), marker_config).unwrap();

    // Seed initial commit
    Command::new("git")
        .args(["add", "-A"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    fs::write(temp.path().join("change.txt"), "pending").unwrap();

    // Make .git read-only to simulate sandbox .git/index.lock restriction
    let git_dir = temp.path().join(".git");
    fs::set_permissions(&git_dir, fs::Permissions::from_mode(0o555)).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Green, temp.path());
    let err = run_checkpoint(&opts)
        .await
        .expect_err("Checkpoint must fail fast when .git is read-only");

    // Restore permissions before assertions / TempDir cleanup
    let _ = fs::set_permissions(&git_dir, fs::Permissions::from_mode(0o755));

    assert!(
        !marker.exists(),
        "Verification pipeline MUST NOT run when .git/ is read-only (fail-fast check failed)"
    );
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains(".git") || err_msg.contains("read-only") || err_msg.contains("writable"),
        "Expected clear read-only .git error message, got: {err}"
    );
}

#[tokio::test]
async fn test_task_084_ac4_red_preflight_allows_mixed_green_and_red_test_refs() {
    let temp = TempDir::new("task_084_mixed_red_refs");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());

    let cargo_toml = r#"[package]
name = "mixed_red_dummy"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(temp.path().join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(
        temp.path().join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .unwrap();

    fs::create_dir_all(temp.path().join("tests")).unwrap();
    fs::write(
        temp.path().join("tests/existing_green_test.rs"),
        "#[test]\nfn regression_ok() { assert_eq!(mixed_red_dummy::add(1, 2), 3); }\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("tests/new_feature_red_test.rs"),
        "#[test]\nfn expected_red_failure() { assert_eq!(mixed_red_dummy::add(2, 2), 5); }\n",
    )
    .unwrap();

    let gauntlet_toml = r#"
stack = "rust"
[[layers]]
name = "unit"
command = ["cargo", "test", "--workspace"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), gauntlet_toml).unwrap();

    let tasks_dir = temp.path().join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let task_md = r#"---
type: Task Package
title: "Task 084: Mixed Red Refs"
description: "Verify RED accepts at least one failing test_ref"
status: active
scope:
  - "src/**"
test_refs:
  - "tests/existing_green_test.rs"
  - "tests/new_feature_red_test.rs"
---

# Task 084: Mixed Red Refs

**Status**: `ACTIVE`

## 🎯 Formål
Test mixed green and red test_refs.

## 📋 Acceptance Criteria
- [ ] Criterion 1

## 🚫 Must NOT
- Must not break invariants.
"#;
    fs::write(tasks_dir.join("084-mixed-red-refs.md"), task_md).unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Red, temp.path());
    xgauntlet_core::features::checkpoint::run_preflight_check(&opts)
        .await
        .expect("RED preflight MUST pass when existing_green_test passes and new_feature_red_test fails");
}

#[test]
fn test_task_085_ac2_xgauntlet_runtime_dir_gitignored_and_exempt_from_scope_check() {
    let temp = TempDir::new("task_085_xgauntlet_scope");
    setup_mock_task(temp.path(), "085", "active");

    let staged = vec![".xgauntlet/session_state.json".to_string()];
    xgauntlet_core::features::checkpoint::engine::validate_staged_files_scope(
        temp.path(),
        "085",
        &staged,
    )
    .expect(".xgauntlet/ runtime state must be exempt from checkpoint scope violations");

    let repo_gitignore =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.gitignore"))
            .unwrap();
    assert!(
        repo_gitignore
            .lines()
            .any(|l| l.trim() == ".xgauntlet/" || l.trim() == ".xgauntlet"),
        "Repository .gitignore must ignore .xgauntlet/ so session_state.json never dirties git status"
    );
}

#[tokio::test]
async fn test_task_100_ac1_done_checkpoint_auto_unbinds_matching_active_task() {
    let temp = TempDir::new("task_100_auto_unbind");
    init_git_repo(temp.path());
    setup_mock_glossary(temp.path());
    setup_mock_task(temp.path(), "084-done-transition", "active");
    setup_mock_task(temp.path(), "099-other-active", "active");
    for id in ["084-done-transition", "099-other-active"] {
        let p = temp.path().join(format!("tasks/{id}.md"));
        let txt = fs::read_to_string(&p).unwrap().replace("- [ ]", "- [x]");
        fs::write(&p, txt).unwrap();
    }

    let passing_config = r#"
stack = "rust"
save_evidence = true
[[layers]]
name = "unit"
command = ["true"]
optional = false
"#;
    fs::write(temp.path().join("gauntlet.toml"), passing_config).unwrap();

    fs::create_dir_all(temp.path().join("crates")).unwrap();
    fs::write(temp.path().join("crates/seed.txt"), "red").unwrap();
    Command::new("git")
        .args(["add", "-A"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "test(084): add failing test [RED]"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    // Case 1: Rollback on NothingToCommit/ScopeViolation must preserve .xgauntlet/active_task
    let active_task_file = temp.path().join(".xgauntlet/active_task");
    fs::create_dir_all(active_task_file.parent().unwrap()).unwrap();
    fs::write(&active_task_file, "084-done-transition\n").unwrap();

    fs::write(temp.path().join("outside_scope.txt"), "oops").unwrap();
    let failed_opts = CheckpointOptions::new(CheckpointPhase::Done, temp.path());
    let err = run_checkpoint(&failed_opts).await;
    assert!(
        err.is_err(),
        "DONE checkpoint with out-of-scope file must fail"
    );
    assert!(
        active_task_file.is_file(),
        "Failed/rolled-back DONE checkpoint MUST preserve .xgauntlet/active_task"
    );
    fs::remove_file(temp.path().join("outside_scope.txt")).unwrap();

    // Case 2: Mismatched .xgauntlet/active_task (099) must NOT be removed when finishing 084 explicitly
    fs::write(&active_task_file, "099-other-active\n").unwrap();
    fs::write(temp.path().join("crates/seed.txt"), "green-mismatch").unwrap();
    let explicit_opts =
        CheckpointOptions::new(CheckpointPhase::Done, temp.path()).with_task_id("084-done-transition");
    run_checkpoint(&explicit_opts)
        .await
        .expect("Explicit DONE checkpoint for 084 should succeed");
    assert!(
        active_task_file.is_file(),
        "DONE checkpoint for 084 must NOT remove .xgauntlet/active_task bound to 099"
    );

    // Case 3: Matching short ID or full ID in .xgauntlet/active_task MUST be auto-unbound on DONE
    fs::write(temp.path().join("crates/seed.txt"), "red-99").unwrap();
    Command::new("git")
        .args(["add", "-A"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "test(099): add failing test [RED]"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    fs::write(&active_task_file, "099\n").unwrap();
    fs::write(temp.path().join("crates/seed.txt"), "green-99").unwrap();
    let matching_opts = CheckpointOptions::new(CheckpointPhase::Done, temp.path());
    run_checkpoint(&matching_opts)
        .await
        .expect("Matching DONE checkpoint for 099 should succeed");
    assert!(
        !active_task_file.exists(),
        "Successful DONE checkpoint MUST automatically remove matching .xgauntlet/active_task"
    );
}
