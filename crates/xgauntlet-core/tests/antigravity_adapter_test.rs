//! Tests for Google Antigravity 2.6.0 adapter protocol, tool normalization, and workspace resolution.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use xgauntlet_core::features::adapters::antigravity::AntigravityAdapter;
use xgauntlet_core::features::adapters::get_adapter;
use xgauntlet_core::features::policy::ToolActionType;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "xg_ag_test_{name}_{}_{seq}_{nanos}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_antigravity_2_6_tools_normalization() {
    let adapter = get_adapter("antigravity").expect("antigravity adapter");

    // Antigravity 2.6 write tools
    for (tool, key) in [("write_file", "Path"), ("edit_file", "Path")] {
        let payload = serde_json::json!({
            "toolCall": {
                "name": tool,
                "args": { key: "crates/xgauntlet-core/src/lib.rs" }
            }
        });
        let n = adapter.normalize_tool_call(&payload);
        assert_eq!(n.action_type, ToolActionType::WriteFile);
        assert_eq!(n.target_resource, "crates/xgauntlet-core/src/lib.rs");
    }

    // Truly passive/interactive read and dialog tools without file path -> ReadFile
    for tool in [
        "ask_question",
        "list_plugin_accounts",
        "search_marketplace",
        "search_web",
    ] {
        let payload = serde_json::json!({
            "toolCall": {
                "name": tool,
                "args": { "query": "test query" }
            }
        });
        let n = adapter.normalize_tool_call(&payload);
        assert_eq!(
            n.action_type,
            ToolActionType::ReadFile,
            "tool {tool} should map to ReadFile"
        );
        assert_eq!(n.target_resource, "");
    }

    // manage_task strictly with Action: "list" or "status" (case-insensitive) -> ReadFile
    for action in ["list", "status", "LIST", "Status"] {
        let payload = serde_json::json!({
            "toolCall": {
                "name": "manage_task",
                "args": { "Action": action }
            }
        });
        let n = adapter.normalize_tool_call(&payload);
        assert_eq!(
            n.action_type,
            ToolActionType::ReadFile,
            "manage_task with action '{action}' should map to ReadFile"
        );
        assert_eq!(n.target_resource, "");
    }

    // Subagent, daemon, messaging and mutating manage_task tools -> Other (fail-closed)
    for tool in [
        "invoke_subagent",
        "define_subagent",
        "manage_subagents",
        "send_message",
        "schedule",
    ] {
        let payload = serde_json::json!({
            "toolCall": {
                "name": tool,
                "args": { "Role": "subagent", "Prompt": "do work" }
            }
        });
        let n = adapter.normalize_tool_call(&payload);
        assert_eq!(
            n.action_type,
            ToolActionType::Other,
            "tool {tool} must map to Other"
        );
    }

    // manage_task with mutating or missing/unknown action -> Other
    for bad_args in [
        serde_json::json!({ "Action": "kill", "TaskId": "task_1" }),
        serde_json::json!({ "Action": "send_input", "TaskId": "task_1", "Input": "yes" }),
        serde_json::json!({ "Action": "destroy" }),
        serde_json::json!({ "question": "Are you ready?" }),
        serde_json::json!({}),
    ] {
        let payload = serde_json::json!({
            "toolCall": {
                "name": "manage_task",
                "args": bad_args
            }
        });
        let n = adapter.normalize_tool_call(&payload);
        assert_eq!(
            n.action_type,
            ToolActionType::Other,
            "mutating or invalid manage_task must map to Other"
        );
    }
}

#[test]
fn test_antigravity_2_6_hook_handler_protocol() {
    let adapter = get_adapter("antigravity").expect("antigravity adapter");
    let temp = TempDir::new("antigravity_hook_2_6");
    let ws = &temp.path;

    // Antigravity 2.6 read_file with absolute path inside workspace
    let abs_read = ws.join("README.md").display().to_string();
    let read_file_json = serde_json::json!({
        "toolCall": {
            "name": "read_file",
            "args": { "Path": abs_read }
        }
    })
    .to_string();
    let (code_rf, out_rf) = adapter.handle_hook(ws, &read_file_json);
    assert_eq!(code_rf, 0);
    assert!(
        out_rf.contains("\"decision\":\"allow\"") || out_rf.contains("\"decision\": \"allow\"")
    );

    // Antigravity 2.6 interactive metadata tool (ask_question)
    let ask_json = serde_json::json!({
        "toolCall": {
            "name": "ask_question",
            "args": { "questions": [] }
        }
    })
    .to_string();
    let (code_ask, out_ask) = adapter.handle_hook(ws, &ask_json);
    assert_eq!(code_ask, 0);
    assert!(
        out_ask.contains("\"decision\":\"allow\"") || out_ask.contains("\"decision\": \"allow\"")
    );
}

#[test]
fn test_antigravity_resolve_effective_workspace_and_containment() {
    let temp = TempDir::new("effective_ws_test");
    let empty_dir = temp.path.join("empty_dir");
    let true_ws = temp.path.join("project");
    fs::create_dir_all(&empty_dir).unwrap();
    fs::create_dir_all(&true_ws).unwrap();
    fs::create_dir_all(true_ws.join("tasks")).unwrap();

    // 1. From workspacePaths payload field when incoming ws is not a valid workspace
    let payload1 = serde_json::json!({
        "workspacePaths": [true_ws.to_str().unwrap()]
    });
    let resolved = AntigravityAdapter::resolve_effective_workspace(&empty_dir, &payload1);
    assert_eq!(resolved, true_ws);

    // 2. Incoming ws is ALREADY a valid workspace: MUST be preserved unchanged!
    let payload_override = serde_json::json!({
        "workspacePaths": [empty_dir.to_str().unwrap()]
    });
    let resolved_preserved =
        AntigravityAdapter::resolve_effective_workspace(&true_ws, &payload_override);
    assert_eq!(resolved_preserved, true_ws);

    // 3. toolCall Path traversal is REMOVED: does NOT change workspace
    let external_repo = temp.path.join("other_repo");
    let subfile = external_repo.join("src").join("ui").join("app.rs");
    fs::create_dir_all(subfile.parent().unwrap()).unwrap();
    fs::write(&subfile, "// external file").unwrap();
    fs::create_dir_all(external_repo.join("tasks")).unwrap();

    let payload_traversal = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "Path": subfile.to_str().unwrap()
            }
        }
    });
    let resolved_no_traversal =
        AntigravityAdapter::resolve_effective_workspace(&true_ws, &payload_traversal);
    assert_eq!(
        resolved_no_traversal, true_ws,
        "toolCall.args.Path must never override valid workspace"
    );

    // 4. Fallback when no metadata present
    let payload_empty = serde_json::json!({});
    let resolved_fallback =
        AntigravityAdapter::resolve_effective_workspace(&empty_dir, &payload_empty);
    assert_eq!(resolved_fallback, empty_dir);
}

#[test]
fn test_antigravity_workspace_containment_denies_outside_repo() {
    let adapter = get_adapter("antigravity").expect("antigravity adapter");
    let temp = TempDir::new("containment_test");
    let repo_a = temp.path.join("repo_a");
    let repo_b = temp.path.join("repo_b");
    fs::create_dir_all(&repo_a).unwrap();
    fs::create_dir_all(&repo_b).unwrap();

    // Mark repo_a as a valid workspace
    fs::create_dir_all(repo_a.join("tasks")).unwrap();
    fs::write(repo_a.join("gauntlet.toml"), "[project]\nname=\"repo_a\"").unwrap();

    // File in external repo_b
    let ext_file = repo_b.join("secret.txt");
    fs::write(&ext_file, "secret data").unwrap();

    // PreToolUse attempting to write to external file in repo_b
    let write_payload = serde_json::json!({
        "toolCall": {
            "name": "write_file",
            "args": {
                "Path": ext_file.to_str().unwrap(),
                "Content": "compromised"
            }
        }
    })
    .to_string();

    let (code, output) = adapter.handle_hook(&repo_a, &write_payload);
    assert_eq!(
        code, 1,
        "Attempt to write outside workspace repo must be denied"
    );
    assert!(
        output.contains("\"decision\":\"deny\"") || output.contains("\"decision\": \"deny\""),
        "Expected deny decision, got: {output}"
    );
}
