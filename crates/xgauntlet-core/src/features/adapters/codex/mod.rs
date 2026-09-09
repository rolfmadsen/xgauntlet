//! OpenAI Codex vertical slice harness adapter.
//!
//! Translates OpenAI function and tool calling payloads into
//! canonical gauntlet operations and enforces fail-closed policy gates.

use serde_json::Value;
use std::path::Path;

use crate::features::adapters::models::{
    AdapterValidationResult, HarnessAdapter, NormalizedToolCall, ValidationIssue,
    ValidationSeverity,
};
use crate::features::policy::ToolActionType;

#[derive(Debug, Default, Clone)]
pub struct CodexAdapter;

impl CodexAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Formats the canonical OpenAI Codex PostToolUse JSON payload on stdout.
    pub fn format_post_tool_use_payload(_box_card: &str) -> serde_json::Value {
        unimplemented!("format_post_tool_use_payload is not yet implemented")
    }

    /// Generates or merges the PostToolUse hook configuration for .codex/hooks.json.
    pub fn generate_hooks_json(_existing_json: Option<&serde_json::Value>) -> serde_json::Value {
        unimplemented!("generate_hooks_json is not yet implemented")
    }

    /// Scaffolds or updates .codex/hooks.json in the specified workspace with PostToolUse telemetry hook.
    pub fn scaffold_hooks(_workspace: &Path) -> Result<std::path::PathBuf, std::io::Error> {
        unimplemented!("scaffold_hooks is not yet implemented")
    }

    /// Wraps response output (e.g. from verify or checkpoint) with the Variant B Box-Drawing Telemetry Card.
    pub fn wrap_response(_box_card: &str, _body: &str) -> String {
        unimplemented!("wrap_response is not yet implemented")
    }
}

impl HarnessAdapter for CodexAdapter {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn normalize_tool_call(&self, payload: &Value) -> NormalizedToolCall {
        let mut tool_name = String::new();
        let mut raw_args = Value::Object(serde_json::Map::new());

        // 1. OpenAI 1.x Tool Call format: {"type": "function", "function": {"name": ..., "arguments": ...}}
        if payload.get("type").and_then(|t| t.as_str()) == Some("function") {
            if let Some(func) = payload.get("function").and_then(|f| f.as_object()) {
                if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                    tool_name = name.trim().to_string();
                }
                if let Some(args) = func.get("arguments") {
                    raw_args = args.clone();
                }
            }
        }
        // 2. Direct OpenAI Function format: {"name": "...", "arguments": ...}
        else if let Some(name) = payload.get("name").and_then(|n| n.as_str()) {
            tool_name = name.trim().to_string();
            if let Some(args) = payload
                .get("arguments")
                .or_else(|| payload.get("parameters"))
            {
                raw_args = args.clone();
            }
        }
        // 3. Flat format: {"tool_name": "...", "tool_input": ...}
        else if let Some(name) = payload.get("tool_name").and_then(|n| n.as_str()) {
            tool_name = name.trim().to_string();
            if let Some(input) = payload.get("tool_input") {
                raw_args = input.clone();
            }
        }

        // Handle stringified JSON arguments (standard OpenAI API behavior)
        let resolved_args: Value = match raw_args {
            Value::String(ref s) => serde_json::from_str(s).unwrap_or(raw_args),
            other => other,
        };

        let name_lower = tool_name.to_ascii_lowercase();

        // Execution tools
        if matches!(
            name_lower.as_str(),
            "bash" | "execute_code" | "run_command" | "shell" | "exec" | "terminal" | "command"
        ) {
            let cmd = resolved_args
                .get("command")
                .or_else(|| resolved_args.get("cmd"))
                .or_else(|| resolved_args.get("code"))
                .or_else(|| resolved_args.get("CommandLine"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            return NormalizedToolCall {
                action_type: ToolActionType::ExecuteCommand,
                target_resource: cmd,
                raw_tool_name: tool_name,
                payload: resolved_args,
            };
        }

        // Write tools
        if matches!(
            name_lower.as_str(),
            "apply_patch" | "write_file" | "edit_file" | "save_file" | "create_file" | "patch"
        ) {
            let target = resolved_args
                .get("path")
                .or_else(|| resolved_args.get("filename"))
                .or_else(|| resolved_args.get("file_path"))
                .or_else(|| resolved_args.get("TargetFile"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            return NormalizedToolCall {
                action_type: ToolActionType::WriteFile,
                target_resource: target,
                raw_tool_name: tool_name,
                payload: resolved_args,
            };
        }

        // Read tools
        if matches!(
            name_lower.as_str(),
            "read_file" | "list_files" | "view_file" | "search_files" | "read_dir" | "find_by_name"
        ) {
            let target = resolved_args
                .get("path")
                .or_else(|| resolved_args.get("directory"))
                .or_else(|| resolved_args.get("query"))
                .or_else(|| resolved_args.get("AbsolutePath"))
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            return NormalizedToolCall {
                action_type: ToolActionType::ReadFile,
                target_resource: target,
                raw_tool_name: tool_name,
                payload: resolved_args,
            };
        }

        NormalizedToolCall {
            action_type: ToolActionType::Other,
            target_resource: String::new(),
            raw_tool_name: tool_name,
            payload: resolved_args,
        }
    }

    fn validate_plugin(&self, plugin_dir: &Path) -> AdapterValidationResult {
        let mut issues = Vec::new();

        let plugin_json = plugin_dir.join("ai-plugin.json");
        if !plugin_json.is_file() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                path: "ai-plugin.json".to_string(),
                message: "No 'ai-plugin.json' found in target directory.".to_string(),
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
