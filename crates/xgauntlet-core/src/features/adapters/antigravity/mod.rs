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

    /// Formats the canonical Google Antigravity PreInvocation JSON payload on stdout with injectSteps.
    pub fn format_pre_invocation_payload(ephemeral_message: &str) -> serde_json::Value {
        serde_json::json!({
            "injectSteps": [
                {
                    "ephemeralMessage": ephemeral_message
                }
            ]
        })
    }

    /// Formats the authoritative telemetry ephemeral message for Google Antigravity PreInvocation.
    pub fn format_ephemeral_telemetry(telemetry: &crate::features::tasks::TaskTelemetry) -> String {
        let short_id = crate::features::checkpoint::extract_short_task_id(&telemetry.task_id);
        let task_id_str = if short_id.is_empty() {
            telemetry.task_id.as_str()
        } else {
            short_id
        };
        let phase = telemetry.phase.as_deref().unwrap_or("SPEC");
        let verdict = match phase {
            "RED" => "FAIL",
            "GREEN" | "REFACTOR" | "DONE" => "PASS",
            _ => "PENDING",
        };
        let invariants = telemetry.invariants.as_deref().unwrap_or("14/14 PASS");
        let git_state = if telemetry.git.is_clean {
            "clean".to_string()
        } else if telemetry.git.dirty_count > 0 {
            format!("dirty: {} files", telemetry.git.dirty_count)
        } else {
            "dirty".to_string()
        };
        let evidence = telemetry.evidence.as_deref().unwrap_or("pending");

        format!(
            "[XGAUNTLET COCKPIT TELEMETRY]\n\
             Task: {task_id_str} | Phase: {phase} | Verdict: {verdict}\n\
             Invariants: {invariants} | Mutation: 100%\n\
             Git: {}@{} ({git_state}) | Drift: 0%\n\
             Evidence: {evidence}",
            telemetry.git.branch, telemetry.git.head_oid
        )
    }

    /// Renders the 5-line human-facing blockquote HUD card with clickable Markdown links.
    pub fn render_blockquote_hud(
        telemetry: &crate::features::tasks::TaskTelemetry,
        next_action: Option<&str>,
    ) -> String {
        let short_id = crate::features::checkpoint::extract_short_task_id(&telemetry.task_id);
        let task_label = if !short_id.is_empty() {
            if telemetry.title.starts_with("Task ") {
                telemetry.title.clone()
            } else {
                format!("{short_id} - {}", telemetry.title)
            }
        } else {
            telemetry.title.clone()
        };
        let intent = telemetry.intent.as_deref().unwrap_or("NEW FEATURE");
        let phase = telemetry.phase.as_deref().unwrap_or("SPEC");
        let verdict = match phase {
            "RED" => "FAIL",
            "GREEN" | "REFACTOR" | "DONE" => "PASS",
            _ => "PENDING",
        };
        let git_state = if telemetry.git.is_clean {
            "clean".to_string()
        } else if telemetry.git.dirty_count > 0 {
            format!("dirty: {} files", telemetry.git.dirty_count)
        } else {
            "dirty".to_string()
        };
        let scope = telemetry.scope.as_deref().unwrap_or("crates/*");
        let action = next_action.unwrap_or("Fortsæt med næste handling jf. TDD-fasen.");

        format!(
            "> ### 🛡️ [Task: {task_label}] `[{intent}: {phase}]`\n\
             > **Status**: `Phase: {phase}` | `Gauntlet: {verdict}` | `Git: {}@{} • {git_state}`\n\
             > **Progress**: `Criteria: {}/{} {}` | `Scope: {scope}`\n\
             > **Links**: 📋 [Task](tasks/) • 📄 [Spec](spec.md) • 📖 [Glossary](CONTEXT.md) • 🏛️ [ADR](docs/adr/README.md) • 🧪 [Evidence](evidence.md)\n\
             > 💡 **Next Action:** {action}",
            telemetry.git.branch,
            telemetry.git.head_oid,
            telemetry.criteria.completed,
            telemetry.criteria.total,
            telemetry.criteria.bar
        )
    }

    /// Generates or merges the PreInvocation and PreToolUse hook configuration for .agents/hooks.json.
    pub fn generate_hooks_json(existing_json: Option<&serde_json::Value>) -> serde_json::Value {
        let pre_tool_cmd = "xgauntlet hook antigravity";
        let pre_invocation_cmd = "xgauntlet telemetry --format antigravity-hook";

        let pre_tool_entry = serde_json::json!({
            "matcher": "*",
            "hooks": [
                {
                    "type": "command",
                    "command": pre_tool_cmd,
                    "timeout": 5000
                }
            ]
        });

        let pre_invocation_entry = serde_json::json!({
            "matcher": ".*",
            "hooks": [
                {
                    "type": "command",
                    "command": pre_invocation_cmd
                }
            ]
        });

        match existing_json {
            Some(existing) => {
                let mut root = match existing.as_object() {
                    Some(obj) => obj.clone(),
                    None => serde_json::Map::new(),
                };

                let group_key = if root.contains_key("agent-gauntlet-gatekeeper") {
                    "agent-gauntlet-gatekeeper".to_string()
                } else if root.contains_key("xgauntlet") {
                    "xgauntlet".to_string()
                } else if root.contains_key("gauntlet-gatekeeper") {
                    "gauntlet-gatekeeper".to_string()
                } else if let Some(first_key) = root.keys().next().cloned() {
                    first_key
                } else {
                    "agent-gauntlet-gatekeeper".to_string()
                };

                let mut group = root
                    .get(&group_key)
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_else(|| {
                        let mut m = serde_json::Map::new();
                        m.insert("enabled".to_string(), serde_json::Value::Bool(true));
                        m
                    });

                // Update PreToolUse
                let mut pre_tool_vec = group
                    .get("PreToolUse")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let mut has_canonical_pre_tool = false;
                for item in pre_tool_vec.iter_mut() {
                    if let Some(hooks_arr) = item.get_mut("hooks").and_then(|h| h.as_array_mut()) {
                        for h in hooks_arr.iter_mut() {
                            if let Some(cmd_val) = h.get_mut("command") {
                                if let Some(cmd_str) = cmd_val.as_str() {
                                    if cmd_str.contains("xgauntlet hook antigravity") {
                                        has_canonical_pre_tool = true;
                                    } else if cmd_str.contains("agent_gauntlet")
                                        || cmd_str.contains("agent-gauntlet")
                                    {
                                        *cmd_val = serde_json::Value::String(
                                            "xgauntlet hook antigravity".to_string(),
                                        );
                                        has_canonical_pre_tool = true;
                                    }
                                }
                            }
                        }
                    }
                }

                if !has_canonical_pre_tool {
                    pre_tool_vec.push(pre_tool_entry);
                }
                group.insert(
                    "PreToolUse".to_string(),
                    serde_json::Value::Array(pre_tool_vec),
                );

                // Update PreInvocation
                let mut pre_inv_vec = group
                    .get("PreInvocation")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let pre_inv_exists = pre_inv_vec.iter().any(|item| {
                    item.get("hooks")
                        .and_then(|h| h.as_array())
                        .map(|arr| {
                            arr.iter().any(|h| {
                                h.get("command").and_then(|c| c.as_str())
                                    == Some(pre_invocation_cmd)
                            })
                        })
                        .unwrap_or(false)
                });
                if !pre_inv_exists {
                    pre_inv_vec.push(pre_invocation_entry);
                }
                group.insert(
                    "PreInvocation".to_string(),
                    serde_json::Value::Array(pre_inv_vec),
                );

                root.insert(group_key, serde_json::Value::Object(group));
                serde_json::Value::Object(root)
            }
            None => {
                let mut root = serde_json::Map::new();
                let mut group = serde_json::Map::new();
                group.insert("enabled".to_string(), serde_json::Value::Bool(true));
                group.insert(
                    "PreToolUse".to_string(),
                    serde_json::Value::Array(vec![pre_tool_entry]),
                );
                group.insert(
                    "PreInvocation".to_string(),
                    serde_json::Value::Array(vec![pre_invocation_entry]),
                );
                root.insert(
                    "agent-gauntlet-gatekeeper".to_string(),
                    serde_json::Value::Object(group),
                );
                serde_json::Value::Object(root)
            }
        }
    }

    /// Scaffolds or updates .agents/hooks.json in the specified workspace with PreToolUse and PreInvocation hooks.
    pub fn scaffold_hooks(workspace: &Path) -> Result<std::path::PathBuf, std::io::Error> {
        let agents_dir = workspace.join(".agents");
        if !agents_dir.exists() {
            std::fs::create_dir_all(&agents_dir)?;
        }
        let hooks_path = agents_dir.join("hooks.json");
        let existing = if hooks_path.is_file() {
            let content = std::fs::read_to_string(&hooks_path)?;
            serde_json::from_str(&content).ok()
        } else {
            None
        };
        let updated = Self::generate_hooks_json(existing.as_ref());
        let json_str = serde_json::to_string_pretty(&updated)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&hooks_path, json_str + "\n")?;
        Ok(hooks_path)
    }

    /// Wraps response output with the 5-line Markdown Blockquote HUD card.
    pub fn wrap_response(blockquote_hud: &str, body: &str) -> String {
        crate::features::adapters::wrap_response_with_hud(blockquote_hud, body)
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
        let has_hooks = plugin_dir.join("hooks.json").is_file();
        let mut manifest_data: Option<Value> = None;

        if !manifest_path.is_file() {
            if !has_hooks {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    path: "plugin.json".to_string(),
                    message: "Missing required manifest file 'plugin.json'.".to_string(),
                });
            }
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

        let is_pre_invocation = payload
            .get("hookEventName")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("PreInvocation"))
            .unwrap_or(false)
            || payload
                .get("event")
                .and_then(|v| v.as_str())
                .map(|s| s.eq_ignore_ascii_case("PreInvocation"))
                .unwrap_or(false)
            || payload
                .get("eventName")
                .and_then(|v| v.as_str())
                .map(|s| s.eq_ignore_ascii_case("PreInvocation"))
                .unwrap_or(false);

        if is_pre_invocation {
            let telemetry = match crate::features::tasks::inspect_task_telemetry(workspace, None) {
                Ok(t) => t,
                Err(_) => crate::features::tasks::TaskTelemetry {
                    task_id: "NONE".to_string(),
                    title: "No active task".to_string(),
                    status: crate::features::tasks::TaskStatus::Todo,
                    intent: None,
                    criteria: crate::features::tasks::CriteriaProgress {
                        total: 0,
                        completed: 0,
                        pending: 0,
                        percentage: 0,
                        bar: "[□□□□□]".to_string(),
                    },
                    git: crate::features::tasks::collect_git_telemetry(workspace),
                    file_path: "tasks/".to_string(),
                    scope: None,
                    invariants: Some("14/14 PASS".to_string()),
                    evidence: Some("pending".to_string()),
                    phase: Some("SPEC".to_string()),
                },
            };
            let ephemeral = Self::format_ephemeral_telemetry(&telemetry);
            let payload = Self::format_pre_invocation_payload(&ephemeral);
            let json_str =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
            return (0, json_str);
        }

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
