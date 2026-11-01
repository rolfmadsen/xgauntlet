//! Deterministic capability evaluation and policy enforcement.
//!
//! Evaluates strongly-typed capability requests against trusted enforcement contexts
//! via the embedded WebAssembly policy engine (`wit/gauntlet_policy.wit`).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::features::wasm::{WasmError, WasmRuntimeHost};

pub mod path;
pub use path::{PathSecurityError, WorkspaceRelativePath};

/// Returns true if a raw path string contains null bytes or any `..` traversal segment
/// across both POSIX (`/`) and Windows (`\`) separators.
pub fn has_path_traversal(raw: &str) -> bool {
    if raw.contains('\0') {
        return true;
    }
    raw.split(['/', '\\']).any(|seg| seg == "..")
}

fn normalize_lexical_path(raw: &str) -> String {
    let mut s = raw.trim().replace('\\', "/");
    if let Some(rest) = s.strip_prefix("//?/") {
        let bytes = rest.as_bytes();
        if bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && bytes[2] == b'/'
        {
            s = rest.to_string();
        }
    }

    let is_unc = s.starts_with("//");
    let mut collapsed = String::with_capacity(s.len());
    let mut prev_slash = false;
    for (idx, ch) in s.chars().enumerate() {
        if ch == '/' {
            if prev_slash && !(is_unc && idx == 1) {
                continue;
            }
            prev_slash = true;
        } else {
            prev_slash = false;
        }
        collapsed.push(ch);
    }

    let bytes = collapsed.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        let mut chars = collapsed.chars();
        if let Some(first) = chars.next() {
            let mut upper = String::with_capacity(collapsed.len());
            upper.push(first.to_ascii_uppercase());
            upper.push_str(chars.as_str());
            return upper;
        }
    }

    collapsed
}

fn normalize_valid_home_anchor(raw_home: &str) -> Option<String> {
    if has_path_traversal(raw_home) {
        return None;
    }
    let norm = normalize_lexical_path(raw_home);
    let trimmed = norm.trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let bytes = trimmed.as_bytes();
    // Reject bare Windows drive root such as "C:" (from "C:\" or "C:/")
    if bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return None;
    }
    let is_posix_abs = trimmed.starts_with('/') && !trimmed.starts_with("//");
    let is_win_drive_abs =
        bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/';
    if !is_posix_abs && !is_win_drive_abs {
        return None;
    }
    Some(trimmed.to_string())
}

fn strip_home_prefix<'a>(norm_path: &'a str, home_anchor: &str) -> Option<&'a str> {
    let rest = norm_path.strip_prefix(home_anchor)?;
    rest.strip_prefix('/')
}

fn collect_env_homes() -> Vec<String> {
    let mut homes = Vec::with_capacity(2);
    if let Ok(h) = std::env::var("HOME") {
        homes.push(h);
    }
    if let Ok(up) = std::env::var("USERPROFILE") {
        if !homes.contains(&up) {
            homes.push(up);
        }
    }
    homes
}

/// Checks if a file path is within known agent harness configuration or skill directories
/// against an explicit list of candidate user home directories.
pub fn is_harness_whitelisted_path_with_homes<H: AsRef<str>>(
    path: &std::path::Path,
    homes: &[H],
) -> bool {
    let raw = path.to_string_lossy();
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
        || has_path_traversal(&raw)
    {
        return false;
    }

    // Relative workspace agent directories
    let norm_path = normalize_lexical_path(&raw);
    let rel_norm = norm_path.trim_start_matches("./");
    if rel_norm.starts_with(".agents/")
        || rel_norm.starts_with(".claude/")
        || rel_norm.starts_with(".codex/")
    {
        return true;
    }

    // Absolute paths MUST be anchored to a valid user home directory ($HOME or %USERPROFILE%)
    for home in homes {
        if let Some(anchor) = normalize_valid_home_anchor(home.as_ref()) {
            if let Some(s) = strip_home_prefix(&norm_path, &anchor) {
                if s.starts_with(".gemini/config/plugins/")
                    || s.starts_with(".gemini/antigravity/builtin/")
                    || s.starts_with(".agents/")
                    || s.starts_with(".claude/")
                    || s.starts_with(".codex/")
                {
                    return true;
                }
            }
        }
    }

    false
}

/// Checks if a file path is within known agent harness configuration or skill directories.
pub fn is_harness_whitelisted_path(path: &std::path::Path) -> bool {
    let homes = collect_env_homes();
    is_harness_whitelisted_path_with_homes(path, &homes)
}

/// Checks if a file path is within Antigravity session brain artifacts/scratch directory
/// against an explicit list of candidate user home directories.
pub fn is_session_brain_artifact_path_with_homes<H: AsRef<str>>(
    path: &std::path::Path,
    homes: &[H],
) -> bool {
    let raw = path.to_string_lossy();
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
        || has_path_traversal(&raw)
    {
        return false;
    }

    let norm_path = normalize_lexical_path(&raw);
    for home in homes {
        if let Some(anchor) = normalize_valid_home_anchor(home.as_ref()) {
            if let Some(s) = strip_home_prefix(&norm_path, &anchor) {
                if s.starts_with(".gemini/antigravity/brain/")
                    || s.starts_with(".gemini/antigravity-ide/brain/")
                {
                    return true;
                }
            }
        }
    }

    false
}

/// Checks if a file path is within Antigravity session brain artifacts/scratch directory.
pub fn is_session_brain_artifact_path(path: &std::path::Path) -> bool {
    let homes = collect_env_homes();
    is_session_brain_artifact_path_with_homes(path, &homes)
}

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EnforcementContext {
    pub workspace_id: String,
    pub has_active_task: bool,
    pub active_task_id: String,
    pub read_only: bool,
    #[serde(default)]
    pub allowed_scopes: Vec<String>,
    #[serde(default)]
    pub allowed_test_targets: Vec<String>,
}

impl EnforcementContext {
    /// Creates an enforcement context directly resolved from a workspace directory.
    pub fn from_workspace(workspace: &std::path::Path, read_only: bool) -> Self {
        let active_task_id = crate::features::tasks::resolve_active_task_id(workspace);
        let mut has_active_task = false;
        let mut active_task_id_str = String::new();
        let mut allowed_scopes = Vec::new();
        let mut allowed_test_targets = Vec::new();

        if let Some(ref tid) = active_task_id {
            let task_file = workspace.join("tasks").join(format!("{tid}.md"));
            if task_file.is_file() {
                match crate::features::tasks::parse_task_file(&task_file) {
                    Ok(info) => {
                        has_active_task = true;
                        active_task_id_str = tid.clone();
                        allowed_scopes = info.scope;
                        allowed_test_targets = info.test_refs;
                    }
                    Err(_) => {
                        // Fail-closed: corrupt or invalid task files leave scopes empty and has_active_task = false
                        has_active_task = false;
                        active_task_id_str.clear();
                        allowed_scopes.clear();
                        allowed_test_targets.clear();
                    }
                }
            }
        }

        let workspace_id = workspace
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("workspace")
            .to_string();

        Self {
            workspace_id,
            has_active_task,
            active_task_id: active_task_id_str,
            read_only,
            allowed_scopes,
            allowed_test_targets,
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
        allowed_scopes: ctx.allowed_scopes.clone(),
        allowed_test_targets: ctx.allowed_test_targets.clone(),
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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
