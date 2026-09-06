//! Deterministic capability evaluation and policy enforcement.
//!
//! Evaluates strongly-typed capability requests against trusted enforcement contexts
//! via the embedded WebAssembly policy engine (`wit/gauntlet_policy.wit`).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::features::wasm::{WasmError, WasmRuntimeHost};

pub const DEFAULT_POLICY_WASM: &[u8] = include_bytes!("../../../wasm/gauntlet_policy.wasm");

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("WebAssembly host error: {0}")]
    Wasm(#[from] WasmError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Policy evaluation internal failure: {0}")]
    EvaluationFailed(String),
}

/// Tool action classification for fine-grained capability evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolActionType {
    ReadFile,
    WriteFile,
    ExecuteCommand,
    Other,
}

impl ToolActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::ExecuteCommand => "execute_command",
            Self::Other => "other",
        }
    }
}

/// Strongly-typed capability request from an agent or harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub action_type: ToolActionType,
    pub raw_tool_name: String,
    pub target_resource: String,
    pub payload_json: String,
}

/// Immutable trusted context provided by the supervisor runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnforcementContext {
    pub workspace_id: String,
    pub has_active_task: bool,
    pub active_task_id: String,
    pub read_only: bool,
}

impl EnforcementContext {
    /// Creates an enforcement context directly resolved from a workspace directory.
    pub fn from_workspace(workspace: &std::path::Path, read_only: bool) -> Self {
        let active_task_id = crate::features::tasks::resolve_active_task_id(workspace);
        let has_active_task = active_task_id.is_some();
        let workspace_id = workspace
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("workspace")
            .to_string();

        Self {
            workspace_id,
            has_active_task,
            active_task_id: active_task_id.unwrap_or_default(),
            read_only,
        }
    }
}

/// Policy evaluation decision verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionVerdict {
    Allow,
    Deny,
    Ask,
    ForceAsk,
}

impl DecisionVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Ask => "ask",
            Self::ForceAsk => "force_ask",
        }
    }
}

/// Strongly-typed policy decision returned by the policy engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub verdict: DecisionVerdict,
    pub reason: String,
    pub reason_code: u32,
}

/// Common trait for evaluating capability requests.
pub trait PolicyEvaluator {
    fn evaluate(
        &mut self,
        req: &CapabilityRequest,
        ctx: &EnforcementContext,
    ) -> Result<PolicyDecision, PolicyError>;
}

/// Embedded WebAssembly policy engine running inside Wasmtime with Zero Ambient Authority.
pub struct WasmPolicyEngine {
    host: WasmRuntimeHost,
}

impl WasmPolicyEngine {
    /// Creates a new policy engine using the embedded default WebAssembly bytecode.
    pub fn new() -> Result<Self, PolicyError> {
        Self::from_wasm_bytes(DEFAULT_POLICY_WASM)
    }

    /// Creates a new policy engine from custom WebAssembly bytecode.
    pub fn from_wasm_bytes(wasm_bytes: &[u8]) -> Result<Self, PolicyError> {
        let host = WasmRuntimeHost::new(wasm_bytes)?;
        Ok(Self { host })
    }

    /// Returns the policy version reported by the WebAssembly module.
    pub fn get_policy_version(&mut self) -> Result<String, PolicyError> {
        Ok(self.host.get_policy_version()?)
    }
}

impl PolicyEvaluator for WasmPolicyEngine {
    fn evaluate(
        &mut self,
        req: &CapabilityRequest,
        ctx: &EnforcementContext,
    ) -> Result<PolicyDecision, PolicyError> {
        let req_json = serde_json::to_string(req)?;
        let ctx_json = serde_json::to_string(ctx)?;

        // Fail-closed enforcement: any evaluation error results in a secure denial
        let out_json = match self.host.evaluate_json(&req_json, &ctx_json) {
            Ok(json) => json,
            Err(e) => {
                return Ok(PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: format!("Wasm evaluation failed closed: {e}"),
                    reason_code: 5000,
                });
            }
        };

        let decision: PolicyDecision = match serde_json::from_str(&out_json) {
            Ok(d) => d,
            Err(e) => {
                return Ok(PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: format!("Corrupted policy decision output (fail-closed): {e}"),
                    reason_code: 5001,
                });
            }
        };

        Ok(decision)
    }
}

/// Pure reference evaluator for bit-for-bit parity testing and in-process fallback.
pub fn evaluate_reference(req: &CapabilityRequest, ctx: &EnforcementContext) -> PolicyDecision {
    let internal_req = gauntlet_policy_engine::CapabilityRequest {
        action_type: match req.action_type {
            ToolActionType::ReadFile => gauntlet_policy_engine::ToolActionType::ReadFile,
            ToolActionType::WriteFile => gauntlet_policy_engine::ToolActionType::WriteFile,
            ToolActionType::ExecuteCommand => {
                gauntlet_policy_engine::ToolActionType::ExecuteCommand
            }
            ToolActionType::Other => gauntlet_policy_engine::ToolActionType::Other,
        },
        raw_tool_name: req.raw_tool_name.clone(),
        target_resource: req.target_resource.clone(),
        payload_json: req.payload_json.clone(),
    };

    let internal_ctx = gauntlet_policy_engine::EnforcementContext {
        workspace_id: ctx.workspace_id.clone(),
        has_active_task: ctx.has_active_task,
        active_task_id: ctx.active_task_id.clone(),
        read_only: ctx.read_only,
    };

    let dec = gauntlet_policy_engine::evaluate(&internal_req, &internal_ctx);

    PolicyDecision {
        verdict: match dec.verdict {
            gauntlet_policy_engine::DecisionVerdict::Allow => DecisionVerdict::Allow,
            gauntlet_policy_engine::DecisionVerdict::Deny => DecisionVerdict::Deny,
            gauntlet_policy_engine::DecisionVerdict::Ask => DecisionVerdict::Ask,
            gauntlet_policy_engine::DecisionVerdict::ForceAsk => DecisionVerdict::ForceAsk,
        },
        reason: dec.reason,
        reason_code: dec.reason_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_version_matches_crate_version() {
        let mut engine = WasmPolicyEngine::new().unwrap();
        let ver = engine.get_policy_version().unwrap();
        assert_eq!(ver, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_reference_parity() {
        let mut engine = WasmPolicyEngine::new().unwrap();

        let scenarios = vec![
            (
                CapabilityRequest {
                    action_type: ToolActionType::ReadFile,
                    raw_tool_name: "read_file".into(),
                    target_resource: "any/file.rs".into(),
                    payload_json: "{}".into(),
                },
                EnforcementContext {
                    workspace_id: "w1".into(),
                    has_active_task: false,
                    active_task_id: "".into(),
                    read_only: false,
                },
            ),
            (
                CapabilityRequest {
                    action_type: ToolActionType::WriteFile,
                    raw_tool_name: "write_file".into(),
                    target_resource: "src/lib.rs".into(),
                    payload_json: "{}".into(),
                },
                EnforcementContext {
                    workspace_id: "w1".into(),
                    has_active_task: false,
                    active_task_id: "".into(),
                    read_only: false,
                },
            ),
            (
                CapabilityRequest {
                    action_type: ToolActionType::WriteFile,
                    raw_tool_name: "write_file".into(),
                    target_resource: "src/lib.rs".into(),
                    payload_json: "{}".into(),
                },
                EnforcementContext {
                    workspace_id: "w1".into(),
                    has_active_task: true,
                    active_task_id: "T-1".into(),
                    read_only: false,
                },
            ),
            (
                CapabilityRequest {
                    action_type: ToolActionType::ExecuteCommand,
                    raw_tool_name: "bash".into(),
                    target_resource: "git push origin main".into(),
                    payload_json: "{}".into(),
                },
                EnforcementContext {
                    workspace_id: "w1".into(),
                    has_active_task: true,
                    active_task_id: "T-1".into(),
                    read_only: false,
                },
            ),
        ];

        for (req, ctx) in scenarios {
            let wasm_decision = engine.evaluate(&req, &ctx).unwrap();
            let ref_decision = evaluate_reference(&req, &ctx);

            assert_eq!(
                wasm_decision.verdict, ref_decision.verdict,
                "Verdict mismatch for req {:?}",
                req
            );
            assert_eq!(
                wasm_decision.reason_code, ref_decision.reason_code,
                "Reason code mismatch for req {:?}",
                req
            );
            assert_eq!(
                wasm_decision.reason, ref_decision.reason,
                "Reason mismatch for req {:?}",
                req
            );
        }
    }

    #[test]
    fn test_corrupted_json_fails_closed() {
        let mut host = WasmRuntimeHost::new(DEFAULT_POLICY_WASM).unwrap();
        let res = host.evaluate_json("not valid json", "{ corrupt");
        // Must either error or evaluate to Deny, never panic
        if let Ok(json_str) = res {
            let parsed: Result<PolicyDecision, _> = serde_json::from_str(&json_str);
            if let Ok(d) = parsed {
                assert_ne!(
                    d.verdict,
                    DecisionVerdict::Allow,
                    "Malformed input must NEVER be allowed"
                );
            }
        }
    }
}
