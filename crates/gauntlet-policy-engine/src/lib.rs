use serde::{Deserialize, Serialize};

pub const POLICY_VERSION: &str = env!("CARGO_PKG_VERSION");

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub action_type: ToolActionType,
    #[serde(default)]
    pub raw_tool_name: String,
    #[serde(default)]
    pub target_resource: String,
    #[serde(default)]
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnforcementContext {
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub has_active_task: bool,
    #[serde(default)]
    pub active_task_id: String,
    #[serde(default)]
    pub read_only: bool,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub verdict: DecisionVerdict,
    pub reason: String,
    pub reason_code: u32,
}

/// Deterministically evaluates a capability request against trusted context.
pub fn evaluate(req: &CapabilityRequest, ctx: &EnforcementContext) -> PolicyDecision {
    if ctx.read_only
        && matches!(
            req.action_type,
            ToolActionType::WriteFile | ToolActionType::ExecuteCommand
        )
    {
        return PolicyDecision {
            verdict: DecisionVerdict::Deny,
            reason: "Workspace is in read-only mode; state mutation is prohibited.".into(),
            reason_code: 4030,
        };
    }

    match req.action_type {
        ToolActionType::ReadFile => PolicyDecision {
            verdict: DecisionVerdict::Allow,
            reason: "Read operations are unrestricted.".into(),
            reason_code: 2000,
        },
        ToolActionType::WriteFile => {
            let target = req.target_resource.replace('\\', "/");
            let target_str = target.trim();

            // Fail-closed path traversal and escape defense
            if target_str.contains("..")
                || target_str.starts_with('/')
                || target_str.starts_with("//")
                || (target_str.len() >= 2
                    && target_str.as_bytes()[0].is_ascii_alphabetic()
                    && target_str.as_bytes()[1] == b':')
            {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Path traversal or illegal absolute path detected (fail-closed)."
                        .into(),
                    reason_code: 4036,
                };
            }

            // Safe documentation & specification paths can be written/updated even during planning
            if target_str.starts_with("tasks/")
                || target_str == "spec.md"
                || target_str == "CONTEXT.md"
                || target_str == "CODING_STANDARDS.md"
                || target_str == "README.md"
                || target_str.starts_with("docs/")
            {
                return PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: "Writing task definitions or domain documentation is permitted.".into(),
                    reason_code: 2001,
                };
            }

            // Protected source code paths require an active task
            if target_str.starts_with("src/")
                || target_str.starts_with("tests/")
                || target_str.starts_with("crates/")
                || target_str.starts_with("packages/")
                || target_str.starts_with(".agents/")
            {
                if !ctx.has_active_task {
                    return PolicyDecision {
                        verdict: DecisionVerdict::Deny,
                        reason: "Writing to production code without active task is prohibited."
                            .into(),
                        reason_code: 4031,
                    };
                }
                return PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: format!(
                        "Writing to code permitted under active task '{}'.",
                        ctx.active_task_id
                    ),
                    reason_code: 2002,
                };
            }

            if !ctx.has_active_task {
                PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Mutating repository files without an active task is prohibited."
                        .into(),
                    reason_code: 4032,
                }
            } else {
                PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: "Write operation permitted under active task.".into(),
                    reason_code: 2003,
                }
            }
        }
        ToolActionType::ExecuteCommand => {
            let cmd = req.target_resource.trim();

            // Explicitly block dangerous destructive commands
            let dangerous_patterns = [
                "git push",
                "git reset --hard",
                "git clean -f",
                "git branch -D",
                "rm -rf /",
                "rm -rf ~",
                "| bash",
                "| sh",
                "| zsh",
            ];
            for pattern in &dangerous_patterns {
                if cmd.contains(pattern) {
                    return PolicyDecision {
                        verdict: DecisionVerdict::Deny,
                        reason: format!(
                            "Destructive command pattern '{}' is strictly prohibited.",
                            pattern
                        ),
                        reason_code: 4039,
                    };
                }
            }

            // Reject newline injection in commands
            if cmd.contains('\n') || cmd.contains('\r') {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Newline injection in command execution is strictly prohibited.".into(),
                    reason_code: 4039,
                };
            }

            // Command chaining with &&, ;, ||, | is prohibited from inheriting safe prefix allowances
            let has_chaining =
                cmd.contains("&&") || cmd.contains(';') || cmd.contains("||") || cmd.contains('|');
            if has_chaining
                && (cmd.contains("curl ")
                    || cmd.contains("wget ")
                    || cmd.contains("bash")
                    || cmd.contains("sh")
                    || cmd.contains("rm "))
            {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason:
                        "Unsafe chained shell execution or remote script execution is prohibited."
                            .into(),
                    reason_code: 4039,
                };
            }

            let safe_prefixes = [
                "git status",
                "git diff",
                "git log",
                "ls",
                "pwd",
                "echo",
                "which",
                "pytest",
                "cargo check",
                "cargo test",
                "cargo clippy",
                "cargo fmt",
                "npm test",
                "npm run test",
                "npx xgauntlet verify",
                "xgauntlet verify",
                "xgauntlet doctor",
                "xgauntlet check-evidence",
                "xgauntlet check-spec",
            ];
            if !has_chaining && safe_prefixes.iter().any(|prefix| cmd.starts_with(prefix)) {
                return PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: "Read-only or verification command is permitted.".into(),
                    reason_code: 2004,
                };
            }

            if !ctx.has_active_task {
                PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Executing modifying commands without an active task is prohibited."
                        .into(),
                    reason_code: 4033,
                }
            } else {
                PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: "Command execution permitted under active task.".into(),
                    reason_code: 2005,
                }
            }
        }
        ToolActionType::Other => PolicyDecision {
            verdict: DecisionVerdict::Deny,
            reason: "Unrecognized or unspecified capability request (fail-closed).".into(),
            reason_code: 4038,
        },
    }
}

pub fn parse_capability_request(json: &str) -> CapabilityRequest {
    serde_json::from_str(json).unwrap_or(CapabilityRequest {
        action_type: ToolActionType::Other,
        raw_tool_name: String::new(),
        target_resource: String::new(),
        payload_json: String::new(),
    })
}

pub fn parse_enforcement_context(json: &str) -> EnforcementContext {
    serde_json::from_str(json).unwrap_or(EnforcementContext {
        workspace_id: String::new(),
        has_active_task: false,
        active_task_id: String::new(),
        read_only: false,
    })
}

impl PolicyDecision {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"verdict":"{}","reason":"{}","reason_code":{}}}"#,
                self.verdict.as_str(),
                self.reason,
                self.reason_code
            )
        })
    }
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Deallocates memory previously allocated by `alloc`.
///
/// # Safety
/// Caller must ensure `ptr` was allocated by `alloc` with the exact same `size`.
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    if !ptr.is_null() && size > 0 {
        let _ = Vec::from_raw_parts(ptr, 0, size);
    }
}

/// Evaluates a capability request against enforcement context as raw C byte buffers.
///
/// # Safety
/// Caller must ensure `req_ptr` and `ctx_ptr` point to valid byte buffers of length `req_len`
/// and `ctx_len` respectively, and that `out_len` is a valid mutable pointer to receive the output length.
#[no_mangle]
pub unsafe extern "C" fn evaluate_json(
    req_ptr: *const u8,
    req_len: usize,
    ctx_ptr: *const u8,
    ctx_len: usize,
    out_len: *mut usize,
) -> *mut u8 {
    let req_bytes = std::slice::from_raw_parts(req_ptr, req_len);
    let ctx_bytes = std::slice::from_raw_parts(ctx_ptr, ctx_len);

    let req_res: Result<CapabilityRequest, _> = serde_json::from_slice(req_bytes);
    let ctx_res: Result<EnforcementContext, _> = serde_json::from_slice(ctx_bytes);

    let decision = match (req_res, ctx_res) {
        (Ok(req), Ok(ctx)) => evaluate(&req, &ctx),
        _ => PolicyDecision {
            verdict: DecisionVerdict::Deny,
            reason: "Malformed or invalid JSON input to policy verifier (fail-closed).".into(),
            reason_code: 4037,
        },
    };

    let mut json_out = serde_json::to_vec(&decision).unwrap_or_else(|_| {
        b"{\"verdict\":\"deny\",\"reason\":\"Serialization error\",\"reason_code\":4037}".to_vec()
    });
    json_out.shrink_to_fit();

    if !out_len.is_null() {
        *out_len = json_out.len();
    }
    let ptr = json_out.as_mut_ptr();
    std::mem::forget(json_out);
    ptr
}

/// WebAssembly 32-bit linear memory bridge returning packed `(len << 32) | ptr`.
///
/// # Safety
/// Caller must ensure `req_ptr` and `ctx_ptr` point to valid WebAssembly linear memory addresses
/// with lengths `req_len` and `ctx_len`.
#[no_mangle]
pub unsafe extern "C" fn evaluate_json_wasm(
    req_ptr: u32,
    req_len: u32,
    ctx_ptr: u32,
    ctx_len: u32,
) -> u64 {
    let mut out_len: usize = 0;
    let ptr = evaluate_json(
        req_ptr as *const u8,
        req_len as usize,
        ctx_ptr as *const u8,
        ctx_len as usize,
        &mut out_len,
    );
    ((out_len as u64) << 32) | ((ptr as usize as u64) & 0xFFFFFFFF)
}

#[no_mangle]
pub extern "C" fn get_policy_version_ptr() -> *const u8 {
    POLICY_VERSION.as_ptr()
}

#[no_mangle]
pub extern "C" fn get_policy_version_len() -> usize {
    POLICY_VERSION.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_is_unrestricted() {
        let req = CapabilityRequest {
            action_type: ToolActionType::ReadFile,
            raw_tool_name: "view_file".into(),
            target_resource: "crates/xgauntlet-core/src/lib.rs".into(),
            payload_json: "{}".into(),
        };
        let ctx = EnforcementContext {
            workspace_id: "ws-1".into(),
            has_active_task: false,
            active_task_id: "".into(),
            read_only: false,
        };
        let dec = evaluate(&req, &ctx);
        assert_eq!(dec.verdict, DecisionVerdict::Allow);
    }

    #[test]
    fn test_write_without_task_denied() {
        let req = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write_to_file".into(),
            target_resource: "crates/xgauntlet-core/src/lib.rs".into(),
            payload_json: "{}".into(),
        };
        let ctx = EnforcementContext {
            workspace_id: "ws-1".into(),
            has_active_task: false,
            active_task_id: "".into(),
            read_only: false,
        };
        let dec = evaluate(&req, &ctx);
        assert_eq!(dec.verdict, DecisionVerdict::Deny);
        assert_eq!(dec.reason_code, 4031);
    }

    #[test]
    fn test_write_with_task_allowed() {
        let req = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write_to_file".into(),
            target_resource: "crates/xgauntlet-core/src/lib.rs".into(),
            payload_json: "{}".into(),
        };
        let ctx = EnforcementContext {
            workspace_id: "ws-1".into(),
            has_active_task: true,
            active_task_id: "001-bootstrap".into(),
            read_only: false,
        };
        let dec = evaluate(&req, &ctx);
        assert_eq!(dec.verdict, DecisionVerdict::Allow);
    }

    #[test]
    fn test_git_push_strictly_denied() {
        let req = CapabilityRequest {
            action_type: ToolActionType::ExecuteCommand,
            raw_tool_name: "run_command".into(),
            target_resource: "git push origin main".into(),
            payload_json: "{}".into(),
        };
        let ctx = EnforcementContext {
            workspace_id: "ws-1".into(),
            has_active_task: true,
            active_task_id: "001-bootstrap".into(),
            read_only: false,
        };
        let dec = evaluate(&req, &ctx);
        assert_eq!(dec.verdict, DecisionVerdict::Deny);
        assert_eq!(dec.reason_code, 4039);
    }

    #[test]
    fn test_json_roundtrip_evaluation() {
        let req_json = r#"{"action_type":"write_file","raw_tool_name":"write_to_file","target_resource":"crates/xgauntlet-core/src/lib.rs","payload_json":"{}"}"#;
        let ctx_json = r#"{"workspace_id":"ws-1","has_active_task":false,"active_task_id":"","read_only":false}"#;

        unsafe {
            let mut out_len: usize = 0;
            let ptr = evaluate_json(
                req_json.as_ptr(),
                req_json.len(),
                ctx_json.as_ptr(),
                ctx_json.len(),
                &mut out_len,
            );
            assert!(!ptr.is_null());
            assert!(out_len > 0);
            let res_bytes = std::slice::from_raw_parts(ptr, out_len);
            let res_str = std::str::from_utf8(res_bytes).unwrap();
            assert!(res_str.contains(r#""verdict":"deny""#));
            assert!(res_str.contains(r#""reason_code":4031"#));
            dealloc(ptr, out_len);
        }
    }
}
