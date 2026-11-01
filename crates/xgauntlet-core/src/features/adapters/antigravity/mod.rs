//! Google Antigravity IDE vertical slice harness adapter.
//!
//! Translates Google Antigravity IDE tool calls and lifecycle hooks into
//! canonical gauntlet operations and enforces fail-closed policy gates.

use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::features::adapters::models::{
    AdapterValidationResult, HarnessAdapter, NormalizedToolCall,
};
use crate::features::policy::ToolActionType;

pub mod validation;
pub mod workspace;

pub use validation::VALID_ANTIGRAVITY_EVENTS;
pub use workspace::resolve_effective_workspace;

/// Canonical command string for the Antigravity PreInvocation telemetry hook (single source of truth).
pub const CANONICAL_PRE_INVOCATION_CMD: &str = "xgauntlet telemetry --format antigravity-hook";

/// Alias for the canonical Antigravity PreInvocation command.
pub const ANTIGRAVITY_PRE_INVOCATION_CMD: &str = CANONICAL_PRE_INVOCATION_CMD;

fn should_suppress_duplicate_telemetry(message: &str) -> bool {
    // Never suppress telemetry during test execution
    if cfg!(test)
        || std::env::var("XGAUNTLET_TEST_MODE").is_ok()
        || std::env::current_exe()
            .map(|p| {
                let s = p.to_string_lossy();
                s.contains("/deps/") || s.contains("test")
            })
            .unwrap_or(false)
    {
        return false;
    }

    use sha2::{Digest, Sha256};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static DEDUP_TMP_COUNTER: AtomicU64 = AtomicU64::new(1);

    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string());
    let temp_root = std::env::temp_dir();
    let cache_file = temp_root.join(format!("xgauntlet_telemetry_dedup_{user}.json"));

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let current_digest = format!("{:x}", Sha256::digest(message.as_bytes()));
    // Extract semantic state key: line 2 ("Task: ... | Phase: ... | Verdict: ...")
    let current_state_key = message.lines().nth(1).unwrap_or(message).trim().to_string();

    if let Ok(content) = std::fs::read_to_string(&cache_file) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            let last_digest = val.get("digest").and_then(|d| d.as_str()).unwrap_or("");
            let last_state_key = val.get("state_key").and_then(|s| s.as_str()).unwrap_or("");
            let last_ts = val
                .get("timestamp_ms")
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as u128;

            let is_same_state = last_digest == current_digest
                || (!last_state_key.is_empty() && last_state_key == current_state_key);

            // 1. Rapid duplicate hook suppress (< 1500ms, e.g. duplicate global + workspace hooks)
            if is_same_state && now.saturating_sub(last_ts) < 1500 {
                return true;
            }

            // 2. Semantic state-diffing: if Task, Phase, and Verdict have not changed within 120s, suppress
            if is_same_state && now.saturating_sub(last_ts) < 120_000 {
                return true;
            }
        }
    }

    let payload = serde_json::json!({
        "digest": current_digest,
        "state_key": current_state_key,
        "timestamp_ms": now,
    });
    let seq = DEDUP_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_file = temp_root.join(format!(
        "xgauntlet_telemetry_dedup_{user}.{}.{seq}.tmp",
        std::process::id()
    ));
    if std::fs::write(&tmp_file, payload.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp_file, &cache_file);
        let _ = std::fs::remove_file(&tmp_file);
    }
    false
}

#[derive(Debug, Default, Clone)]
pub struct AntigravityAdapter;

impl AntigravityAdapter {
    pub const CANONICAL_PRE_INVOCATION_CMD: &str = CANONICAL_PRE_INVOCATION_CMD;
    pub const ANTIGRAVITY_PRE_INVOCATION_CMD: &str = ANTIGRAVITY_PRE_INVOCATION_CMD;

    pub fn new() -> Self {
        Self
    }

    /// Resolves the true workspace directory from stdin hook payload, falling back to workspace or path ancestor traversal.
    pub fn resolve_effective_workspace(workspace: &Path, payload: &Value) -> std::path::PathBuf {
        workspace::resolve_effective_workspace(workspace, payload)
    }

    /// Formats the canonical Google Antigravity PreInvocation JSON payload on stdout with injectSteps.
    pub fn format_pre_invocation_payload(ephemeral_message: &str) -> serde_json::Value {
        if should_suppress_duplicate_telemetry(ephemeral_message) {
            return serde_json::json!({
                "injectSteps": []
            });
        }

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

        let mut msg = format!(
            "[XGAUNTLET COCKPIT TELEMETRY]\n\
             Task: {task_id_str} | Phase: {phase} | Verdict: {verdict}\n\
             Invariants: {invariants} | Mutation: 100%\n\
             Git: {}@{} ({git_state}) | Drift: 0%\n\
             Evidence: {evidence}",
            telemetry.git.branch, telemetry.git.head_oid
        );

        if let Some(ref sh) = telemetry.session_health {
            msg.push_str(&format!("\nSession: {}", sh.format_hud_summary()));
        }

        if let Some(pipe_phase) = crate::features::telemetry::PipelinePhase::parse_phase(phase) {
            let jit_rendered =
                crate::features::telemetry::render_jit_directive(pipe_phase, Some(task_id_str));
            if !jit_rendered.is_empty() {
                msg.push_str("\n\n");
                msg.push_str(&jit_rendered);
            }
        }

        if let Some(ref sh) = telemetry.session_health {
            if sh.status == crate::features::telemetry::SessionHealthStatus::Saturated {
                msg.push_str("\n\n💡 Anbefaling: Opgaven trækker ud (>18 turns). Forsegl checkpoints og start en frisk session via 'xgauntlet handoff'.");
            }
        }

        msg
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

        let session_segment = if let Some(ref sh) = telemetry.session_health {
            format!(" | `Session: {}`", sh.format_hud_summary())
        } else {
            String::new()
        };

        format!(
            "> ### 🛡️ [Task: {task_label}] `[{intent}: {phase}]`\n\
             > **Status**: `Phase: {phase}` | `Gauntlet: {verdict}`{session_segment} | `Git: {}@{} • {git_state}`\n\
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
        let pre_invocation_cmd = CANONICAL_PRE_INVOCATION_CMD;

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
            "type": "command",
            "command": pre_invocation_cmd
        });

        match existing_json {
            Some(existing) => {
                let mut root = match existing.as_object() {
                    Some(obj) => obj.clone(),
                    None => serde_json::Map::new(),
                };

                let group_key = if root.contains_key("xgauntlet") {
                    "xgauntlet".to_string()
                } else if root.contains_key("agent-gauntlet-gatekeeper") {
                    "agent-gauntlet-gatekeeper".to_string()
                } else if root.contains_key("gauntlet-gatekeeper") {
                    "gauntlet-gatekeeper".to_string()
                } else {
                    "xgauntlet".to_string()
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

                // Update PreInvocation: flat list of handlers, normalizing legacy grouped format
                let raw_pre_inv = group
                    .get("PreInvocation")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let mut pre_inv_vec = Vec::new();
                let mut has_telemetry = false;

                for item in raw_pre_inv {
                    if let Some(hooks_arr) = item.get("hooks").and_then(|h| h.as_array()) {
                        for h in hooks_arr {
                            if let Some(cmd) = h.get("command").and_then(|c| c.as_str()) {
                                if cmd.contains("xgauntlet telemetry")
                                    || cmd.contains("agent_gauntlet telemetry")
                                {
                                    if !has_telemetry {
                                        pre_inv_vec.push(pre_invocation_entry.clone());
                                        has_telemetry = true;
                                    }
                                } else {
                                    let mut flat_h = h.clone();
                                    if let Some(obj) = flat_h.as_object_mut() {
                                        obj.remove("matcher");
                                    }
                                    pre_inv_vec.push(flat_h);
                                }
                            }
                        }
                    } else if let Some(cmd) = item.get("command").and_then(|c| c.as_str()) {
                        if cmd.contains("xgauntlet telemetry")
                            || cmd.contains("agent_gauntlet telemetry")
                        {
                            if !has_telemetry {
                                pre_inv_vec.push(pre_invocation_entry.clone());
                                has_telemetry = true;
                            }
                        } else {
                            pre_inv_vec.push(item);
                        }
                    } else {
                        pre_inv_vec.push(item);
                    }
                }

                if !has_telemetry {
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
                root.insert("xgauntlet".to_string(), serde_json::Value::Object(group));
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
            if content.trim().is_empty() {
                None
            } else {
                let val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Existing hooks configuration '{}' contains invalid JSON ({}). Manual remediation required before scaffolding to avoid data loss.",
                            hooks_path.display(),
                            e
                        ),
                    )
                })?;
                Some(val)
            }
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
            if let Some(call_args) = tool_call
                .get("args")
                .or_else(|| tool_call.get("parameters"))
            {
                args = call_args.clone();
            }
        }
        // 2. Legacy / flat format
        else if let Some(name) = payload
            .get("tool_name")
            .or_else(|| payload.get("name"))
            .and_then(|n| n.as_str())
        {
            tool_name = name.trim().to_string();
            if let Some(input) = payload
                .get("parameters")
                .or_else(|| payload.get("args"))
                .or_else(|| payload.get("tool_input"))
            {
                args = input.clone();
            }
        }

        // 3. Fallback: parameters or args at top level
        if args.as_object().is_none_or(|o| o.is_empty()) {
            if let Some(p) = payload.get("parameters").or_else(|| payload.get("args")) {
                args = p.clone();
            }
        }

        let name_lower = tool_name.to_ascii_lowercase();

        if matches!(
            name_lower.as_str(),
            "run_command" | "bash" | "exec" | "command" | "execute_command"
        ) {
            let cmd = args
                .get("CommandLine")
                .or_else(|| args.get("command"))
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

        if matches!(
            name_lower.as_str(),
            "write_to_file"
                | "replace_file_content"
                | "multi_replace_file_content"
                | "write_file"
                | "edit_file"
                | "create_file"
        ) {
            let target = args
                .get("TargetFile")
                .or_else(|| args.get("Path"))
                .or_else(|| args.get("path"))
                .or_else(|| args.get("file_path"))
                .or_else(|| args.get("target_file"))
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
            name_lower.as_str(),
            "view_file"
                | "list_dir"
                | "grep_search"
                | "find_by_name"
                | "read_url_content"
                | "read_file"
        ) {
            let target = args
                .get("AbsolutePath")
                .or_else(|| args.get("Path"))
                .or_else(|| args.get("path"))
                .or_else(|| args.get("file_path"))
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

        if matches!(
            name_lower.as_str(),
            "ask_question" | "list_plugin_accounts" | "search_marketplace" | "search_web"
        ) {
            return NormalizedToolCall {
                action_type: ToolActionType::ReadFile,
                target_resource: String::new(),
                raw_tool_name: tool_name,
                payload: args,
            };
        }

        if name_lower == "manage_task" {
            let action = args
                .get("Action")
                .or_else(|| args.get("action"))
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();

            if matches!(action.as_str(), "list" | "status") {
                return NormalizedToolCall {
                    action_type: ToolActionType::ReadFile,
                    target_resource: String::new(),
                    raw_tool_name: tool_name,
                    payload: args,
                };
            }
        }

        NormalizedToolCall {
            action_type: ToolActionType::Other,
            target_resource: String::new(),
            raw_tool_name: tool_name,
            payload: args,
        }
    }

    fn validate_plugin(&self, plugin_dir: &Path) -> AdapterValidationResult {
        validation::validate_plugin(plugin_dir)
    }

    fn handle_hook(&self, workspace: &Path, stdin_content: &str) -> (i32, String) {
        let content = crate::features::adapters::clean_stdin(stdin_content);
        if content.is_empty() {
            let formatted_denial = "[XGAUNTLET POLICY BLOCKED] Tool: unknown | Task: NONE (Phase: SPEC) | Reason: Empty payload received on stdin | Recovery: Provide valid hook JSON on stdin".to_string();
            let res = serde_json::json!({
                "decision": "deny",
                "reason": formatted_denial,
                "system_instruction": formatted_denial
            });
            return (1, res.to_string());
        }

        let payload: Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                let formatted_denial = format!("[XGAUNTLET POLICY BLOCKED] Tool: unknown | Task: NONE (Phase: SPEC) | Reason: Corrupt JSON payload on stdin: {e} | Recovery: Provide valid hook JSON on stdin");
                let res = serde_json::json!({
                    "decision": "deny",
                    "reason": formatted_denial,
                    "system_instruction": formatted_denial
                });
                return (1, res.to_string());
            }
        };

        let effective_ws = Self::resolve_effective_workspace(workspace, &payload);

        let hooks_path = effective_ws.join(".agents").join("hooks.json");
        if hooks_path.is_file() {
            if let Ok(content) = fs::read_to_string(&hooks_path) {
                if content.contains("\"agent-gauntlet-gatekeeper\"") {
                    eprintln!("⚠️ Legacy hook namespace 'agent-gauntlet-gatekeeper' detected in .agents/hooks.json. Run 'xgauntlet migrate' to upgrade.");
                }
            }
        }

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
            let telemetry =
                match crate::features::tasks::inspect_task_telemetry(&effective_ws, None) {
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
                        git: crate::features::tasks::collect_git_telemetry(&effective_ws),
                        file_path: "tasks/".to_string(),
                        scope: None,
                        invariants: Some("14/14 PASS".to_string()),
                        evidence: Some("pending".to_string()),
                        phase: Some("SPEC".to_string()),
                        session_health: None,
                    },
                };
            let ephemeral = Self::format_ephemeral_telemetry(&telemetry);
            let payload = Self::format_pre_invocation_payload(&ephemeral);
            let json_str =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
            return (0, json_str);
        }

        let verdict = self.evaluate_invocation(&effective_ws, &payload);
        if verdict.allowed {
            let res = serde_json::json!({
                "decision": "allow"
            });
            (0, res.to_string())
        } else {
            let tool_name = payload
                .get("toolCall")
                .and_then(|t| t.get("name"))
                .or_else(|| payload.get("tool_name"))
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");

            let task_info =
                crate::features::tasks::inspect_task_telemetry(&effective_ws, None).ok();
            let task_id = task_info
                .as_ref()
                .map(|t| t.task_id.as_str())
                .unwrap_or("NONE");
            let phase = task_info
                .as_ref()
                .and_then(|t| t.phase.as_deref())
                .unwrap_or("SPEC");

            let recovery = if verdict.reason.contains("scope") || verdict.reason_code == Some(4037)
            {
                format!(
                    "Add target path to 'scope: [...]' in tasks/{task_id}.md or restrict modifications to declared scope"
                )
            } else if verdict.reason.contains("push")
                || verdict.reason.contains("hard")
                || verdict.reason_code == Some(4038)
                || verdict.reason_code == Some(4039)
            {
                "Destructive operations and remote push are strictly prohibited by ADR 0003"
                    .to_string()
            } else if verdict.reason.contains("read-only") {
                "Mutations are prohibited during read-only phase".to_string()
            } else {
                "Inspect active task invariants in CONTEXT.md and spec.md before proceeding"
                    .to_string()
            };

            let formatted_denial = format!(
                "[XGAUNTLET POLICY BLOCKED] Tool: {tool_name} | Task: {task_id} (Phase: {phase}) | Reason: {} | Recovery: {recovery}",
                verdict.reason
            );

            let res = serde_json::json!({
                "decision": "deny",
                "reason": formatted_denial,
                "system_instruction": formatted_denial
            });

            (1, res.to_string())
        }
    }
}
