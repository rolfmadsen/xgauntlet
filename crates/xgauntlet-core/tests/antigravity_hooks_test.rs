use std::fs;
use std::path::{Path, PathBuf};
use xgauntlet_core::features::adapters::AntigravityAdapter;
use xgauntlet_core::features::adapters::HarnessAdapter;
use xgauntlet_core::features::telemetry::{
    estimate_tokens_from_chars, SessionTracker, SESSION_STATE_FILE,
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
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_ac1_pre_invocation_schema_compliance_flat_format() {
    let generated = AntigravityAdapter::generate_hooks_json(None);
    let xg = generated
        .get("xgauntlet")
        .expect("xgauntlet key must exist");

    // PreInvocation must be a flat array of handler objects directly without matcher
    let pre_inv = xg
        .get("PreInvocation")
        .and_then(|v| v.as_array())
        .expect("PreInvocation must be an array");
    assert_eq!(pre_inv.len(), 1, "Must contain exactly one handler");

    let handler = &pre_inv[0];
    assert!(
        handler.get("matcher").is_none(),
        "PreInvocation handler must NOT contain 'matcher' in official schema. Got: {:?}",
        handler
    );
    assert_eq!(
        handler.get("type").and_then(|v| v.as_str()),
        Some("command")
    );
    assert_eq!(
        handler.get("command").and_then(|v| v.as_str()),
        Some("xgauntlet telemetry --format antigravity-hook")
    );

    // In contrast, PreToolUse is grouped and must have matcher
    let pre_tool = xg
        .get("PreToolUse")
        .and_then(|v| v.as_array())
        .expect("PreToolUse must be an array");
    assert_eq!(pre_tool.len(), 1);
    assert!(
        pre_tool[0].get("matcher").is_some(),
        "PreToolUse must contain 'matcher'"
    );
    assert!(
        pre_tool[0].get("hooks").is_some(),
        "PreToolUse must contain 'hooks' wrapper"
    );
}

#[test]
fn test_ac1_pre_invocation_legacy_grouped_normalization() {
    let legacy = serde_json::json!({
        "xgauntlet": {
            "enabled": true,
            "PreInvocation": [
                {
                    "matcher": ".*",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "xgauntlet telemetry --format antigravity-hook"
                        }
                    ]
                }
            ]
        }
    });

    let migrated = AntigravityAdapter::generate_hooks_json(Some(&legacy));
    let xg = migrated.get("xgauntlet").expect("xgauntlet key must exist");

    let pre_inv = xg
        .get("PreInvocation")
        .and_then(|v| v.as_array())
        .expect("PreInvocation must be an array");
    assert_eq!(
        pre_inv.len(),
        1,
        "PreInvocation should normalize legacy entries to 1 flat handler without duplicate"
    );

    let handler = &pre_inv[0];
    assert!(
        handler.get("matcher").is_none(),
        "Migrated PreInvocation must not have matcher"
    );
    assert_eq!(
        handler.get("command").and_then(|v| v.as_str()),
        Some("xgauntlet telemetry --format antigravity-hook")
    );
}

#[test]
fn test_ac2_scaffold_hooks_fails_on_corrupt_json_without_clobbering() {
    let temp = TempDir::new("antigravity_corrupt_json");
    let ws = &temp.path;
    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let hooks_path = agents_dir.join("hooks.json");
    let corrupt_content = "{\n  \"xgauntlet\": {\n    broken json syntax without closing\n";
    fs::write(&hooks_path, corrupt_content).unwrap();

    let result = AntigravityAdapter::scaffold_hooks(ws);
    assert!(
        result.is_err(),
        "scaffold_hooks must return Err when hooks.json is corrupt"
    );

    let err = result.unwrap_err();
    assert_eq!(
        err.kind(),
        std::io::ErrorKind::InvalidData,
        "Error kind must be InvalidData"
    );
    assert!(
        err.to_string().contains("JSON") || err.to_string().contains("corrupt"),
        "Error message must provide actionable hint about corrupt JSON: {}",
        err
    );

    // Invariant: original file must remain 100% untouched
    let preserved_content = fs::read_to_string(&hooks_path).unwrap();
    assert_eq!(
        preserved_content, corrupt_content,
        "Corrupted hooks.json must NOT be clobbered or modified"
    );
}

#[test]
fn test_ac3_group_key_isolation_does_not_hijack_foreign_keys() {
    let foreign = serde_json::json!({
        "custom-security-scanner": {
            "enabled": true,
            "PreToolUse": [
                {
                    "matcher": "run_command",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "./scripts/custom-scan.sh"
                        }
                    ]
                }
            ]
        }
    });

    let merged = AntigravityAdapter::generate_hooks_json(Some(&foreign));

    // Foreign key must remain completely untouched
    let custom = merged
        .get("custom-security-scanner")
        .expect("Foreign key must still exist");
    assert!(
        custom.get("PreInvocation").is_none(),
        "Foreign key must not be hijacked with PreInvocation"
    );
    let foreign_ptu = custom.get("PreToolUse").and_then(|v| v.as_array()).unwrap();
    assert_eq!(foreign_ptu.len(), 1);
    assert_eq!(
        foreign_ptu[0]["hooks"][0]["command"].as_str(),
        Some("./scripts/custom-scan.sh")
    );

    // xgauntlet key must be added distinctly
    let xg = merged
        .get("xgauntlet")
        .expect("xgauntlet key must be created cleanly");
    assert!(xg.get("PreInvocation").is_some());
    assert!(xg.get("PreToolUse").is_some());
}

#[test]
fn test_ac4_scaffold_scenarios_comprehensive_suite() {
    let temp = TempDir::new("antigravity_scenarios");
    let ws = &temp.path;

    // Scenario 1: Non-existent file
    let path = AntigravityAdapter::scaffold_hooks(ws).expect("Scenario 1 must succeed");
    assert!(path.is_file());
    let content_scen1: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let pre_inv = content_scen1["xgauntlet"]["PreInvocation"]
        .as_array()
        .unwrap();
    assert_eq!(pre_inv.len(), 1);
    assert!(pre_inv[0].get("matcher").is_none());

    // Scenario 2: Valid file with foreign plugin -> merges cleanly
    let foreign_plugin_json = serde_json::json!({
        "my-linter": {
            "enabled": true,
            "PreToolUse": [
                {
                    "matcher": "view_file",
                    "hooks": [{"type": "command", "command": "run-linter"}]
                }
            ]
        }
    });
    fs::write(
        &path,
        serde_json::to_string_pretty(&foreign_plugin_json).unwrap(),
    )
    .unwrap();

    let path2 = AntigravityAdapter::scaffold_hooks(ws).expect("Scenario 2 must succeed");
    let content_scen2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path2).unwrap()).unwrap();
    assert!(
        content_scen2.get("my-linter").is_some(),
        "my-linter must be preserved"
    );
    assert!(
        content_scen2.get("xgauntlet").is_some(),
        "xgauntlet must be merged cleanly"
    );

    // Scenario 3: Corrupt file -> fails without clobbering
    let corrupt_raw = "{ invalid json content ...";
    fs::write(&path2, corrupt_raw).unwrap();
    let res3 = AntigravityAdapter::scaffold_hooks(ws);
    assert!(res3.is_err(), "Scenario 3 must fail on corrupt JSON");
    assert_eq!(fs::read_to_string(&path2).unwrap(), corrupt_raw);
}

fn setup_task_workspace(ws: &Path, status: &str) {
    fs::create_dir_all(ws.join("tasks")).unwrap();
    fs::write(
        ws.join("CONTEXT.md"),
        "# Context\n\n**Task**:\nAn executable unit.\n_Avoid_: Ticket.\n",
    )
    .unwrap();
    fs::write(
        ws.join("tasks/085-telemetry-test.md"),
        format!(
            "---\ntype: Task Package\ntitle: \"Task 085: Telemetry Test\"\ndescription: \"Test\"\nstatus: {status}\nscope:\n  - \"crates/**\"\n---\n\n# Task 085\n\n## 📋 Acceptance Criteria\n- [ ] AC1\n"
        ),
    )
    .unwrap();
}

#[test]
fn test_task_085_ac2_pre_invocation_records_session_and_pre_tool_use_does_not() {
    let temp = TempDir::new("task_085_pre_inv_session");
    let ws = &temp.path;
    setup_task_workspace(ws, "active");

    let adapter = AntigravityAdapter::new();

    // 1. PreToolUse (read_file) MUST NOT record a session turn
    let pre_tool_payload = serde_json::json!({
        "hookEventName": "PreToolUse",
        "toolCall": {
            "name": "read_file",
            "args": { "Path": ws.join("CONTEXT.md").to_string_lossy() }
        }
    });
    let (code_ptu, _) = adapter.handle_hook(ws, &pre_tool_payload.to_string());
    assert_eq!(code_ptu, 0);
    assert!(
        !ws.join(SESSION_STATE_FILE).exists(),
        "PreToolUse must NOT create or increment .xgauntlet/session_state.json"
    );

    // 2. PreInvocation MUST record turn 1 and expose Session: in systemMessage
    let pre_inv_payload = serde_json::json!({
        "hookEventName": "PreInvocation",
        "cwd": ws.to_string_lossy()
    });
    let (code_1, out_1) = adapter.handle_hook(ws, &pre_inv_payload.to_string());
    assert_eq!(code_1, 0);
    assert!(
        ws.join(SESSION_STATE_FILE).is_file(),
        "PreInvocation must persist .xgauntlet/session_state.json"
    );
    let parsed_1: serde_json::Value = serde_json::from_str(&out_1).unwrap();
    let msg_1 = parsed_1["injectSteps"][0]["systemMessage"]["systemMessage"]
        .as_str()
        .unwrap_or("");
    assert!(
        msg_1.contains("Session: 🟢 Optimal (1 turn"),
        "systemMessage must include live Session metric for turn 1, got:\n{msg_1}"
    );

    // 3. Second PreInvocation increments to 2 turns
    let (code_2, out_2) = adapter.handle_hook(ws, &pre_inv_payload.to_string());
    assert_eq!(code_2, 0);
    let parsed_2: serde_json::Value = serde_json::from_str(&out_2).unwrap();
    let msg_2 = parsed_2["injectSteps"][0]["systemMessage"]["systemMessage"]
        .as_str()
        .unwrap_or("");
    assert!(
        msg_2.contains("Session: 🟢 Optimal (2 turns"),
        "systemMessage must increment to 2 turns, got:\n{msg_2}"
    );

    let health = SessionTracker::load(ws, "085-telemetry-test")
        .unwrap()
        .expect("session_state.json must be loadable");
    assert_eq!(health.turns_count, 2);
}

#[test]
fn test_task_085_ac2_pre_invocation_skips_session_recording_when_no_active_task() {
    let temp = TempDir::new("task_085_no_active_task");
    let ws = &temp.path;
    setup_task_workspace(ws, "done");

    let adapter = AntigravityAdapter::new();
    let pre_inv_payload = serde_json::json!({
        "hookEventName": "PreInvocation",
        "cwd": ws.to_string_lossy()
    });
    let (code, _) = adapter.handle_hook(ws, &pre_inv_payload.to_string());
    assert_eq!(code, 0);
    assert!(
        !ws.join(SESSION_STATE_FILE).exists(),
        "PreInvocation must NOT record session turns when no active task exists"
    );
}

#[test]
fn test_task_085_ac4_pre_invocation_snapshot_transcript_and_explicit_tokens() {
    let temp = TempDir::new("task_085_ac4_snapshot");
    let ws = &temp.path;
    setup_task_workspace(ws, "active");

    let transcript_path = ws.join("transcript.jsonl");
    let step1_text = "Analyzing HUD protocol";
    let step2_thinking = "Checking session state wiring across hooks";
    let step3_text = "Running verification gauntlet";
    let jsonl = format!(
        "{}\n{}\n{}\n{}\n",
        serde_json::json!({"step_index": 1, "type": "USER_INPUT", "content": "Start task"}),
        serde_json::json!({"step_index": 2, "type": "PLANNER_RESPONSE", "content": step1_text}),
        serde_json::json!({"step_index": 3, "type": "PLANNER_RESPONSE", "thinking": step2_thinking}),
        serde_json::json!({"step_index": 4, "type": "PLANNER_RESPONSE", "content": step3_text}),
    );
    fs::write(&transcript_path, jsonl).unwrap();

    let expected_chars =
        step1_text.chars().count() + step2_thinking.chars().count() + step3_text.chars().count();
    let expected_tokens = estimate_tokens_from_chars(expected_chars);

    let adapter = AntigravityAdapter::new();
    let payload_with_transcript = serde_json::json!({
        "hookEventName": "PreInvocation",
        "cwd": ws.to_string_lossy(),
        "transcript_path": transcript_path.to_string_lossy()
    });

    // Invoke twice with the SAME transcript snapshot -> must NOT double-count (3 turns, not 6!)
    adapter.handle_hook(ws, &payload_with_transcript.to_string());
    adapter.handle_hook(ws, &payload_with_transcript.to_string());

    let health = SessionTracker::load(ws, "085-telemetry-test")
        .unwrap()
        .expect("session_state.json must exist");
    assert_eq!(
        health.turns_count, 3,
        "transcript_path snapshot must yield 3 PLANNER_RESPONSE turns without double-counting"
    );
    assert_eq!(
        health.estimated_tokens, expected_tokens,
        "transcript_path snapshot must use deterministic char heuristic without accumulation"
    );

    // Explicit total_tokens snapshot called twice -> monotonic max, no accumulation
    let payload_with_tokens = serde_json::json!({
        "hookEventName": "PreInvocation",
        "cwd": ws.to_string_lossy(),
        "total_tokens": 12_000
    });
    adapter.handle_hook(ws, &payload_with_tokens.to_string());
    adapter.handle_hook(ws, &payload_with_tokens.to_string());

    let health2 = SessionTracker::load(ws, "085-telemetry-test")
        .unwrap()
        .expect("session_state.json must exist");
    assert_eq!(
        health2.estimated_tokens, 12_000,
        "explicit total_tokens snapshot must not accumulate quadratically across calls"
    );
}
