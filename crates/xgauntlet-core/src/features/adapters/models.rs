//! Common domain models and traits for harness adapters.
//!
//! Defined in accordance with ADR 0004 (Vertical Slice Harness Adapters)
//! and ADR 0006 (Multi-Harness Policy Adapter Contract and Trusted Context).

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::features::policy::{
    evaluate_reference, CapabilityRequest, DecisionVerdict, EnforcementContext, ToolActionType,
};

/// Canonical representation of an agent action across different harnesses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedToolCall {
    pub action_type: ToolActionType,
    pub target_resource: String,
    pub raw_tool_name: String,
    pub payload: serde_json::Value,
}

/// Severity of a plugin validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

impl ValidationSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
        }
    }
}

/// A specific issue encountered during mechanical plugin and manifest validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub path: String,
    pub message: String,
}

/// Consolidated result of mechanical plugin, skills, and manifest validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

impl AdapterValidationResult {
    pub fn success() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
        }
    }

    pub fn failure(issues: Vec<ValidationIssue>) -> Self {
        Self {
            valid: false,
            issues,
        }
    }
}

/// Verdict returned by adapter invocation evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterHookVerdict {
    pub allowed: bool,
    pub decision: String,
    pub reason: String,
    pub reason_code: Option<u32>,
}

/// Autonomous vertical slice interface required for all harness adapters.
pub trait HarnessAdapter: Send + Sync {
    /// Human-readable harness identifier (e.g. "antigravity", "claude_code", "codex").
    fn name(&self) -> &'static str;

    /// Translates platform-specific raw payload into canonical NormalizedToolCall.
    fn normalize_tool_call(&self, payload: &serde_json::Value) -> NormalizedToolCall;

    /// Maps normalized action into strongly-typed CapabilityRequest (ADR 0006).
    fn to_capability_request(&self, payload: &serde_json::Value) -> CapabilityRequest {
        let normalized = self.normalize_tool_call(payload);
        let payload_json =
            serde_json::to_string(&normalized.payload).unwrap_or_else(|_| "{}".to_string());

        CapabilityRequest {
            action_type: normalized.action_type,
            raw_tool_name: normalized.raw_tool_name,
            target_resource: normalized.target_resource,
            payload_json,
        }
    }

    /// Evaluates tool invocation against trusted enforcement context (ADR 0006, ADR 0007).
    ///
    /// Ensures callers cannot override context parameters such as `has_active_task`
    /// or `read_only` from tool payload parameters, sanitizes paths against workspace
    /// containment, and executes the embedded WebAssembly policy engine.
    fn evaluate_invocation(
        &self,
        workspace: &Path,
        payload: &serde_json::Value,
    ) -> AdapterHookVerdict {
        let mut req = self.to_capability_request(payload);

        // Path authorization hardening: sanitize paths against workspace containment
        if matches!(
            req.action_type,
            ToolActionType::WriteFile | ToolActionType::ReadFile
        ) {
            match crate::features::policy::WorkspaceRelativePath::sanitize(
                workspace,
                &req.target_resource,
            ) {
                Ok(clean) => {
                    req.target_resource = clean.into_inner();
                }
                Err(err) => {
                    return AdapterHookVerdict {
                        allowed: false,
                        decision: "deny".to_string(),
                        reason: format!(
                            "Path traversal or workspace escape attempt detected: {err}"
                        ),
                        reason_code: Some(4036),
                    };
                }
            }
        }

        let ctx = EnforcementContext::from_workspace(workspace, false);

        // Execute deterministic WebAssembly policy engine in-memory (ADR 0007)
        let decision = match crate::features::policy::WasmPolicyEngine::new() {
            Ok(mut engine) => {
                match crate::features::policy::PolicyEvaluator::evaluate(&mut engine, &req, &ctx) {
                    Ok(d) => d,
                    Err(e) => {
                        return AdapterHookVerdict {
                            allowed: false,
                            decision: "deny".to_string(),
                            reason: format!("Wasm policy evaluation failed (fail-closed): {e}"),
                            reason_code: Some(5000),
                        };
                    }
                }
            }
            Err(_) => {
                // Fallback to reference evaluator if Wasm instantiation fails
                evaluate_reference(&req, &ctx)
            }
        };

        AdapterHookVerdict {
            allowed: decision.verdict == DecisionVerdict::Allow,
            decision: decision.verdict.as_str().to_string(),
            reason: decision.reason,
            reason_code: Some(decision.reason_code),
        }
    }

    /// Mechanically validates plugin directory, manifest, skills, and hooks.
    fn validate_plugin(&self, plugin_dir: &Path) -> AdapterValidationResult;

    /// Evaluates raw stdin hook input according to platform protocol and returns
    /// `(exit_code, stdout_or_stderr_output)` adhering to fail-closed contract.
    fn handle_hook(&self, workspace: &Path, stdin_content: &str) -> (i32, String);
}
