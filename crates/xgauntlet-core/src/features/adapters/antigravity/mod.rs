//! Google Antigravity IDE vertical slice harness adapter.
//!
//! Translates Google Antigravity IDE tool calls and lifecycle hooks into
//! canonical gauntlet operations and enforces fail-closed policy gates.

use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::features::adapters::models::{
    AdapterValidationResult, HarnessAdapter, NormalizedToolCall, ValidationIssue,
    ValidationSeverity,
};
use crate::features::policy::ToolActionType;

pub const VALID_ANTIGRAVITY_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PreInvocation",
    "PostInvocation",
    "Stop",
];

#[derive(Debug, Default, Clone)]
pub struct AntigravityAdapter;

impl AntigravityAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl HarnessAdapter for AntigravityAdapter {
    fn name(&self) -> &'static str {
        "antigravity"
    }

    fn normalize_tool_call(&self, payload: &Value) -> NormalizedToolCall {
        let mut tool_name = String::new();
        let mut args = Value::Object(serde_json::Map::new());

        // 1. Canonical Antigravity PreToolUse payload format
        if let Some(tool_call) = payload.get("toolCall").and_then(|t| t.as_object()) {
            if let Some(name) = tool_call.get("name").and_then(|n| n.as_str()) {
                tool_name = name.trim().to_string();
            }
            if let Some(call_args) = tool_call.get("args") {
                args = call_args.clone();
            }
        }
        // 2. Legacy / flat format
        else if let Some(name) = payload.get("tool_name").and_then(|n| n.as_str()) {
            tool_name = name.trim().to_string();
            if let Some(input) = payload.get("tool_input") {
                args = input.clone();
            }
        }

        if tool_name == "run_command" {
            let cmd = args
                .get("CommandLine")
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

        if matches!(
            tool_name.as_str(),
            "write_to_file" | "replace_file_content" | "multi_replace_file_content"
        ) {
            let target = args
                .get("TargetFile")
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

        if matches!(
            tool_name.as_str(),
            "view_file" | "list_dir" | "grep_search" | "find_by_name" | "read_url_content"
        ) {
            let target = args
                .get("AbsolutePath")
                .or_else(|| args.get("SearchPath"))
                .or_else(|| args.get("SearchDirectory"))
                .or_else(|| args.get("DirectoryPath"))
                .or_else(|| args.get("Url"))
                .and_then(|v| v.as_str())
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

        if !plugin_dir.is_dir() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                path: plugin_dir.display().to_string(),
                message: format!(
                    "Plugin directory '{}' does not exist.",
                    plugin_dir.display()
                ),
            });
            return AdapterValidationResult::failure(issues);
        }

        // 1. Validate plugin.json
        let manifest_path = plugin_dir.join("plugin.json");
        let mut manifest_data: Option<Value> = None;

        if !manifest_path.is_file() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                path: "plugin.json".to_string(),
                message: "Missing required manifest file 'plugin.json'.".to_string(),
            });
        } else {
            match fs::read_to_string(&manifest_path) {
                Ok(content) => match serde_json::from_str::<Value>(&content) {
                    Ok(val) => {
                        if !val.is_object() {
                            issues.push(ValidationIssue {
                                severity: ValidationSeverity::Error,
                                path: "plugin.json".to_string(),
                                message: "'plugin.json' must be a JSON object.".to_string(),
                            });
                        } else {
                            manifest_data = Some(val);
                        }
                    }
                    Err(e) => {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            path: "plugin.json".to_string(),
                            message: format!("Invalid JSON in 'plugin.json': {e}"),
                        });
                    }
                },
                Err(e) => {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Error,
                        path: "plugin.json".to_string(),
                        message: format!("Could not read 'plugin.json': {e}"),
                    });
                }
            }
        }

        if let Some(manifest) = manifest_data {
            let name_valid = manifest
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            if !name_valid {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    path: "plugin.json".to_string(),
                    message: "Missing required field 'name' in 'plugin.json'.".to_string(),
                });
            }

            let version_valid = manifest
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            if !version_valid {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    path: "plugin.json".to_string(),
                    message: "Missing required field 'version' in 'plugin.json'.".to_string(),
                });
            }

            // 2. Validate declared skills
            if let Some(skills) = manifest.get("skills").and_then(|s| s.as_array()) {
                for skill in skills {
                    if let Some(skill_name) = skill.as_str() {
                        let skill_file =
                            plugin_dir.join("skills").join(skill_name).join("SKILL.md");

                        if !skill_file.is_file() {
                            issues.push(ValidationIssue {
                                severity: ValidationSeverity::Error,
                                path: format!("skills/{}/SKILL.md", skill_name),
                                message: format!(
                                    "Declared skill '{}' is missing 'SKILL.md' file.",
                                    skill_name
                                ),
                            });
                        } else {
                            match fs::read_to_string(&skill_file) {
                                Ok(content) => {
                                    let has_frontmatter = content.starts_with("---")
                                        && content.contains("name:")
                                        && content.contains("description:");
                                    if !has_frontmatter {
                                        issues.push(ValidationIssue {
                                            severity: ValidationSeverity::Error,
                                            path: format!("skills/{}/SKILL.md", skill_name),
                                            message: format!(
                                                "Skill '{}' is missing valid YAML frontmatter (name, description).",
                                                skill_name
                                            ),
                                        });
                                    }
                                }
                                Err(e) => {
                                    issues.push(ValidationIssue {
                                        severity: ValidationSeverity::Error,
                                        path: format!("skills/{}/SKILL.md", skill_name),
                                        message: format!("Failed to read SKILL.md: {e}"),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Validate hooks.json if present
        let hooks_path = plugin_dir.join("hooks.json");
        if hooks_path.is_file() {
            match fs::read_to_string(&hooks_path) {
                Ok(content) => match serde_json::from_str::<Value>(&content) {
                    Ok(val) => {
                        if let Some(root_obj) = val.as_object() {
                            if root_obj.is_empty() {
                                issues.push(ValidationIssue {
                                    severity: ValidationSeverity::Error,
                                    path: "hooks.json".to_string(),
                                    message: "'hooks.json' root object cannot be empty."
                                        .to_string(),
                                });
                            } else {
                                for (hook_name, hook_config) in root_obj {
                                    let Some(h_obj) = hook_config.as_object() else {
                                        issues.push(ValidationIssue {
                                            severity: ValidationSeverity::Error,
                                            path: "hooks.json".to_string(),
                                            message: format!("Configuration for hook '{hook_name}' must be an object."),
                                        });
                                        continue;
                                    };

                                    let has_valid_event = h_obj
                                        .keys()
                                        .any(|k| VALID_ANTIGRAVITY_EVENTS.contains(&k.as_str()));

                                    if !has_valid_event {
                                        issues.push(ValidationIssue {
                                            severity: ValidationSeverity::Error,
                                            path: "hooks.json".to_string(),
                                            message: format!(
                                                "Hook '{hook_name}' must define at least one lifecycle event from {:?}",
                                                VALID_ANTIGRAVITY_EVENTS
                                            ),
                                        });
                                    }
                                }
                            }
                        } else {
                            issues.push(ValidationIssue {
                                severity: ValidationSeverity::Error,
                                path: "hooks.json".to_string(),
                                message: "'hooks.json' root must be a JSON object.".to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            path: "hooks.json".to_string(),
                            message: format!("Invalid JSON in 'hooks.json': {e}"),
                        });
                    }
                },
                Err(e) => {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Error,
                        path: "hooks.json".to_string(),
                        message: format!("Failed to read 'hooks.json': {e}"),
                    });
                }
            }
        }

        let has_errors = issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error);
        AdapterValidationResult {
            valid: !has_errors,
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
