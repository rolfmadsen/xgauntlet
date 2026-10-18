//! Acceptance and integration tests for Task 041:
//! Workspace Agnosticism & Production Readiness Triage.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xgauntlet_core::features::adapters::{ClaudeCodeAdapter, CodexAdapter};
use xgauntlet_core::features::checkpoint::{run_checkpoint, CheckpointOptions, CheckpointPhase};
use xgauntlet_core::features::evidence::compute_workspace_manifest;

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
        let path = std::env::temp_dir().join(format!("xgauntlet_triage_test_{}", unique));
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
"#;
    fs::write(workspace.join("CONTEXT.md"), glossary).unwrap();
}

fn setup_mock_task_with_scope(workspace: &Path, task_id: &str, scopes: &[&str]) {
    let tasks_dir = workspace.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let scopes_yaml = scopes
        .iter()
        .map(|s| format!("  - \"{s}\""))
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!(
        r#"---
type: Task Package
title: "Task {task_id}: Scoped Task"
description: "Test task description"
status: active
scope:
{scopes_yaml}
test_refs:
  - "tests/**"
---

# Task {task_id}: Scoped Task

**Status**: `ACTIVE`

## 🎯 Formål
Test task purpose.

## 📋 Acceptance Criteria
- [x] Criterion 1

## 🚫 Must NOT
- Must not violate invariants.
"#
    );
    fs::write(tasks_dir.join(format!("{task_id}.md")), content).unwrap();
}

#[test]
fn test_generic_workspace_manifest_scans_arbitrary_roots() {
    let temp = TempDir::new("arbitrary_manifest_ws");
    let ws = temp.path();

    // Create directories outside DEFAULT_SCOPES (e.g. apps, libs, services)
    fs::create_dir_all(ws.join("apps").join("core")).unwrap();
    fs::write(
        ws.join("apps").join("core").join("main.rs"),
        "fn main() {}\n",
    )
    .unwrap();

    fs::create_dir_all(ws.join("libs").join("common")).unwrap();
    fs::write(
        ws.join("libs").join("common").join("lib.rs"),
        "pub fn hello() {}\n",
    )
    .unwrap();

    fs::create_dir_all(ws.join("services").join("api")).unwrap();
    fs::write(
        ws.join("services").join("api").join("index.ts"),
        "console.log('hi');\n",
    )
    .unwrap();

    fs::write(ws.join("Cargo.toml"), "[workspace]\n").unwrap();

    // Compute manifest without explicit scopes (default)
    let manifest = compute_workspace_manifest(ws, None).expect("Manifest computation must succeed");

    assert!(
        manifest.files.iter().any(|f| f == "apps/core/main.rs"),
        "Manifest must include files in 'apps/' outside hardcoded DEFAULT_SCOPES"
    );
    assert!(
        manifest.files.iter().any(|f| f == "libs/common/lib.rs"),
        "Manifest must include files in 'libs/' outside hardcoded DEFAULT_SCOPES"
    );
    assert!(
        manifest.files.iter().any(|f| f == "services/api/index.ts"),
        "Manifest must include files in 'services/' outside hardcoded DEFAULT_SCOPES"
    );
}

#[tokio::test]
async fn test_checkpoint_fails_closed_when_staged_file_is_outside_task_scope() {
    let temp = TempDir::new("scope_violation_ws");
    let ws = temp.path();
    init_git_repo(ws);
    setup_mock_glossary(ws);

    let task_id = "041-scoped-test";
    setup_mock_task_with_scope(ws, task_id, &["crates/allowed/**"]);

    // Create an allowed file
    fs::create_dir_all(ws.join("crates").join("allowed")).unwrap();
    fs::write(
        ws.join("crates").join("allowed").join("lib.rs"),
        "pub fn ok() {}\n",
    )
    .unwrap();

    // Create an un-scoped foreign file that should trigger fail-closed rejection
    fs::create_dir_all(ws.join("crates").join("forbidden")).unwrap();
    fs::write(
        ws.join("crates").join("forbidden").join("secret.env"),
        "SECRET_KEY=12345\n",
    )
    .unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Spec, ws).with_task_id(task_id);

    let res = run_checkpoint(&opts).await;

    // Checkpoint must fail closed because crates/forbidden/secret.env is outside the task's scope!
    assert!(
        res.is_err(),
        "Checkpoint must fail closed when un-scoped files are staged"
    );
    let err_msg = format!("{:?}", res.err().unwrap());
    assert!(
        err_msg.contains("ScopeViolation")
            || err_msg.contains("scope")
            || err_msg.contains("forbidden"),
        "Error message should mention scope violation or unauthorized file: {err_msg}"
    );
}

#[test]
fn test_claude_code_adapter_scaffolds_pre_tool_use_gatekeeper() {
    let settings = ClaudeCodeAdapter::generate_settings_json(None);

    let pre_tool = settings.get("PreToolUse").expect("Settings must include PreToolUse hook");
    let pre_tool_str = serde_json::to_string(pre_tool).unwrap();
    assert!(
        pre_tool_str.contains("xgauntlet hook"),
        "PreToolUse hook in Claude Code settings must execute 'xgauntlet hook claude_code'"
    );

    let post_tool = settings.get("PostToolUse").expect("Settings must include PostToolUse hook");
    let post_tool_str = serde_json::to_string(post_tool).unwrap();
    assert!(
        post_tool_str.contains("xgauntlet telemetry"),
        "PostToolUse hook in Claude Code settings must execute 'xgauntlet telemetry'"
    );
}

#[test]
fn test_codex_adapter_scaffolds_pre_tool_use_gatekeeper() {
    let hooks = CodexAdapter::generate_hooks_json(None);

    let pre_tool = hooks.get("PreToolUse").expect("Hooks must include PreToolUse hook");
    let pre_tool_str = serde_json::to_string(pre_tool).unwrap();
    assert!(
        pre_tool_str.contains("xgauntlet hook"),
        "PreToolUse hook in Codex hooks must execute 'xgauntlet hook codex'"
    );

    let post_tool = hooks.get("PostToolUse").expect("Hooks must include PostToolUse hook");
    let post_tool_str = serde_json::to_string(post_tool).unwrap();
    assert!(
        post_tool_str.contains("xgauntlet telemetry"),
        "PostToolUse hook in Codex hooks must execute 'xgauntlet telemetry'"
    );
}

#[test]
fn test_review_skill_contains_definition_of_enough() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let skill_path = root.join("../../.agents/plugins/xgauntlet/skills/review/SKILL.md");
    let content = fs::read_to_string(&skill_path)
        .expect("Skill file .agents/plugins/xgauntlet/skills/review/SKILL.md must exist");

    assert!(
        content.contains("Definition of Enough"),
        "Review skill must document 'Definition of Enough'"
    );
    assert!(
        content.contains("SHIP IT"),
        "Review skill must specify 'SHIP IT' completion verdict"
    );
    assert!(
        content.contains("Trusselsmodel") || content.contains("trusselsmodel"),
        "Review skill must specify threat model boundaries"
    );
}
