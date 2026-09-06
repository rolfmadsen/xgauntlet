//! Claude Code vertical slice harness adapter.
//!
//! Translates Claude Code tool calls and lifecycle hooks into
//! canonical gauntlet operations and enforces fail-closed policy gates.

use serde_json::Value;
use std::path::Path;

use crate::features::adapters::models::{
    AdapterValidationResult, HarnessAdapter, NormalizedToolCall, ValidationIssue,
    ValidationSeverity,
};
use crate::features::policy::ToolActionType;

#[derive(Debug, Default, Clone)]
pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl HarnessAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &'static str {
        "claude_code"
    }

    fn normalize_tool_call(&self, payload: &Value) -> NormalizedToolCall {
        let mut tool_name = String::new();
        let mut args = Value::Object(serde_json::Map::new());

        // 1. Claude Code direct payload format {"name": "...", "input": {...}}
        if let Some(name) = payload.get("name").and_then(|n| n.as_str()) {
            tool_name = name.trim().to_string();
            if let Some(input) = payload.get("input") {
                args = input.clone();
            }
        }
        // 2. Flat format {"tool_name": "...", "tool_input": {...}}
        else if let Some(name) = payload.get("tool_name").and_then(|n| n.as_str()) {
            tool_name = name.trim().to_string();
            if let Some(input) = payload.get("tool_input") {
                args = input.clone();
            }
        }
        // 3. Nested toolCall format
        else if let Some(tool_call) = payload.get("toolCall").and_then(|t| t.as_object()) {
            if let Some(name) = tool_call.get("name").and_then(|n| n.as_str()) {
                tool_name = name.trim().to_string();
            }
            if let Some(call_args) = tool_call.get("args") {
                args = call_args.clone();
            }
        }
        // 4. Naked command payload {"command": "..."}
        else if payload.get("command").is_some() {
            tool_name = "Bash".to_string();
            args = payload.clone();
        }

        let name_lower = tool_name.to_ascii_lowercase();

        // Execution tools
        if matches!(
            name_lower.as_str(),
            "bash" | "run_command" | "terminal" | "command" | "execute_command"
        ) {
            let cmd = args
                .get("command")
                .or_else(|| args.get("CommandLine"))
                .or_else(|| args.get("cmd"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            return NormalizedToolCall {
                action_type: ToolActionType::ExecuteCommand,
                target_resource: cmd,
                raw_tool_name: tool_name,
                payload: args,
            };
        }

        // Write tools
        if matches!(
            name_lower.as_str(),
            "fileedit"
                | "filewrite"
                | "write_file"
                | "edit_file"
                | "strreplace"
                | "patch"
                | "create_file"
                | "update_file"
        ) {
            let target = args
                .get("file_path")
                .or_else(|| args.get("path"))
                .or_else(|| args.get("target_file"))
                .or_else(|| args.get("TargetFile"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            return NormalizedToolCall {
                action_type: ToolActionType::WriteFile,
                target_resource: target,
                raw_tool_name: tool_name,
                payload: args,
            };
        }

        // Read tools
        if matches!(
            name_lower.as_str(),
            "fileread"
                | "read_file"
                | "view"
                | "view_file"
                | "globtool"
                | "greptool"
                | "ls"
                | "list_dir"
                | "grep_search"
        ) {
            let target = args
                .get("file_path")
                .or_else(|| args.get("path"))
                .or_else(|| args.get("pattern"))
                .or_else(|| args.get("directory"))
                .or_else(|| args.get("AbsolutePath"))
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            return NormalizedToolCall {
                action_type: ToolActionType::ReadFile,
                target_resource: target,
                raw_tool_name: tool_name,
                payload: args,
            };
        }

        NormalizedToolCall {
            action_type: ToolActionType::Other,
            target_resource: String::new(),
            raw_tool_name: tool_name,
            payload: args,
        }
    }

    fn validate_plugin(&self, plugin_dir: &Path) -> AdapterValidationResult {
        let mut issues = Vec::new();

        // Check if plugin_dir or parent workspace has CLAUDE.md or .claude config
        let parent_ws = plugin_dir.parent().unwrap_or(plugin_dir);
        let has_claude_md =
            plugin_dir.join("CLAUDE.md").is_file() || parent_ws.join("CLAUDE.md").is_file();
        let has_claude_dir = plugin_dir.is_dir()
            || plugin_dir.join(".claude").is_dir()
            || parent_ws.join(".claude").is_dir();

        if !has_claude_md && !has_claude_dir {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                path: plugin_dir.display().to_string(),
                message: "No CLAUDE.md or .claude directory found in workspace.".to_string(),
            });
        }

        AdapterValidationResult {
            valid: issues
                .iter()
                .all(|i| i.severity != ValidationSeverity::Error),
            issues,
        }
    }

    fn handle_hook(&self, workspace: &Path, stdin_content: &str) -> (i32, String) {
        let content = stdin_content.trim();
        if content.is_empty() {
            let res = serde_json::json!({
                "decision": "deny",
                "reason": "Empty payload received on stdin."
            });
            return (1, res.to_string());
        }

        let payload: Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                let res = serde_json::json!({
                    "decision": "deny",
                    "reason": format!("Corrupt JSON payload on stdin: {e}")
                });
                return (1, res.to_string());
            }
        };

        let verdict = self.evaluate_invocation(workspace, &payload);
        if verdict.allowed {
            let res = serde_json::json!({
                "decision": "allow"
            });
            (0, res.to_string())
        } else {
            let res = serde_json::json!({
                "decision": "deny",
                "reason": verdict.reason
            });
            (1, res.to_string())
        }
    }
}
