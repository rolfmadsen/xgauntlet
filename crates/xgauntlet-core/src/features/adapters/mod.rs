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

/// Instantiates an autonomous harness adapter slice by name.
pub fn get_adapter(harness: &str) -> Option<Box<dyn HarnessAdapter>> {
    match harness {
        "antigravity" => Some(Box::new(AntigravityAdapter::new())),
        "claude_code" | "claude" => Some(Box::new(ClaudeCodeAdapter::new())),
        "codex" | "openai" => Some(Box::new(CodexAdapter::new())),
        _ => None,
    }
}
