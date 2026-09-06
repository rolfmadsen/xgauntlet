use std::fs;
use std::path::PathBuf;
use xgauntlet_core::features::adapters::{get_adapter, HarnessAdapter, SUPPORTED_HARNESSES};
use xgauntlet_core::features::policy::ToolActionType;

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

// ============================================================================
// Parameterized Shared Conformance Test Suite (ADR 0006 §4)
// ============================================================================

fn run_shared_conformance_suite(
    adapter: &dyn HarnessAdapter,
    cmd_payload: serde_json::Value,
    write_payload: serde_json::Value,
    read_payload: serde_json::Value,
    other_payload: serde_json::Value,
) {
    let temp = TempDir::new("conformance");
    let ws = &temp.path;

    // 1. Typed Request Translation
    let norm_cmd = adapter.normalize_tool_call(&cmd_payload);
    assert_eq!(
        norm_cmd.action_type,
        ToolActionType::ExecuteCommand,
        "Adapter {} must map command to ExecuteCommand",
        adapter.name()
    );
    assert!(
        !norm_cmd.target_resource.is_empty(),
        "Adapter {} must extract command target resource",
        adapter.name()
    );

    let norm_write = adapter.normalize_tool_call(&write_payload);
    assert_eq!(
        norm_write.action_type,
        ToolActionType::WriteFile,
        "Adapter {} must map write to WriteFile",
        adapter.name()
    );
    assert!(
        !norm_write.target_resource.is_empty(),
        "Adapter {} must extract write target file",
        adapter.name()
    );

    let norm_read = adapter.normalize_tool_call(&read_payload);
    assert_eq!(
        norm_read.action_type,
        ToolActionType::ReadFile,
        "Adapter {} must map read to ReadFile",
        adapter.name()
    );
    assert!(
        !norm_read.target_resource.is_empty(),
        "Adapter {} must extract read target path",
        adapter.name()
    );

    let norm_other = adapter.normalize_tool_call(&other_payload);
    assert_eq!(
        norm_other.action_type,
        ToolActionType::Other,
        "Adapter {} must map unknown tool to Other",
        adapter.name()
    );

    // 2. Fail-Closed on Malformed, Corrupt, or Empty Input
    let (code_empty, out_empty) = adapter.handle_hook(ws, "");
    assert_eq!(
        code_empty,
        1,
        "Adapter {} must fail closed with exit code 1 on empty input",
        adapter.name()
    );
    assert!(
        out_empty.contains("deny") || out_empty.contains("error") || out_empty.contains("Empty"),
        "Adapter {} must return denial message on empty input",
        adapter.name()
    );

    let (code_corrupt, out_corrupt) = adapter.handle_hook(ws, "{ not-a-json");
    assert_eq!(
        code_corrupt,
        1,
        "Adapter {} must fail closed with exit code 1 on corrupt JSON",
        adapter.name()
    );
    assert!(
        out_corrupt.contains("deny")
            || out_corrupt.contains("error")
            || out_corrupt.contains("Corrupt"),
        "Adapter {} must return denial message on corrupt input",
        adapter.name()
    );

    // 3. Immutability of Trusted Context (ADR 0006 §1)
    // Untrusted payload tries to override has_active_task and read_only
    let mut spoof_payload = write_payload.clone();
    if let serde_json::Value::Object(ref mut map) = spoof_payload {
        map.insert("has_active_task".to_string(), serde_json::Value::Bool(true));
        map.insert("read_only".to_string(), serde_json::Value::Bool(false));
    }
    // With empty workspace and no active task in tasks/, write to production code must be denied
    let verdict = adapter.evaluate_invocation(ws, &spoof_payload);
    assert!(
        !verdict.allowed,
        "Adapter {} must NOT allow untrusted payload to override context without active task",
        adapter.name()
    );

    // 4. Read operations must be allowed
    let read_verdict = adapter.evaluate_invocation(ws, &read_payload);
    assert!(
        read_verdict.allowed,
        "Adapter {} must allow read operations: {}",
        adapter.name(),
        read_verdict.reason
    );

    // 5. Destructive Commands Denied
    let git_push_payload = serde_json::json!({
        "CommandLine": "git push origin main",
        "command": "git push origin main",
        "cmd": "git push origin main"
    });
    // Create wrapped payload matching harness
    let wrapped_git_push = match adapter.name() {
        "antigravity" => serde_json::json!({
            "toolCall": { "name": "run_command", "args": { "CommandLine": "git push origin main" } }
        }),
        "claude_code" => serde_json::json!({
            "name": "Bash",
            "input": { "command": "git push origin main" }
        }),
        "codex" => serde_json::json!({
            "name": "bash",
            "arguments": { "command": "git push origin main" }
        }),
        _ => git_push_payload,
    };
    let push_verdict = adapter.evaluate_invocation(ws, &wrapped_git_push);
    assert!(
        !push_verdict.allowed,
        "Adapter {} must deny git push command",
        adapter.name()
    );
    assert_eq!(push_verdict.decision, "deny");
}

// ============================================================================
// Registry Tests
// ============================================================================

#[test]
fn test_supported_harnesses_registry() {
    assert_eq!(SUPPORTED_HARNESSES.len(), 3);
    assert!(SUPPORTED_HARNESSES.contains(&"antigravity"));
    assert!(SUPPORTED_HARNESSES.contains(&"claude_code"));
    assert!(SUPPORTED_HARNESSES.contains(&"codex"));

    assert!(get_adapter("antigravity").is_some());
    assert!(get_adapter("claude_code").is_some());
    assert!(get_adapter("codex").is_some());
    assert!(get_adapter("unknown").is_none());
}

// ============================================================================
// Antigravity Conformance & Specific Tests
// ============================================================================

#[test]
fn test_antigravity_conformance() {
    let adapter = get_adapter("antigravity").expect("antigravity adapter");

    let cmd_payload = serde_json::json!({
        "toolCall": {
            "name": "run_command",
            "args": { "CommandLine": "cargo test" }
        }
    });
    let write_payload = serde_json::json!({
        "toolCall": {
            "name": "write_to_file",
            "args": { "TargetFile": "src/lib.rs" }
        }
    });
    let read_payload = serde_json::json!({
        "toolCall": {
            "name": "view_file",
            "args": { "AbsolutePath": "README.md" }
        }
    });
    let other_payload = serde_json::json!({
        "toolCall": {
            "name": "custom_extension_tool",
            "args": {}
        }
    });

    run_shared_conformance_suite(
        adapter.as_ref(),
        cmd_payload,
        write_payload,
        read_payload,
        other_payload,
    );
}

#[test]
fn test_antigravity_legacy_and_multiple_tools() {
    let adapter = get_adapter("antigravity").expect("antigravity adapter");

    // Legacy flat format
    let legacy_cmd = serde_json::json!({
        "tool_name": "run_command",
        "tool_input": { "CommandLine": "git status" }
    });
    let norm = adapter.normalize_tool_call(&legacy_cmd);
    assert_eq!(norm.action_type, ToolActionType::ExecuteCommand);
    assert_eq!(norm.target_resource, "git status");

    // File editing tools
    for tool in [
        "write_to_file",
        "replace_file_content",
        "multi_replace_file_content",
    ] {
        let payload = serde_json::json!({
            "toolCall": {
                "name": tool,
                "args": { "TargetFile": "crates/xgauntlet-core/src/lib.rs" }
            }
        });
        let n = adapter.normalize_tool_call(&payload);
        assert_eq!(n.action_type, ToolActionType::WriteFile);
        assert_eq!(n.target_resource, "crates/xgauntlet-core/src/lib.rs");
    }

    // Read tools
    for (tool, key) in [
        ("view_file", "AbsolutePath"),
        ("list_dir", "DirectoryPath"),
        ("grep_search", "SearchPath"),
        ("find_by_name", "SearchDirectory"),
        ("read_url_content", "Url"),
    ] {
        let payload = serde_json::json!({
            "toolCall": {
                "name": tool,
                "args": { key: "/path/to/target" }
            }
        });
        let n = adapter.normalize_tool_call(&payload);
        assert_eq!(n.action_type, ToolActionType::ReadFile);
        assert_eq!(n.target_resource, "/path/to/target");
    }
}

#[test]
fn test_antigravity_hook_handler_protocol() {
    let adapter = get_adapter("antigravity").expect("antigravity adapter");
    let temp = TempDir::new("antigravity_hook");
    let ws = &temp.path;

    // Allowed read invocation
    let read_json = serde_json::json!({
        "toolCall": {
            "name": "view_file",
            "args": { "AbsolutePath": "README.md" }
        }
    })
    .to_string();

    let (code, out) = adapter.handle_hook(ws, &read_json);
    assert_eq!(code, 0);
    assert!(out.contains("\"decision\":\"allow\"") || out.contains("\"decision\": \"allow\""));

    // Denied git push invocation
    let push_json = serde_json::json!({
        "toolCall": {
            "name": "run_command",
            "args": { "CommandLine": "git push --force" }
        }
    })
    .to_string();

    let (code_deny, out_deny) = adapter.handle_hook(ws, &push_json);
    assert_eq!(code_deny, 1);
    assert!(
        out_deny.contains("\"decision\":\"deny\"") || out_deny.contains("\"decision\": \"deny\"")
    );
    assert!(out_deny.contains("reason"));
}

#[test]
fn test_antigravity_plugin_validation() {
    let adapter = get_adapter("antigravity").expect("antigravity adapter");
    let temp = TempDir::new("antigravity_plugin");
    let pdir = temp.path.join(".agents");
    fs::create_dir_all(&pdir).unwrap();

    // 1. Missing plugin.json
    let res = adapter.validate_plugin(&pdir);
    assert!(!res.valid);
    assert!(res.issues.iter().any(|i| i.path == "plugin.json"));

    // 2. Invalid plugin.json (missing name and version)
    fs::write(pdir.join("plugin.json"), "{}").unwrap();
    let res = adapter.validate_plugin(&pdir);
    assert!(!res.valid);
    assert!(res.issues.iter().any(|i| i.message.contains("name")));

    // 3. Valid plugin.json with skill
    let manifest = serde_json::json!({
        "name": "test-plugin",
        "version": "1.0.0",
        "skills": ["my-skill"]
    });
    fs::write(
        pdir.join("plugin.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    // Skill missing SKILL.md
    let res = adapter.validate_plugin(&pdir);
    assert!(!res.valid);
    assert!(res.issues.iter().any(|i| i.path.contains("SKILL.md")));

    // Create valid SKILL.md
    let skill_dir = pdir.join("skills").join("my-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: my-skill\ndescription: A test skill\n---\n# Skill body\n",
    )
    .unwrap();

    // Create valid hooks.json
    let hooks = serde_json::json!({
        "gauntlet-gatekeeper": {
            "enabled": true,
            "PreToolUse": [
                {
                    "matcher": "*",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "xgauntlet hook antigravity",
                            "timeout": 5000
                        }
                    ]
                }
            ]
        }
    });
    fs::write(
        pdir.join("hooks.json"),
        serde_json::to_string_pretty(&hooks).unwrap(),
    )
    .unwrap();

    let res = adapter.validate_plugin(&pdir);
    assert!(
        res.valid,
        "Validation failed: {:?}",
        res.issues.iter().map(|i| &i.message).collect::<Vec<_>>()
    );
}

// ============================================================================
// Claude Code Conformance & Specific Tests
// ============================================================================

#[test]
fn test_claude_code_conformance() {
    let adapter = get_adapter("claude_code").expect("claude_code adapter");

    let cmd_payload = serde_json::json!({
        "name": "Bash",
        "input": { "command": "npm test" }
    });
    let write_payload = serde_json::json!({
        "name": "FileEdit",
        "input": { "file_path": "src/index.ts" }
    });
    let read_payload = serde_json::json!({
        "name": "FileRead",
        "input": { "file_path": "package.json" }
    });
    let other_payload = serde_json::json!({
        "name": "TodoManager",
        "input": {}
    });

    run_shared_conformance_suite(
        adapter.as_ref(),
        cmd_payload,
        write_payload,
        read_payload,
        other_payload,
    );
}

#[test]
fn test_claude_code_tools_mapping() {
    let adapter = get_adapter("claude_code").expect("claude_code adapter");

    // Command tools
    for tool in ["Bash", "bash", "run_command"] {
        let p = serde_json::json!({ "name": tool, "input": { "command": "git status" } });
        let n = adapter.normalize_tool_call(&p);
        assert_eq!(n.action_type, ToolActionType::ExecuteCommand);
        assert_eq!(n.target_resource, "git status");
    }

    // Write tools
    for tool in [
        "FileEdit",
        "FileWrite",
        "StrReplace",
        "write_file",
        "edit_file",
        "create_file",
    ] {
        let p = serde_json::json!({ "name": tool, "input": { "file_path": "docs/readme.md" } });
        let n = adapter.normalize_tool_call(&p);
        assert_eq!(n.action_type, ToolActionType::WriteFile);
        assert_eq!(n.target_resource, "docs/readme.md");
    }

    // Read tools
    for (tool, key) in [
        ("FileRead", "file_path"),
        ("View", "path"),
        ("GlobTool", "pattern"),
        ("GrepTool", "path"),
        ("LS", "path"),
    ] {
        let p = serde_json::json!({ "name": tool, "input": { key: "crates/" } });
        let n = adapter.normalize_tool_call(&p);
        assert_eq!(n.action_type, ToolActionType::ReadFile);
        assert_eq!(n.target_resource, "crates/");
    }
}

#[test]
fn test_claude_code_hook_handler() {
    let adapter = get_adapter("claude_code").expect("claude_code adapter");
    let temp = TempDir::new("claude_hook");
    let ws = &temp.path;

    let read_payload = serde_json::json!({
        "name": "FileRead",
        "input": { "file_path": "README.md" }
    })
    .to_string();

    let (code, out) = adapter.handle_hook(ws, &read_payload);
    assert_eq!(code, 0);
    assert!(out.contains("allow") || out.contains("\"decision\":\"allow\""));

    let push_payload = serde_json::json!({
        "name": "Bash",
        "input": { "command": "git push origin main" }
    })
    .to_string();

    let (code_deny, out_deny) = adapter.handle_hook(ws, &push_payload);
    assert_eq!(code_deny, 1);
    assert!(out_deny.contains("deny") || out_deny.contains("denied"));
}

#[test]
fn test_claude_code_validation() {
    let adapter = get_adapter("claude_code").expect("claude_code adapter");
    let temp = TempDir::new("claude_validation");
    let pdir = temp.path.join(".claude");
    fs::create_dir_all(&pdir).unwrap();

    // Valid when CLAUDE.md or settings exist
    fs::write(temp.path.join("CLAUDE.md"), "# Project Guide").unwrap();
    let res = adapter.validate_plugin(&pdir);
    assert!(res.valid);
}

// ============================================================================
// OpenAI Codex Conformance & Specific Tests
// ============================================================================

#[test]
fn test_codex_conformance() {
    let adapter = get_adapter("codex").expect("codex adapter");

    let cmd_payload = serde_json::json!({
        "name": "bash",
        "arguments": { "command": "python test.py" }
    });
    let write_payload = serde_json::json!({
        "name": "apply_patch",
        "arguments": { "path": "app.py" }
    });
    let read_payload = serde_json::json!({
        "name": "read_file",
        "arguments": { "path": "requirements.txt" }
    });
    let other_payload = serde_json::json!({
        "name": "web_search",
        "arguments": { "query": "rust docs" }
    });

    run_shared_conformance_suite(
        adapter.as_ref(),
        cmd_payload,
        write_payload,
        read_payload,
        other_payload,
    );
}

#[test]
fn test_codex_function_call_variations() {
    let adapter = get_adapter("codex").expect("codex adapter");

    // 1. JSON object arguments
    let obj_call = serde_json::json!({
        "name": "execute_code",
        "arguments": { "code": "cargo build" }
    });
    let n1 = adapter.normalize_tool_call(&obj_call);
    assert_eq!(n1.action_type, ToolActionType::ExecuteCommand);
    assert_eq!(n1.target_resource, "cargo build");

    // 2. Stringified JSON arguments
    let str_call = serde_json::json!({
        "name": "write_file",
        "arguments": "{\"path\": \"src/main.rs\"}"
    });
    let n2 = adapter.normalize_tool_call(&str_call);
    assert_eq!(n2.action_type, ToolActionType::WriteFile);
    assert_eq!(n2.target_resource, "src/main.rs");

    // 3. Standard OpenAI Tool Call structure (type="function")
    let tool_call = serde_json::json!({
        "type": "function",
        "function": {
            "name": "bash",
            "arguments": "{\"command\": \"ls -la\"}"
        }
    });
    let n3 = adapter.normalize_tool_call(&tool_call);
    assert_eq!(n3.action_type, ToolActionType::ExecuteCommand);
    assert_eq!(n3.target_resource, "ls -la");
}

#[test]
fn test_codex_hook_handler() {
    let adapter = get_adapter("codex").expect("codex adapter");
    let temp = TempDir::new("codex_hook");
    let ws = &temp.path;

    let read_payload = serde_json::json!({
        "name": "read_file",
        "arguments": { "path": "Cargo.toml" }
    })
    .to_string();

    let (code, out) = adapter.handle_hook(ws, &read_payload);
    assert_eq!(code, 0);
    assert!(out.contains("allow") || out.contains("\"decision\":\"allow\""));

    let deny_payload = serde_json::json!({
        "name": "bash",
        "arguments": { "command": "git push" }
    })
    .to_string();

    let (code_deny, out_deny) = adapter.handle_hook(ws, &deny_payload);
    assert_eq!(code_deny, 1);
    assert!(out_deny.contains("deny") || out_deny.contains("denied"));
}
