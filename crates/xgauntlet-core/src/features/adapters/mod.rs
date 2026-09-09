//! Autonomous vertical harness adapter slices.
//!
//! Implements ADR 0004 (Harness Adapter Slices) and ADR 0006
//! (Multi-Harness Policy Adapter Contract and Trusted Context).

pub mod antigravity;
pub mod claude_code;
pub mod codex;
pub mod models;

pub use antigravity::AntigravityAdapter;
pub use claude_code::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
pub use models::{
    AdapterHookVerdict, AdapterValidationResult, HarnessAdapter, NormalizedToolCall,
    ValidationIssue, ValidationSeverity,
};

/// Canonical list of all supported agent harness environments.
pub const SUPPORTED_HARNESSES: &[&str] = &["antigravity", "claude_code", "codex"];

/// Strongly-typed harness kind enumeration supporting all canonical aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HarnessKind {
    Antigravity,
    ClaudeCode,
    Codex,
}

impl HarnessKind {
    /// Resolves canonical harness kind from user-supplied alias string.
    pub fn parse_alias(alias: &str) -> Option<Self> {
        let normalized = alias.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "antigravity" | "google_antigravity" | "google-antigravity" => Some(Self::Antigravity),
            "claude_code" | "claude" | "claude-code" => Some(Self::ClaudeCode),
            "codex" | "openai" | "openai_codex" | "openai-codex" => Some(Self::Codex),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Antigravity => "antigravity",
            Self::ClaudeCode => "claude_code",
            Self::Codex => "codex",
        }
    }
}

/// Instantiates an autonomous harness adapter slice by name.
pub fn get_adapter(harness: &str) -> Option<Box<dyn HarnessAdapter>> {
    match HarnessKind::parse_alias(harness)? {
        HarnessKind::Antigravity => Some(Box::new(AntigravityAdapter::new())),
        HarnessKind::ClaudeCode => Some(Box::new(ClaudeCodeAdapter::new())),
        HarnessKind::Codex => Some(Box::new(CodexAdapter::new())),
    }
}

/// Formats the canonical PostToolUse JSON payload on stdout (Claude Code and OpenAI Codex).
pub fn format_post_tool_use_payload(box_card: &str) -> serde_json::Value {
    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": box_card
        }
    })
}

/// Merges a PostToolUse hook into a settings or hooks JSON object.
pub fn merge_post_tool_use_hook(
    existing_json: Option<&serde_json::Value>,
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

    let mut post_tool_vec = match hooks.get("PostToolUse").and_then(|p| p.as_array()) {
        Some(arr) => arr.clone(),
        None => Vec::new(),
    };

    let already_exists = post_tool_vec.iter().any(|item| {
        item.get("hooks")
            .and_then(|h| h.as_array())
            .map(|arr| {
                arr.iter()
                    .any(|h| h.get("command").and_then(|c| c.as_str()) == Some(hook_command))
            })
            .unwrap_or(false)
    });

    if !already_exists {
        post_tool_vec.push(hook_entry);
    }

    hooks.insert(
        "PostToolUse".to_string(),
        serde_json::Value::Array(post_tool_vec),
    );
    root.insert("hooks".to_string(), serde_json::Value::Object(hooks));
    serde_json::Value::Object(root)
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
