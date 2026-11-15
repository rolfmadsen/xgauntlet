//! Shared subagent and swarm delegation classification across all harness adapters (ADR 0014).

use serde_json::Value;
use std::fs;
use std::path::Path;

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

fn is_read_only_tool_name(tool: &str) -> bool {
    matches!(
        tool.trim().to_ascii_lowercase().as_str(),
        "view_file"
            | "read_file"
            | "grep_search"
            | "find_by_name"
            | "list_dir"
            | "read_url_content"
            | "search_web"
            | "list_plugin_accounts"
            | "search_marketplace"
            | "ask_question"
    )
}

fn extract_frontmatter_tools(content: &str) -> Option<Vec<String>> {
    let trimmed = content.trim_start_matches('\u{feff}');
    let rest = trimmed.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    let yaml = &rest[..end];

    let mut tools = Vec::new();
    let mut in_tools_list = false;
    for raw_line in yaml.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(after) = line.strip_prefix("tools:") {
            let inline = after.trim();
            if inline.starts_with('[') && inline.ends_with(']') {
                for item in inline[1..inline.len() - 1].split(',') {
                    let clean = item.trim().trim_matches('"').trim_matches('\'');
                    if !clean.is_empty() {
                        tools.push(clean.to_string());
                    }
                }
                return Some(tools);
            }
            in_tools_list = true;
            continue;
        }
        if in_tools_list {
            if let Some(item) = line.strip_prefix('-') {
                let clean = item.trim().trim_matches('"').trim_matches('\'');
                if !clean.is_empty() {
                    tools.push(clean.to_string());
                }
            } else if line.contains(':') {
                break;
            }
        }
    }

    if tools.is_empty() {
        None
    } else {
        Some(tools)
    }
}

/// Checks if a custom Antigravity `.md` subagent in `.agents/agents/` restricts `tools` strictly to read-only operations.
pub fn is_custom_read_only_subagent(workspace: &Path, type_name: &str) -> bool {
    let clean = type_name.trim();
    if clean.is_empty() || clean.contains("..") || clean.contains('/') || clean.contains('\\') {
        return false;
    }

    let candidates = [
        workspace.join(".agents/agents").join(format!("{clean}.md")),
        workspace
            .join(".agents/agents")
            .join(clean)
            .join("agent.md"),
    ];

    for path in candidates {
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(tools) = extract_frontmatter_tools(&content) {
                    return !tools.is_empty() && tools.iter().all(|t| is_read_only_tool_name(t));
                }
            }
        }
    }
    false
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
            if matches!(
                type_name.as_str(),
                "research" | "browser" | "image-generator"
            ) {
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
