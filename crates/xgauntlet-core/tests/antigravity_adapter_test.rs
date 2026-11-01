//! Tests for Google Antigravity 2.6.0 adapter protocol, tool normalization, and workspace resolution.

use std::fs;
use std::path::PathBuf;
use xgauntlet_core::features::adapters::antigravity::AntigravityAdapter;
use xgauntlet_core::features::adapters::get_adapter;
use xgauntlet_core::features::policy::ToolActionType;

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("xg_ag_test_{}_{}", name, std::process::id()));
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

    // Antigravity 2.6 interactive / meta tools
    for tool in ["ask_question", "list_plugin_accounts", "manage_task"] {
        let payload = serde_json::json!({
            "toolCall": {
                "name": tool,
                "args": { "question": "Are you ready?" }
            }
        });
        let n = adapter.normalize_tool_call(&payload);
        assert_eq!(n.action_type, ToolActionType::ReadFile);
        assert_eq!(n.target_resource, "");
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
fn test_antigravity_resolve_effective_workspace() {
    let temp = TempDir::new("effective_ws_test");
    let fallback = temp.path.join("fallback");
    let true_ws = temp.path.join("project");
    fs::create_dir_all(&fallback).unwrap();
    fs::create_dir_all(&true_ws).unwrap();

    // 1. From workspacePaths payload field
    let payload1 = serde_json::json!({
        "workspacePaths": [true_ws.to_str().unwrap()]
    });
    let resolved = AntigravityAdapter::resolve_effective_workspace(&fallback, &payload1);
    assert_eq!(resolved, true_ws);

    // 2. From toolCall Path ancestor traversal
    let subfile = true_ws.join("src").join("ui").join("app.rs");
    fs::create_dir_all(subfile.parent().unwrap()).unwrap();
    fs::write(&subfile, "// test").unwrap();
    fs::create_dir_all(true_ws.join("tasks")).unwrap();

    let payload2 = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "Path": subfile.to_str().unwrap()
            }
        }
    });
    let resolved2 = AntigravityAdapter::resolve_effective_workspace(&fallback, &payload2);
    assert_eq!(resolved2, true_ws);

    // 3. Fallback when no metadata present
    let payload3 = serde_json::json!({});
    let resolved3 = AntigravityAdapter::resolve_effective_workspace(&fallback, &payload3);
    assert_eq!(resolved3, fallback);
}
