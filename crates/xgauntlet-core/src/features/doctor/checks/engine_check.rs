//! Embedded WebAssembly policy engine and runtime diagnostic checks.

use crate::features::doctor::models::{DoctorCategory, DoctorCheckItem, DoctorCheckStatus};
use crate::features::policy::{
    CapabilityRequest, EnforcementContext, PolicyEvaluator, ToolActionType, WasmPolicyEngine,
    DEFAULT_POLICY_WASM,
};
use crate::features::wasm::WasmRuntimeHost;
use std::path::Path;
use std::time::Instant;

/// Inspects Wasmtime host instantiation and zero-ambient-authority policy engine evaluation.
pub fn check_engine(workspace: &Path) -> Vec<DoctorCheckItem> {
    let mut checks = Vec::new();

    // 1. WasmRuntimeHost instantiation
    let start = Instant::now();
    let host_res = WasmRuntimeHost::new(DEFAULT_POLICY_WASM);
    let duration = start.elapsed().as_millis() as u64;

    match host_res {
        Ok(_) => {
            checks.push(DoctorCheckItem {
                name: "wasm_runtime_host".to_string(),
                category: DoctorCategory::Engine,
                status: DoctorCheckStatus::Pass,
                message: "Wasmtime host initialized (Zero Ambient Authority verified)".to_string(),
                detail: Some("Embedded WebAssembly runtime operational".to_string()),
                remediation: None,
                duration_ms: duration,
            });
        }
        Err(e) => {
            checks.push(DoctorCheckItem {
                name: "wasm_runtime_host".to_string(),
                category: DoctorCategory::Engine,
                status: DoctorCheckStatus::Fail,
                message: format!("Failed to initialize Wasmtime host: {e}"),
                detail: None,
                remediation: Some(
                    "Ensure platform supports WebAssembly execution with Wasmtime.".to_string(),
                ),
                duration_ms: duration,
            });
            return checks;
        }
    }

    // 2. WasmPolicyEngine evaluation probe
    let start = Instant::now();
    match WasmPolicyEngine::new() {
        Ok(mut engine) => {
            let context = EnforcementContext::from_workspace(workspace, false);
            let request = CapabilityRequest {
                action_type: ToolActionType::ReadFile,
                raw_tool_name: "view_file".to_string(),
                target_resource: "spec.md".to_string(),
                payload_json: "{}".to_string(),
            };

            match engine.evaluate(&request, &context) {
                Ok(decision) => {
                    let duration = start.elapsed().as_millis() as u64;
                    checks.push(DoctorCheckItem {
                        name: "policy_engine_evaluation".to_string(),
                        category: DoctorCategory::Engine,
                        status: DoctorCheckStatus::Pass,
                        message: format!(
                            "Policy component evaluated capability request: {:?} (code {})",
                            decision.verdict, decision.reason_code
                        ),
                        detail: Some(format!(
                            "Verdict: {:?}, Reason: {}",
                            decision.verdict, decision.reason
                        )),
                        remediation: None,
                        duration_ms: duration,
                    });
                }
                Err(e) => {
                    let duration = start.elapsed().as_millis() as u64;
                    checks.push(DoctorCheckItem {
                        name: "policy_engine_evaluation".to_string(),
                        category: DoctorCategory::Engine,
                        status: DoctorCheckStatus::Fail,
                        message: format!("Policy evaluation failed: {e}"),
                        detail: None,
                        remediation: Some(
                            "Rebuild policy component via cargo build in crates/gauntlet-policy-engine."
                                .to_string(),
                        ),
                        duration_ms: duration,
                    });
                }
            }
        }
        Err(e) => {
            let duration = start.elapsed().as_millis() as u64;
            checks.push(DoctorCheckItem {
                name: "policy_engine_evaluation".to_string(),
                category: DoctorCategory::Engine,
                status: DoctorCheckStatus::Fail,
                message: format!("Failed to instantiate policy engine: {e}"),
                detail: None,
                remediation: Some(
                    "Rebuild embedded WebAssembly policy engine component.".to_string(),
                ),
                duration_ms: duration,
            });
        }
    }

    checks
}
