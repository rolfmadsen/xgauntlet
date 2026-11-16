//! Autonomous vertical harness adapter slices.
//!
//! Implements ADR 0004 (Harness Adapter Slices) and ADR 0006
//! (Multi-Harness Policy Adapter Contract and Trusted Context).

pub mod antigravity;
pub mod claude_code;
pub mod codex;
pub mod delegation;
pub mod mistral;
pub mod models;
pub mod workspace;

pub use antigravity::AntigravityAdapter;
pub use claude_code::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
pub use delegation::classify_delegation_tool;
pub use mistral::MistralAdapter;
pub use models::{
    AdapterHookVerdict, AdapterValidationResult, HarnessAdapter, NormalizedToolCall,
    ValidationIssue, ValidationSeverity,
};
pub use workspace::{is_same_git_repo_worktree, is_valid_workspace, resolve_effective_workspace};

/// Canonical list of all supported agent harness environments.
pub const SUPPORTED_HARNESSES: &[&str] = &["antigravity", "claude_code", "codex", "mistral"];

/// Strongly-typed harness kind enumeration supporting all canonical aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HarnessKind {
    Antigravity,
    ClaudeCode,
    Codex,
    Mistral,
}

impl HarnessKind {
    /// Resolves canonical harness kind from user-supplied alias string.
    pub fn parse_alias(alias: &str) -> Option<Self> {
        let normalized = alias.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "antigravity" | "google_antigravity" | "google-antigravity" => Some(Self::Antigravity),
            "claude_code" | "claude" | "claude-code" => Some(Self::ClaudeCode),
            "codex" | "openai" | "openai_codex" | "openai-codex" => Some(Self::Codex),
            "mistral" | "mistral_vibe" | "mistral-vibe" | "vibe" => Some(Self::Mistral),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Antigravity => "antigravity",
            Self::ClaudeCode => "claude_code",
            Self::Codex => "codex",
            Self::Mistral => "mistral",
        }
    }
}

/// Instantiates an autonomous harness adapter slice by name.
pub fn get_adapter(harness: &str) -> Option<Box<dyn HarnessAdapter>> {
    match HarnessKind::parse_alias(harness)? {
        HarnessKind::Antigravity => Some(Box::new(AntigravityAdapter::new())),
        HarnessKind::ClaudeCode => Some(Box::new(ClaudeCodeAdapter::new())),
        HarnessKind::Codex => Some(Box::new(CodexAdapter::new())),
        HarnessKind::Mistral => Some(Box::new(MistralAdapter::new())),
    }
}

/// Formats the canonical PostToolUse JSON payload on stdout (Claude Code, OpenAI Codex, Mistral).
pub fn format_post_tool_use_payload(box_card: &str) -> serde_json::Value {
    format_post_tool_use_payload_with_directive(box_card, None)
}

/// Formats PostToolUse JSON payload with optional JIT phase directive appended to additionalContext.
pub fn format_post_tool_use_payload_with_directive(
    box_card: &str,
    directive: Option<&str>,
) -> serde_json::Value {
    let context = match directive {
        Some(d) if !d.is_empty() => format!("{box_card}\n\n{d}"),
        _ => box_card.to_string(),
    };
    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": context
        }
    })
}

/// Merges an arbitrary lifecycle hook into a settings or hooks JSON object under "hooks.<event_name>".
pub fn merge_lifecycle_hook(
    existing_json: Option<&serde_json::Value>,
    event_name: &str,
    matcher: &str,
    hook_command: &str,
) -> serde_json::Value {
    let hook_entry = serde_json::json!({
        "matcher": matcher,
        "hooks": [
            {
                "type": "command",
                "command": hook_command
            }
        ]
    });

    let mut root = match existing_json.and_then(|e| e.as_object()) {
        Some(obj) => obj.clone(),
        None => serde_json::Map::new(),
    };

    let mut hooks = match root.get("hooks").and_then(|h| h.as_object()) {
        Some(h) => h.clone(),
        None => serde_json::Map::new(),
    };

    let mut event_vec = match hooks.get(event_name).and_then(|p| p.as_array()) {
        Some(arr) => arr.clone(),
        None => Vec::new(),
    };

    let already_exists = event_vec.iter().any(|item| {
        item.get("hooks")
            .and_then(|h| h.as_array())
            .map(|arr| {
                arr.iter()
                    .any(|h| h.get("command").and_then(|c| c.as_str()) == Some(hook_command))
            })
            .unwrap_or(false)
    });

    if !already_exists {
        event_vec.push(hook_entry);
    }

    hooks.insert(event_name.to_string(), serde_json::Value::Array(event_vec));
    root.insert("hooks".to_string(), serde_json::Value::Object(hooks));
    serde_json::Value::Object(root)
}

/// Merges a PostToolUse hook into a settings or hooks JSON object.
pub fn merge_post_tool_use_hook(
    existing_json: Option<&serde_json::Value>,
    matcher: &str,
    hook_command: &str,
) -> serde_json::Value {
    merge_lifecycle_hook(existing_json, "PostToolUse", matcher, hook_command)
}

/// Wraps response output with telemetry card or HUD blockquote.
pub fn wrap_response_with_hud(hud_or_card: &str, body: &str) -> String {
    let trimmed_body = body.trim();
    if trimmed_body.is_empty() {
        hud_or_card.to_string()
    } else {
        format!("{}\n\n{}", hud_or_card.trim_end(), trimmed_body)
    }
}

/// Strips UTF-8 byte-order-marks (BOM) and leading/trailing whitespace from incoming hook payloads.
pub fn clean_stdin(raw: &str) -> &str {
    raw.trim().trim_start_matches('\u{feff}').trim()
}

/// Formats standard actionable denial message for stderr across all adapters (AC3, Must NOT #4).
pub fn format_policy_denial(
    tool_name: &str,
    task_id: &str,
    phase: &str,
    reason: &str,
    recovery: &str,
) -> String {
    format!(
        "[XGAUNTLET POLICY BLOCKED] Tool: {tool_name} | Task: {task_id} (Phase: {phase}) | Reason: {reason} | Recovery: {recovery}"
    )
}

/// Derives an actionable, policy-code-aware recovery instruction shared across all harness adapters.
pub fn format_recovery_hint(task_id: &str, verdict: &AdapterHookVerdict) -> String {
    if verdict.reason_code == Some(4040) {
        "Specify workspace: \"share\" or \"branch\" (or isolation: \"worktree\") to run mutating workers in isolated git worktrees".to_string()
    } else if verdict.reason_code == Some(4034) {
        "Create or bind an ACTIVE task before delegating mutating subagent or swarm work"
            .to_string()
    } else if verdict.reason.contains("scope") || verdict.reason_code == Some(4037) {
        format!(
            "Add target path to 'scope: [...]' in tasks/{task_id}.md or restrict modifications to declared scope"
        )
    } else if verdict.reason.contains("push")
        || verdict.reason.contains("hard")
        || verdict.reason_code == Some(4038)
        || verdict.reason_code == Some(4039)
    {
        "Destructive operations and remote push are strictly prohibited by ADR 0003".to_string()
    } else if verdict.reason.contains("read-only") {
        "Mutations are prohibited during read-only phase".to_string()
    } else if matches!(verdict.reason_code, Some(4031..=4033)) {
        "Create or activate a task in tasks/ (or run 'xgauntlet task bind --task <id>') before modifying files".to_string()
    } else {
        "Adhere to active task scope and invariants in CONTEXT.md and spec.md".to_string()
    }
}

/// Derives an actionable recovery hint enriched with stale `.xgauntlet/active_task` diagnostics when applicable.
pub fn format_recovery_hint_for_workspace(
    workspace: &std::path::Path,
    task_id: &str,
    verdict: &AdapterHookVerdict,
) -> String {
    if task_id == "NONE" || matches!(verdict.reason_code, Some(4031..=4034)) {
        if let Some(stale) = crate::features::tasks::parser::inspect_stale_task_binding(workspace) {
            return format!("⚠️ {}", stale.warning_message());
        }
    }
    format_recovery_hint(task_id, verdict)
}
