//! Pure delegation policy evaluation for subagents and swarms (ADR 0014).

use crate::models::{CapabilityRequest, DecisionVerdict, EnforcementContext, PolicyDecision};
use serde_json::Value;

fn has_shared_inherit_swarm_add(raw_tool: &str, payload: &Value) -> bool {
    let tool_lower = raw_tool.to_ascii_lowercase();
    if tool_lower != "swarm" {
        return false;
    }
    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if action != "add" {
        return false;
    }
    let ws = payload
        .get("workspace")
        .or_else(|| payload.get("Workspace"))
        .and_then(|v| v.as_str())
        .unwrap_or("inherit")
        .to_ascii_lowercase();
    ws != "share" && ws != "branch" && ws != "worktree"
}

fn is_read_only_lifecycle(payload: &Value) -> bool {
    let action = payload
        .get("action")
        .or_else(|| payload.get("Action"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(action.as_str(), "status" | "models" | "list")
}

/// Evaluates an agent delegation capability request inside the pure policy engine.
pub fn evaluate_delegation(req: &CapabilityRequest, ctx: &EnforcementContext) -> PolicyDecision {
    let parsed_payload = serde_json::from_str::<Value>(&req.payload_json).ok();
    if req.target_resource != "coordinated_inherit" {
        if let Some(ref payload) = parsed_payload {
            if has_shared_inherit_swarm_add(&req.raw_tool_name, payload) {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Concurrent mutating worker in shared 'inherit' workspace is prohibited (ADR 0014). Specify workspace: \"share\" or \"branch\".".into(),
                    reason_code: 4040,
                };
            }
        }
    }

    match req.target_resource.as_str() {
        "read_only" => PolicyDecision {
            verdict: DecisionVerdict::Allow,
            reason: "Read-only subagent or query delegation allowed.".into(),
            reason_code: 2006,
        },
        "shared_inherit" => PolicyDecision {
            verdict: DecisionVerdict::Deny,
            reason: "Concurrent mutating agent delegation in shared 'inherit' workspace is prohibited (ADR 0014). Use an isolated worktree (`share`, `branch`, or `worktree`).".into(),
            reason_code: 4040,
        },
        "isolated" | "coordinated_inherit" => {
            if ctx.read_only {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Workspace is in read-only mode; mutating agent delegation is prohibited.".into(),
                    reason_code: 4030,
                };
            }
            if !ctx.has_active_task {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Mutating agent delegation requires an active task (ADR 0014).".into(),
                    reason_code: 4034,
                };
            }
            PolicyDecision {
                verdict: DecisionVerdict::Allow,
                reason: if req.target_resource == "coordinated_inherit" {
                    "Coordinated inherit-mode multi-agent delegation allowed under active task."
                        .into()
                } else {
                    "Isolated worktree mutating agent delegation allowed under active task.".into()
                },
                reason_code: 2007,
            }
        }
        "lifecycle" => {
            if parsed_payload.as_ref().is_some_and(is_read_only_lifecycle) {
                return PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: "Read-only delegation lifecycle inspection allowed.".into(),
                    reason_code: 2006,
                };
            }
            if ctx.read_only {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Workspace is in read-only mode; mutating delegation lifecycle operation is prohibited.".into(),
                    reason_code: 4030,
                };
            }
            if !ctx.has_active_task {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Mutating delegation lifecycle operation requires an active task (ADR 0014).".into(),
                    reason_code: 4034,
                };
            }
            PolicyDecision {
                verdict: DecisionVerdict::Allow,
                reason: "Delegation lifecycle operation allowed under active task.".into(),
                reason_code: 2007,
            }
        }
        _ => PolicyDecision {
            verdict: DecisionVerdict::Deny,
            reason: "Unrecognized delegation classification (fail-closed).".into(),
            reason_code: 4038,
        },
    }
}
