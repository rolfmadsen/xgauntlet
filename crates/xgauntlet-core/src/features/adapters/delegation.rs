//! Shared subagent and swarm delegation classification across all harness adapters (ADR 0014).

use serde_json::Value;

fn str_field(args: &Value, keys: &[&str]) -> String {
    for key in keys {
        if let Some(s) = args.get(*key).and_then(|v| v.as_str()) {
            return s.trim().to_ascii_lowercase();
        }
    }
    String::new()
}

fn is_isolated_workspace(ws: &str) -> bool {
    matches!(ws, "share" | "branch" | "worktree")
}

/// Classifies multi-agent delegation tool invocations into policy-engine `target_resource` modes:
/// `"read_only"`, `"isolated"`, `"shared_inherit"`, `"lifecycle"`, or `"unknown"` (fail-closed).
/// Returns `None` if `raw_tool_name` is not a delegation tool.
pub fn classify_delegation_tool(raw_tool_name: &str, args: &Value) -> Option<&'static str> {
    let tool_lower = raw_tool_name.trim().to_ascii_lowercase();
    match tool_lower.as_str() {
        "invoke_subagent" => {
            let type_name = str_field(args, &["TypeName", "type_name", "type"]);
            let ws = str_field(args, &["Workspace", "workspace"]);
            if type_name == "research" {
                Some("read_only")
            } else if is_isolated_workspace(&ws) {
                Some("isolated")
            } else {
                Some("shared_inherit")
            }
        }
        "swarm" => {
            let action = str_field(args, &["action", "Action"]);
            match action.as_str() {
                "status" | "models" => Some("read_only"),
                "add" => {
                    let ws = str_field(args, &["workspace", "Workspace"]);
                    if is_isolated_workspace(&ws) {
                        Some("isolated")
                    } else {
                        Some("shared_inherit")
                    }
                }
                "kill" | "title" | "model" => Some("lifecycle"),
                _ => Some("unknown"),
            }
        }
        "manage_subagents" => {
            let action = str_field(args, &["Action", "action"]);
            match action.as_str() {
                "list" => Some("read_only"),
                "kill" | "kill_all" => Some("lifecycle"),
                _ => Some("unknown"),
            }
        }
        "send_message" | "define_subagent" | "run_workflow" => Some("lifecycle"),
        "agent" | "task" | "spawn_agent" | "subagent" | "delegate" => {
            let subagent_type = str_field(args, &["subagent_type", "type", "agent_type", "role"]);
            let isolation = str_field(args, &["isolation", "workspace", "Workspace"]);
            if matches!(
                subagent_type.as_str(),
                "explore" | "plan" | "research" | "statusline-setup"
            ) {
                Some("read_only")
            } else if is_isolated_workspace(&isolation) {
                Some("isolated")
            } else {
                Some("shared_inherit")
            }
        }
        _ => None,
    }
}
