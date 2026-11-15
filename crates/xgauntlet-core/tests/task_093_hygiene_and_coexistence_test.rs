use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::adapters::{get_adapter, AntigravityAdapter};
use xgauntlet_core::features::checkpoint::{run_checkpoint, stage_workspace_changes, CheckpointOptions, CheckpointPhase};
use xgauntlet_core::features::config::GauntletConfig;
use xgauntlet_core::features::evidence::{
    compute_workspace_manifest, load_verification_report, save_verification_report,
    verify_self_mutation, VerificationReport,
};
use xgauntlet_core::features::gauntlet::{execute_layer, LayerDefinition};
use xgauntlet_core::features::telemetry::{
    record_session_turn_for_session, reset_session_health, SessionTracker,
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

fn make_temp_dir(prefix: &str) -> PathBuf {
    let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "xgauntlet_t093_{prefix}_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn git_ok(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git command failed to spawn");
    assert!(
        out.status.success(),
        "git {:?} failed in {:?}: {}",
        args,
        dir,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn init_git_repo(dir: &Path) {
    git_ok(dir, &["init"]);
    git_ok(dir, &["config", "user.email", "test@example.com"]);
    git_ok(dir, &["config", "user.name", "Tester"]);
}

fn write_active_task(ws: &Path, id: &str, scope: &str, test_ref: &str, intent: &str) {
    let tasks_dir = ws.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let content = format!(
        "---\n\
         type: Task Package\n\
         title: \"Task {id}\"\n\
         description: \"Test task {id}\"\n\
         status: active\n\
         intent: {intent}\n\
         generated: {{ by: process:test, at: \"2026-11-15T12:00:00Z\" }}\n\
         tags: [test]\n\
         scope:\n\
           - \"{scope}\"\n\
         test_refs:\n\
           - \"{test_ref}\"\n\
         ---\n\n\
         # Task {id}\n\n\
         **Status**: `ACTIVE`\n\
         **Intent**: `🐛 BUG FIX`\n\n\
         ## 🎯 Formål\nTest.\n\n\
         ## 📋 Acceptance Criteria\n- [ ] AC1\n\n\
         ## 🚫 Must NOT\n- None.\n"
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).unwrap();
}

#[test]
fn test_ac1_manifest_excludes_xgauntlet_and_worktrees() {
    let ws = make_temp_dir("manifest_excl");
    fs::create_dir_all(ws.join("src")).unwrap();
    fs::write(ws.join("src/lib.rs"), "pub fn ok() -> bool { true }\n").unwrap();

    fs::create_dir_all(ws.join(".xgauntlet/sessions")).unwrap();
    fs::write(ws.join(".xgauntlet/session_state.json"), r#"{"turns":1}"#).unwrap();
    fs::write(ws.join(".xgauntlet/sessions/s1.json"), r#"{"turns":1}"#).unwrap();

    fs::create_dir_all(ws.join(".worktrees/094/src")).unwrap();
    fs::write(ws.join(".worktrees/094/src/lib.rs"), "pub fn wip() {}\n").unwrap();

    let pre = compute_workspace_manifest(&ws, None).unwrap();
    assert!(
        !pre.files.iter().any(|f| f.starts_with(".xgauntlet/")),
        ".xgauntlet/ must be excluded from canonical workspace manifest: {:?}",
        pre.files
    );
    assert!(
        !pre.files.iter().any(|f| f.starts_with(".worktrees/")),
        ".worktrees/ must be excluded from canonical workspace manifest: {:?}",
        pre.files
    );

    // Mutate runtime telemetry and worktree files during verification
    fs::write(ws.join(".xgauntlet/session_state.json"), r#"{"turns":2}"#).unwrap();
    fs::write(ws.join(".xgauntlet/sessions/s2.json"), r#"{"turns":1}"#).unwrap();
    fs::write(ws.join(".worktrees/094/src/lib.rs"), "pub fn wip2() {}\n").unwrap();

    let post = compute_workspace_manifest(&ws, None).unwrap();
    verify_self_mutation(&pre, &post)
        .expect("Mutating .xgauntlet/ or .worktrees/ must never trigger self-mutation failure");

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_ac1_checkpoint_stage_excludes_previously_tracked_xgauntlet() {
    let ws = make_temp_dir("stage_excl");
    init_git_repo(&ws);
    fs::create_dir_all(ws.join("src")).unwrap();
    fs::create_dir_all(ws.join(".xgauntlet")).unwrap();
    fs::write(ws.join("src/lib.rs"), "pub fn v1() {}\n").unwrap();
    fs::write(ws.join(".xgauntlet/session_state.json"), r#"{"turns":1}"#).unwrap();

    git_ok(&ws, &["add", "."]);
    git_ok(&ws, &["commit", "-m", "init with tracked session_state"]);

    // Add .gitignore afterwards (just like in Kant), then mutate both src/lib.rs and .xgauntlet/session_state.json
    fs::write(ws.join(".gitignore"), ".xgauntlet/\n.worktrees/\n").unwrap();
    fs::write(ws.join(".xgauntlet/session_state.json"), r#"{"turns":2}"#).unwrap();
    fs::write(ws.join("src/lib.rs"), "pub fn v2() {}\n").unwrap();

    let staged = stage_workspace_changes(&ws).expect("stage_workspace_changes must succeed");
    assert!(
        !staged.iter().any(|p| p.starts_with(".xgauntlet/")),
        "checkpoint staging must never stage .xgauntlet/ files even if previously tracked: {:?}",
        staged
    );
    assert!(staged.iter().any(|p| p == "src/lib.rs"));

    let _ = fs::remove_dir_all(&ws);
}

#[tokio::test]
async fn test_ac2_checkpoint_done_leaves_clean_working_tree_when_evidence_tracked() {
    let ws = make_temp_dir("done_clean");
    init_git_repo(&ws);

    fs::create_dir_all(ws.join("src")).unwrap();
    fs::create_dir_all(ws.join("tests")).unwrap();
    fs::write(ws.join("src/lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a }\n").unwrap();
    fs::write(ws.join("tests/bug_test.rs"), "#[test] fn t() {}\n").unwrap();
    fs::write(ws.join("gauntlet.toml"), "stack = \"rust\"\n").unwrap();
    write_active_task(&ws, "093-clean", "src/lib.rs", "tests/bug_test.rs", "bug");

    let initial_report = VerificationReport::new_local("093-clean");
    save_verification_report(&ws, &initial_report).unwrap();

    git_ok(&ws, &["add", "."]);
    git_ok(&ws, &["commit", "-m", "init tracked evidence"]);

    // Simulate RED commit
    fs::write(ws.join("tests/bug_test.rs"), "#[test] fn t() { assert_eq!(2, 1 + 1); }\n").unwrap();
    git_ok(&ws, &["add", "."]);
    git_ok(&ws, &["commit", "-m", "test(093-clean): failing test [RED]"]);

    // Minimal fix for DONE
    fs::write(ws.join("src/lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Done, &ws)
        .with_task_id("093-clean")
        .with_skip_verify(true);
    let res = run_checkpoint(&opts).await.expect("DONE checkpoint must succeed");

    let report = load_verification_report(&ws).expect("verification-report.json must exist");
    let lineage = report.lineage.expect("lineage must be recorded");
    assert!(
        lineage.entries.iter().any(|e| Some(&e.commit_oid) == res.commit_oid.as_ref()),
        "lineage must still record the DONE commit OID"
    );

    let status = git_ok(&ws, &["status", "--porcelain"]);
    assert!(
        status.trim().is_empty(),
        "git status must be completely clean after checkpoint --phase done, got:\n{status}"
    );

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_ac3_per_session_files_isolated_and_reset_preserves_other_tasks() {
    let ws = make_temp_dir("session_files");

    let h1 = record_session_turn_for_session(&ws, "093-task", "swarm-worker-1", 100, None).unwrap();
    let h2 = record_session_turn_for_session(&ws, "094-other", "other-session-2", 200, None).unwrap();
    assert_eq!(h1.turns_count, 1);
    assert_eq!(h2.turns_count, 1);

    let sessions_dir = ws.join(".xgauntlet/sessions");
    assert!(
        sessions_dir.is_dir(),
        "Explicit session IDs must persist under .xgauntlet/sessions/ to prevent shared-file write races"
    );
    let file_count = fs::read_dir(&sessions_dir).unwrap().count();
    assert_eq!(file_count, 2, "Expected 2 isolated per-session telemetry files");

    // Resetting task 093-task must NOT wipe 094-other's session
    reset_session_health(&ws, "093-task").unwrap();
    let other_loaded = SessionTracker::load_for_session(&ws, "094-other", Some("other-session-2"))
        .unwrap()
        .expect("Resetting 093-task must preserve sibling session for 094-other");
    assert_eq!(other_loaded.turns_count, 1);

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_ac4_read_only_browser_and_custom_md_subagents_allowed_without_task() {
    let ws = make_temp_dir("ro_subagents");
    fs::write(ws.join("gauntlet.toml"), "stack = \"rust\"\n").unwrap();
    fs::create_dir_all(ws.join(".agents/agents")).unwrap();
    fs::write(
        ws.join(".agents/agents/code-auditor.md"),
        "---\nname: code-auditor\ndescription: Read-only auditor\ntools:\n  - view_file\n  - grep_search\n  - find_by_name\n---\n# Auditor\n",
    )
    .unwrap();
    fs::write(
        ws.join(".agents/agents/mutating-coder.md"),
        "---\nname: mutating-coder\ndescription: Writes code\ntools:\n  - view_file\n  - write_to_file\n---\n# Coder\n",
    )
    .unwrap();

    let ag = get_adapter("antigravity").unwrap();

    // 1. Built-in browser subagent -> read_only -> allowed without active task
    let browser_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "invoke_subagent",
            "args": { "TypeName": "browser", "Role": "UI Tester", "Workspace": "inherit" }
        }
    });
    let (code, out) = ag.handle_hook(&ws, &browser_payload.to_string());
    assert_eq!(code, 0, "browser subagent must be allowed as read_only: {out}");

    // 2. Custom read-only .agents/agents/code-auditor.md -> allowed without active task
    let auditor_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "invoke_subagent",
            "args": { "TypeName": "code-auditor", "Role": "Security Auditor", "Workspace": "inherit" }
        }
    });
    let (code, out) = ag.handle_hook(&ws, &auditor_payload.to_string());
    assert_eq!(code, 0, "read-only custom .md subagent must be allowed: {out}");

    // 3. Mutating custom subagent in inherit with default config -> still denied (4040)
    write_active_task(&ws, "093-ro", "crates/**", "tests/t.rs", "feature");
    let coder_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "invoke_subagent",
            "args": { "TypeName": "mutating-coder", "Role": "Coder", "Workspace": "inherit" }
        }
    });
    let (code, out) = ag.handle_hook(&ws, &coder_payload.to_string());
    assert_ne!(code, 0, "mutating custom subagent in inherit must remain denied by default: {out}");

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_ac4_workspace_branch_clone_resolved_safely() {
    let root = make_temp_dir("branch_clone");
    let parent = root.join("parent");
    let clone = root.join("branch-clone");
    fs::create_dir_all(&parent).unwrap();
    init_git_repo(&parent);
    fs::write(parent.join("gauntlet.toml"), "stack = \"rust\"\n").unwrap();
    write_active_task(&parent, "093-p", "crates/a/**", "tests/a.rs", "feature");
    git_ok(&parent, &["add", "."]);
    git_ok(&parent, &["commit", "-m", "init"]);

    git_ok(
        &root,
        &[
            "clone",
            parent.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );

    let payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "cwd": clone.to_string_lossy(),
        "toolCall": {
            "name": "write_to_file",
            "args": { "TargetFile": "crates/a/mod.rs" }
        }
    });

    let resolved = AntigravityAdapter::resolve_effective_workspace(&parent, &payload);
    assert_eq!(
        resolved.canonicalize().unwrap(),
        clone.canonicalize().unwrap(),
        "Antigravity Workspace: 'branch' clone must resolve to the cloned workspace"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_ac4_allow_coordinated_inherit_opt_in_via_gauntlet_toml() {
    let ws = make_temp_dir("coord_inherit");
    fs::write(
        ws.join("gauntlet.toml"),
        "stack = \"rust\"\n\n[multi_agent]\nallow_coordinated_inherit = true\nshare_cargo_target_dir = true\n",
    )
    .unwrap();
    write_active_task(&ws, "093-swarm", "crates/**", "tests/s.rs", "feature");

    let cfg = GauntletConfig::load(&ws).unwrap();
    let rendered = cfg.render_toml();
    assert!(
        rendered.contains("allow_coordinated_inherit = true"),
        "GauntletConfig must parse and render [multi_agent].allow_coordinated_inherit: {rendered}"
    );
    assert!(
        rendered.contains("share_cargo_target_dir = true"),
        "GauntletConfig must parse and render [multi_agent].share_cargo_target_dir: {rendered}"
    );

    let ag = get_adapter("antigravity").unwrap();
    let swarm_inherit = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "swarm",
            "args": { "action": "add", "name": "worker-1", "workspace": "inherit" }
        }
    });
    let (code, out) = ag.handle_hook(&ws, &swarm_inherit.to_string());
    assert_eq!(
        code, 0,
        "Coordinated swarm inherit must be allowed when allow_coordinated_inherit = true: {out}"
    );

    let _ = fs::remove_dir_all(&ws);
}

#[tokio::test]
async fn test_ac5_shared_cargo_target_dir_injected_in_linked_worktree() {
    let root = make_temp_dir("shared_target");
    let repo = root.join("repo");
    let wt = root.join("wt");
    fs::create_dir_all(&repo).unwrap();
    init_git_repo(&repo);
    fs::write(
        repo.join("gauntlet.toml"),
        "stack = \"rust\"\n\n[multi_agent]\nshare_cargo_target_dir = true\n",
    )
    .unwrap();
    git_ok(&repo, &["add", "."]);
    git_ok(&repo, &["commit", "-m", "init"]);
    git_ok(&repo, &["worktree", "add", "-b", "task/093", wt.to_str().unwrap()]);

    let layer = LayerDefinition::new(
        "cargo-env-probe",
        vec![
            "cargo".to_string(),
            "--version".to_string(),
        ],
    );
    // Use a shell/python-free cross-platform check or sh -c on unix to inspect CARGO_TARGET_DIR when command is cargo-prefixed or layer runner runs in worktree:
    #[cfg(unix)]
    let probe_layer = LayerDefinition::new(
        "env-probe",
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "printf '%s' \"$CARGO_TARGET_DIR\"".to_string(),
        ],
    );
    #[cfg(not(unix))]
    let probe_layer = layer.clone();

    // Ensure CARGO_TARGET_DIR is not leaking from outer test env
    let res = execute_layer(&probe_layer, &wt).await;
    assert!(res.passed, "env probe failed: {}", res.output);
    #[cfg(unix)]
    {
        let expected_target = repo.canonicalize().unwrap().join("target");
        assert_eq!(
            PathBuf::from(res.output.trim()),
            expected_target,
            "execute_layer in linked worktree with share_cargo_target_dir=true must inject CARGO_TARGET_DIR pointing to primary repo target/"
        );
    }

    let _ = layer;
    let _ = fs::remove_dir_all(&root);
}
