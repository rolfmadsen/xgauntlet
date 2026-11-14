use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::adapters::{get_adapter, AntigravityAdapter};
use xgauntlet_core::features::policy::EnforcementContext;
use xgauntlet_core::features::tasks::resolve_active_task_id;
use xgauntlet_core::features::telemetry::record_session_from_hook_payload;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

fn make_temp_dir(prefix: &str) -> PathBuf {
    let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "xgauntlet_ma_{prefix}_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_active_task(ws: &Path, id: &str, scope: &str) {
    let tasks_dir = ws.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let content = format!(
        "---\n\
         type: Task Package\n\
         title: \"Task {id}\"\n\
         description: \"Test task {id}\"\n\
         status: active\n\
         generated: {{ by: process:test, at: \"2026-11-14T22:00:00Z\" }}\n\
         tags: [test]\n\
         scope:\n\
           - \"{scope}\"\n\
         test_refs:\n\
           - \"tests/test_{id}.rs\"\n\
         ---\n\n\
         # Task {id}\n\n\
         **Status**: `ACTIVE`\n\
         **Intent**: `🚀 NEW FEATURE`\n\n\
         ## 🎯 Formål\nTest.\n\n\
         ## 📋 Acceptance Criteria\n- [ ] AC1\n\n\
         ## 🚫 Must NOT\n- None.\n"
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).unwrap();
}

fn write_done_task(ws: &Path, id: &str) {
    let tasks_dir = ws.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let content = format!(
        "---\n\
         type: Task Package\n\
         title: \"Task {id}\"\n\
         status: done\n\
         generated: {{ by: process:test, at: \"2026-11-14T22:00:00Z\" }}\n\
         ---\n\n\
         # Task {id}\n\n\
         **Status**: `DONE`\n\n\
         ## 🎯 Formål\nDone.\n\n\
         ## 📋 Acceptance Criteria\n- [x] AC1\n\n\
         ## 🚫 Must NOT\n- None.\n"
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).unwrap();
}

fn init_git_repo_with_worktree(prefix: &str) -> (PathBuf, PathBuf) {
    let root = make_temp_dir(prefix);
    let repo = root.join("repo");
    let wt = root.join("wt-task-089");
    fs::create_dir_all(&repo).unwrap();

    assert!(Command::new("git")
        .args(["init"])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());

    fs::write(repo.join("README.md"), "# Test\n").unwrap();
    fs::write(repo.join("gauntlet.toml"), "stack = \"rust\"\n").unwrap();
    write_active_task(&repo, "088-alpha", "crates/alpha/**");
    write_active_task(&repo, "089-beta", "crates/beta/**");

    assert!(Command::new("git")
        .args(["add", "."])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());

    assert!(Command::new("git")
        .args(["worktree", "add", "-b", "task/089", wt.to_str().unwrap()])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());

    (repo, wt)
}

#[test]
fn test_worktree_local_active_task_binding_overrides_highest_number() {
    let root = make_temp_dir("task_bind");
    fs::write(root.join("gauntlet.toml"), "stack = \"rust\"\n").unwrap();
    write_active_task(&root, "088-alpha", "crates/alpha/**");
    write_active_task(&root, "089-beta", "crates/beta/**");
    write_done_task(&root, "086-old");

    // Without binding: picks highest active (089-beta)
    assert_eq!(resolve_active_task_id(&root).as_deref(), Some("089-beta"));

    // With .xgauntlet/active_task binding to 088: must resolve 088-alpha
    fs::create_dir_all(root.join(".xgauntlet")).unwrap();
    fs::write(root.join(".xgauntlet/active_task"), "088\n").unwrap();

    assert_eq!(
        resolve_active_task_id(&root).as_deref(),
        Some("088-alpha"),
        "Worktree-local .xgauntlet/active_task must override highest-numbered task"
    );

    let ctx = EnforcementContext::from_workspace(&root, false);
    assert_eq!(ctx.active_task_id, "088-alpha");
    assert_eq!(ctx.allowed_scopes, vec!["crates/alpha/**".to_string()]);

    // If binding points to a DONE/missing task, fail closed (do not silently hijack 089-beta)
    fs::write(root.join(".xgauntlet/active_task"), "086\n").unwrap();
    assert_eq!(
        resolve_active_task_id(&root),
        None,
        "Invalid or non-active .xgauntlet/active_task must fail closed"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_payload_worktree_resolution_across_all_four_harnesses() {
    let (parent_repo, worktree_ws) = init_git_repo_with_worktree("all_harnesses_wt");

    // Parent repo bound to 088-alpha (crates/alpha/**)
    fs::create_dir_all(parent_repo.join(".xgauntlet")).unwrap();
    fs::write(parent_repo.join(".xgauntlet/active_task"), "088-alpha\n").unwrap();

    // Worktree bound to 089-beta (crates/beta/**)
    fs::create_dir_all(worktree_ws.join(".xgauntlet")).unwrap();
    fs::write(worktree_ws.join(".xgauntlet/active_task"), "089-beta\n").unwrap();

    let wt_str = worktree_ws.to_string_lossy().to_string();

    // 1. Antigravity payload with worktree cwd while hook CWD is parent_repo
    let ag_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "cwd": wt_str,
        "toolCall": {
            "name": "write_to_file",
            "args": { "TargetFile": "crates/beta/mod.rs", "CodeContent": "" }
        }
    });
    let resolved = AntigravityAdapter::resolve_effective_workspace(&parent_repo, &ag_payload);
    assert_eq!(
        resolved.canonicalize().unwrap(),
        worktree_ws.canonicalize().unwrap(),
        "Verified git worktree cwd in payload must take precedence over parent_repo"
    );

    // 2. All 4 harnesses must allow crates/beta/mod.rs when payload cwd points to worktree_ws
    let payloads = [
        (
            "antigravity",
            serde_json::json!({
                "hookEventName": "PreToolUse",
                "cwd": worktree_ws,
                "toolCall": {
                    "name": "write_to_file",
                    "args": { "TargetFile": "crates/beta/mod.rs" }
                }
            }),
        ),
        (
            "claude_code",
            serde_json::json!({
                "name": "FileWrite",
                "cwd": worktree_ws,
                "input": { "file_path": "crates/beta/mod.rs" }
            }),
        ),
        (
            "codex",
            serde_json::json!({
                "name": "write_file",
                "cwd": worktree_ws,
                "arguments": { "path": "crates/beta/mod.rs" }
            }),
        ),
        (
            "mistral",
            serde_json::json!({
                "tool_name": "write_file",
                "cwd": worktree_ws,
                "tool_input": { "path": "crates/beta/mod.rs" }
            }),
        ),
    ];

    for (harness, payload) in payloads {
        let adapter = get_adapter(harness).unwrap();
        let (code, out) = adapter.handle_hook(&parent_repo, &payload.to_string());
        assert_eq!(
            code, 0,
            "Harness '{harness}' should allow write inside worktree task 089 scope, got exit {code}: {out}"
        );
        assert!(
            out.contains("allow"),
            "Harness '{harness}' should return allow verdict, got: {out}"
        );
    }

    // Security check (ADR 0006): unrelated /tmp repo in payload.cwd must NOT hijack parent_repo
    let evil_ws = make_temp_dir("evil_repo");
    fs::write(evil_ws.join("gauntlet.toml"), "stack = \"rust\"\n").unwrap();
    write_active_task(&evil_ws, "999-evil", "**");
    let evil_payload = serde_json::json!({
        "cwd": evil_ws,
        "toolCall": {
            "name": "write_to_file",
            "args": { "TargetFile": "crates/beta/mod.rs" }
        }
    });
    let (evil_code, evil_out) =
        get_adapter("antigravity")
            .unwrap()
            .handle_hook(&parent_repo, &evil_payload.to_string());
    assert_ne!(
        evil_code, 0,
        "Unrelated repo in payload.cwd must NOT bypass parent_repo scope: {evil_out}"
    );

    let _ = fs::remove_dir_all(parent_repo.parent().unwrap());
    let _ = fs::remove_dir_all(&evil_ws);
}

#[test]
fn test_antigravity_and_claude_delegation_tools_enforced_by_policy() {
    let ws = make_temp_dir("delegation_hooks");
    fs::write(ws.join("gauntlet.toml"), "stack = \"rust\"\n").unwrap();
    let ag = get_adapter("antigravity").unwrap();
    let cc = get_adapter("claude_code").unwrap();

    // Without active task: read-only research subagent & manage_subagents list MUST be allowed
    let research_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "invoke_subagent",
            "args": { "TypeName": "research", "Role": "Codebase Researcher", "Workspace": "inherit" }
        }
    });
    let (code, out) = ag.handle_hook(&ws, &research_payload.to_string());
    assert_eq!(code, 0, "Read-only research subagent must be allowed without active task: {out}");

    let list_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "manage_subagents",
            "args": { "Action": "list" }
        }
    });
    let (code, out) = ag.handle_hook(&ws, &list_payload.to_string());
    assert_eq!(code, 0, "manage_subagents list must be allowed: {out}");

    // Without active task: isolated swarm worker MUST be denied
    let swarm_share_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "swarm",
            "args": { "action": "add", "name": "worker-1", "workspace": "share" }
        }
    });
    let (code, _) = ag.handle_hook(&ws, &swarm_share_payload.to_string());
    assert_ne!(code, 0, "Swarm add without active task must be denied");

    // With active task: swarm(share) allowed, swarm(inherit) DENIED, Claude Agent(worktree) allowed
    write_active_task(&ws, "088-test", "crates/**");

    let (code, out) = ag.handle_hook(&ws, &swarm_share_payload.to_string());
    assert_eq!(code, 0, "Swarm add with workspace=share and active task must be allowed: {out}");

    let swarm_inherit_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "swarm",
            "args": { "action": "add", "name": "worker-1", "workspace": "inherit" }
        }
    });
    let (code, out) = ag.handle_hook(&ws, &swarm_inherit_payload.to_string());
    assert_ne!(
        code, 0,
        "Swarm add with workspace=inherit must be denied to prevent concurrent writes in one tree: {out}"
    );

    let cc_explore = serde_json::json!({
        "name": "Agent",
        "input": { "subagent_type": "Explore", "prompt": "Inspect crates" }
    });
    let (code, out) = cc.handle_hook(&ws, &cc_explore.to_string());
    assert_eq!(code, 0, "Claude Code Explore subagent must be allowed: {out}");

    let cc_worktree = serde_json::json!({
        "name": "Agent",
        "input": { "subagent_type": "general-purpose", "isolation": "worktree", "prompt": "Implement feature" }
    });
    let (code, out) = cc.handle_hook(&ws, &cc_worktree.to_string());
    assert_eq!(code, 0, "Claude Code isolated worktree Agent must be allowed: {out}");

    let cc_unisolated = serde_json::json!({
        "name": "Agent",
        "input": { "subagent_type": "general-purpose", "prompt": "Edit files directly" }
    });
    let (code, out) = cc.handle_hook(&ws, &cc_unisolated.to_string());
    assert_ne!(code, 0, "Claude Code unisolated mutating Agent must be denied: {out}");

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_session_telemetry_isolated_per_session_id() {
    let ws = make_temp_dir("session_isolation");
    write_active_task(&ws, "088-test", "crates/**");

    let p1 = serde_json::json!({ "session_id": "sess-alpha", "prompt": "hello" });
    let p2 = serde_json::json!({ "session_id": "sess-beta", "prompt": "world" });

    let h1 = record_session_from_hook_payload(&ws, "088-test", Some(&p1), &p1.to_string()).unwrap();
    let h2 = record_session_from_hook_payload(&ws, "088-test", Some(&p2), &p2.to_string()).unwrap();

    assert_eq!(h1.turns_count, 1, "First session must start at 1 turn");
    assert_eq!(
        h2.turns_count, 1,
        "Second concurrent session with distinct session_id must track its own turn count independently (got {})",
        h2.turns_count
    );

    let _ = fs::remove_dir_all(&ws);
}
