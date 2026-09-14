//! Mistral Vibe vertical slice harness adapter.
//!
//! Translates Mistral Vibe tool calls and lifecycle hooks (pre_tool, post_tool) into
//! canonical gauntlet operations and enforces fail-closed policy gates.
//! Reference: https://docs.mistral.ai/vibe/code/cli/hooks

use serde_json::Value;
use std::path::Path;

use crate::features::adapters::models::{
    AdapterValidationResult, HarnessAdapter, NormalizedToolCall, ValidationIssue,
    ValidationSeverity,
};
use crate::features::policy::ToolActionType;

#[derive(Debug, Default, Clone)]
pub struct MistralAdapter;

impl MistralAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Formats the canonical Mistral Vibe PostToolUse JSON payload on stdout.
    /// In Mistral Vibe, post_tool hook returns:
    /// {
    ///   "hook_specific_output": {
    ///     "additional_context": "<telemetry>"
    ///   }
    /// }
    pub fn format_post_tool_use_payload(box_card: &str) -> serde_json::Value {
        serde_json::json!({
            "hook_specific_output": {
                "additional_context": box_card
            }
        })
    }

    /// Generates or merges the hooks TOML configuration for .vibe/hooks.toml.
    pub fn generate_hooks_toml(existing_toml: Option<&str>) -> String {
        let gatekeeper_block = r#"[[hooks]]
name = "xgauntlet-gatekeeper"
type = "pre_tool"
match = "re:^bash|write_file|edit$"
command = "xgauntlet hook --harness mistral"
timeout = 60.0
strict = true
description = "xGauntlet Zero Ambient Authority policy engine gatekeeper.""#;

        let hud_block = r#"[[hooks]]
name = "xgauntlet-hud"
type = "post_tool"
match = "re:^bash|write_file|edit$"
command = "xgauntlet telemetry --format mistral-hook"
timeout = 60.0
strict = false
description = "xGauntlet dynamic cockpit HUD telemetry injection.""#;

        match existing_toml {
            Some(existing) if !existing.trim().is_empty() => {
                let mut output = existing.trim_end().to_string();
                let has_gatekeeper = existing.contains("xgauntlet-gatekeeper");
                let has_hud = existing.contains("xgauntlet-hud");

                if !has_gatekeeper {
                    output.push_str("\n\n");
                    output.push_str(gatekeeper_block);
                }
                if !has_hud {
                    output.push_str("\n\n");
                    output.push_str(hud_block);
                }
                output.push('\n');
                output
            }
            _ => format!("{}\n\n{}\n", gatekeeper_block, hud_block),
        }
    }

    /// Scaffolds or updates .vibe/hooks.toml in the specified workspace.
    pub fn scaffold_hooks(workspace: &Path) -> Result<std::path::PathBuf, std::io::Error> {
        let vibe_dir = workspace.join(".vibe");
        if !vibe_dir.exists() {
            std::fs::create_dir_all(&vibe_dir)?;
        }
        let hooks_path = vibe_dir.join("hooks.toml");
        let existing = if hooks_path.is_file() {
            std::fs::read_to_string(&hooks_path).ok()
        } else {
            None
        };
        let updated = Self::generate_hooks_toml(existing.as_deref());
        std::fs::write(&hooks_path, updated)?;
        Ok(hooks_path)
    }

    /// Wraps response output (e.g. from verify or checkpoint) with the Variant B Box-Drawing Telemetry Card.
    pub fn wrap_response(box_card: &str, body: &str) -> String {
        crate::features::adapters::wrap_response_with_hud(box_card, body)
    }
}

impl HarnessAdapter for MistralAdapter {
    fn name(&self) -> &'static str {
        "mistral"
    }

    fn normalize_tool_call(&self, payload: &Value) -> NormalizedToolCall {
        let mut tool_name = String::new();
        let mut raw_args = Value::Object(serde_json::Map::new());

        // 1. Mistral Vibe hook format: {"tool_name": "...", "tool_input": {...}}
        if let Some(name) = payload.get("tool_name").and_then(|n| n.as_str()) {
            tool_name = name.trim().to_string();
            if let Some(input) = payload.get("tool_input") {
                raw_args = input.clone();
            }
        }
        // 2. OpenAI-compatible function calling format (used by Mistral API):
        //    {"type": "function", "function": {"name": ..., "arguments": ...}}
        else if payload.get("type").and_then(|t| t.as_str()) == Some("function") {
            if let Some(func) = payload.get("function").and_then(|f| f.as_object()) {
                if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                    tool_name = name.trim().to_string();
                }
                if let Some(args) = func.get("arguments") {
                    raw_args = args.clone();
                }
            }
        }
        // 3. Direct function format: {"name": "...", "arguments": ...}
        else if let Some(name) = payload.get("name").and_then(|n| n.as_str()) {
            tool_name = name.trim().to_string();
            if let Some(args) = payload
                .get("arguments")
                .or_else(|| payload.get("parameters"))
            {
                raw_args = args.clone();
            }
        }

        // If raw_args is a JSON string, try to parse it
        if let Some(args_str) = raw_args.as_str() {
            if let Ok(parsed) = serde_json::from_str::<Value>(args_str) {
                raw_args = parsed;
            }
        }

        let normalized_name = tool_name.to_ascii_lowercase();
        let (action_type, target_resource) = match normalized_name.as_str() {
            "bash" | "sh" | "shell" | "exec" | "command" | "run_command" => {
                let cmd = raw_args
                    .get("command")
                    .or_else(|| raw_args.get("cmd"))
                    .or_else(|| raw_args.get("CommandLine"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (ToolActionType::ExecuteCommand, cmd)
            }
            "write_file" | "write" | "edit" | "str_replace" | "file_edit" | "patch" => {
                let path = raw_args
                    .get("path")
                    .or_else(|| raw_args.get("file_path"))
                    .or_else(|| raw_args.get("filename"))
                    .or_else(|| raw_args.get("TargetFile"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (ToolActionType::WriteFile, path)
            }
            "read" | "read_file" | "view" | "cat" | "view_file" => {
                let path = raw_args
                    .get("path")
                    .or_else(|| raw_args.get("file_path"))
                    .or_else(|| raw_args.get("filename"))
                    .or_else(|| raw_args.get("AbsolutePath"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (ToolActionType::ReadFile, path)
            }
            _ => (ToolActionType::Other, String::new()),
        };

        NormalizedToolCall {
            action_type,
            raw_tool_name: tool_name,
            target_resource,
            payload: raw_args,
        }
    }

    fn validate_plugin(&self, plugin_dir: &Path) -> AdapterValidationResult {
        let mut issues = Vec::new();
        let hooks_toml = plugin_dir.join("hooks.toml");
        let config_toml = plugin_dir.join("config.toml");

        if !hooks_toml.exists() && !config_toml.exists() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                path: plugin_dir.display().to_string(),
                message: format!(
                    "Mistral Vibe directory '{}' contains neither hooks.toml nor config.toml",
                    plugin_dir.display()
                ),
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
        let content = crate::features::adapters::clean_stdin(stdin_content);
        if content.is_empty() {
            let err_resp = serde_json::json!({
                "decision": "deny",
                "reason": "Empty payload received on stdin."
            });
            return (1, serde_json::to_string(&err_resp).unwrap());
        }

        let payload: Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                let err_resp = serde_json::json!({
                    "decision": "deny",
                    "reason": format!("Malformed JSON payload on stdin: {e}")
                });
                return (1, serde_json::to_string(&err_resp).unwrap());
            }
        };

        let event = payload
            .get("hook_event_name")
            .and_then(|v| v.as_str())
            .unwrap_or("pre_tool");

        match event {
            "pre_tool" => {
                let verdict = self.evaluate_invocation(workspace, &payload);
                if verdict.allowed {
                    let resp = serde_json::json!({
                        "decision": "allow"
                    });
                    (0, serde_json::to_string(&resp).unwrap())
                } else {
                    let resp = serde_json::json!({
                        "decision": "deny",
                        "reason": verdict.reason
                    });
                    (0, serde_json::to_string(&resp).unwrap())
                }
            }
            "post_tool" => {
                let resp = Self::format_post_tool_use_payload("");
                (0, serde_json::to_string(&resp).unwrap())
            }
            "post_agent" => {
                let resp = serde_json::json!({
                    "decision": "allow"
                });
                (0, serde_json::to_string(&resp).unwrap())
            }
            other => {
                let resp = serde_json::json!({
                    "decision": "allow",
                    "system_message": format!("Ignored event '{other}'")
                });
                (0, serde_json::to_string(&resp).unwrap())
            }
        }
    }
}
