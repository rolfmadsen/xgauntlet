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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EnforcementContext {
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub has_active_task: bool,
    #[serde(default)]
    pub active_task_id: String,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub allowed_scopes: Vec<String>,
    #[serde(default)]
    pub allowed_test_targets: Vec<String>,
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

fn matches_pattern(path: &str, pattern: &str) -> bool {
    let p = pattern.trim().replace('\\', "/");
    let path = path.trim().replace('\\', "/");
    if p == "*" || p == "**" || p == path {
        return true;
    }
    if let Some(prefix) = p.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    if let Some(prefix) = p.strip_suffix("/*") {
        if path.starts_with(&format!("{prefix}/")) {
            let rest = &path[prefix.len() + 1..];
            return !rest.contains('/');
        }
        return false;
    }
    if let Some(suffix) = p.strip_prefix('*') {
        return path.ends_with(suffix);
    }
    path == p
}

fn matches_any(path: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pat| matches_pattern(path, pat))
}

fn is_test_file(path: &str) -> bool {
    let p = path.replace('\\', "/");
    p.starts_with("tests/")
        || p.contains("/tests/")
        || p.contains("__tests__")
        || p.ends_with("_test.rs")
        || p.ends_with(".test.ts")
        || p.ends_with(".spec.ts")
        || p.ends_with(".test.js")
        || p.ends_with(".spec.js")
        || p.ends_with("_test.py")
        || p.ends_with("_test.go")
}

#[derive(Debug, PartialEq, Eq)]
enum TokenizerError {
    UnclosedQuote,
}

fn tokenize_command_line(cmd: &str) -> Result<Vec<Vec<String>>, TokenizerError> {
    let mut segments: Vec<Vec<String>> = Vec::new();
    let mut current_segment: Vec<String> = Vec::new();
    let mut current_token = String::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    let chars: Vec<char> = cmd.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if escaped {
            current_token.push(ch);
            escaped = false;
            i += 1;
            continue;
        }

        if ch == '\\' && !in_single_quote {
            escaped = true;
            i += 1;
            continue;
        }

        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }

        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            i += 1;
            continue;
        }

        if in_single_quote || in_double_quote {
            current_token.push(ch);
            i += 1;
            continue;
        }

        // Outside quotes: check for operators: &&, ||, ;, |
        if ch == '&' && i + 1 < len && chars[i + 1] == '&' {
            if !current_token.is_empty() {
                current_segment.push(std::mem::take(&mut current_token));
            }
            if !current_segment.is_empty() {
                segments.push(std::mem::take(&mut current_segment));
            }
            i += 2;
            continue;
        }

        if ch == '|' && i + 1 < len && chars[i + 1] == '|' {
            if !current_token.is_empty() {
                current_segment.push(std::mem::take(&mut current_token));
            }
            if !current_segment.is_empty() {
                segments.push(std::mem::take(&mut current_segment));
            }
            i += 2;
            continue;
        }

        if ch == ';' || ch == '|' {
            if !current_token.is_empty() {
                current_segment.push(std::mem::take(&mut current_token));
            }
            if !current_segment.is_empty() {
                segments.push(std::mem::take(&mut current_segment));
            }
            i += 1;
            continue;
        }

        if ch.is_whitespace() {
            if !current_token.is_empty() {
                current_segment.push(std::mem::take(&mut current_token));
            }
            i += 1;
            continue;
        }

        current_token.push(ch);
        i += 1;
    }

    if in_single_quote || in_double_quote {
        return Err(TokenizerError::UnclosedQuote);
    }

    if !current_token.is_empty() {
        current_segment.push(current_token);
    }
    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    Ok(segments)
}

fn normalize_executable(exe: &str) -> String {
    let name = exe.rsplit(['/', '\\']).next().unwrap_or(exe);
    let lower = name.to_ascii_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

fn resolve_git_subcommand<'a>(args: &'a [String]) -> Option<(&'a str, &'a [String])> {
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-C"
            || arg == "-c"
            || arg == "--config-env"
            || arg == "--exec-path"
            || arg == "--git-dir"
            || arg == "--work-tree"
            || arg == "--namespace"
            || arg == "--super-prefix"
        {
            i += 2;
        } else if arg.starts_with("--git-dir=")
            || arg.starts_with("--work-tree=")
            || arg.starts_with("--namespace=")
            || arg.starts_with("--exec-path=")
            || arg.starts_with("--config-env=")
            || (arg.starts_with("-c") && arg.len() > 2)
            || (arg.starts_with("-C") && arg.len() > 2)
        {
            i += 1;
        } else if arg.starts_with('-') {
            i += 1;
        } else {
            return Some((arg.as_str(), &args[i + 1..]));
        }
    }
    None
}

fn is_forbidden_wrapper(norm_exe: &str) -> bool {
    matches!(
        norm_exe,
        "env"
            | "sudo"
            | "doas"
            | "su"
            | "eval"
            | "xargs"
            | "exec"
            | "command"
            | "builtin"
            | "nohup"
            | "timeout"
            | "nice"
            | "stdbuf"
            | "strace"
            | "ltrace"
    )
}

fn is_shell_executable(norm_exe: &str) -> bool {
    matches!(
        norm_exe,
        "bash" | "sh" | "zsh" | "dash" | "ksh" | "fish" | "csh" | "tcsh"
    )
}

fn is_safe_command_segment(tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let norm_exe = normalize_executable(&tokens[0]);
    match norm_exe.as_str() {
        "git" => {
            if let Some((sub, _)) = resolve_git_subcommand(&tokens[1..]) {
                matches!(sub, "status" | "diff" | "log")
            } else {
                false
            }
        }
        "ls" | "pwd" | "echo" | "which" | "pytest" => true,
        "cargo" => {
            if tokens.len() > 1 {
                matches!(tokens[1].as_str(), "check" | "test" | "clippy" | "fmt")
            } else {
                false
            }
        }
        "npm" => {
            if tokens.len() > 1 {
                tokens[1] == "test"
                    || (tokens[1] == "run" && tokens.len() > 2 && tokens[2] == "test")
            } else {
                false
            }
        }
        "xgauntlet" => {
            if tokens.len() > 1 {
                matches!(
                    tokens[1].as_str(),
                    "verify" | "doctor" | "check-evidence" | "check-spec"
                )
            } else {
                false
            }
        }
        "npx" => {
            if tokens.len() > 2 && tokens[1] == "xgauntlet" {
                matches!(
                    tokens[2].as_str(),
                    "verify" | "doctor" | "check-evidence" | "check-spec"
                )
            } else {
                false
            }
        }
        _ => false,
    }
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

            let is_code = target_str.starts_with("src/")
                || target_str.starts_with("tests/")
                || target_str.starts_with("crates/")
                || target_str.starts_with("packages/")
                || target_str.starts_with(".agents/");

            if !ctx.has_active_task {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: if is_code {
                        "Writing to production code without active task is prohibited.".into()
                    } else {
                        "Mutating repository files without an active task is prohibited.".into()
                    },
                    reason_code: if is_code { 4031 } else { 4032 },
                };
            }

            // AC3: Fail-closed task scope enforcement
            // 1. If both allowed_scopes and allowed_test_targets are empty -> Deny (4037)
            if ctx.allowed_scopes.is_empty() && ctx.allowed_test_targets.is_empty() {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: format!(
                        "Writing to '{}' prohibited: active task '{}' has no declared scopes or test targets (fail-closed).",
                        target_str, ctx.active_task_id
                    ),
                    reason_code: 4037,
                };
            }

            // 2. If allowed_scopes is empty, only specific test targets can be written; source code is strictly Denied (4037)
            if ctx.allowed_scopes.is_empty() {
                if is_test_file(target_str) && matches_any(target_str, &ctx.allowed_test_targets) {
                    return PolicyDecision {
                        verdict: DecisionVerdict::Allow,
                        reason: format!(
                            "Writing to test target permitted under active task '{}'.",
                            ctx.active_task_id
                        ),
                        reason_code: 2002,
                    };
                }
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: format!(
                        "Writing to '{}' prohibited: active task '{}' has empty allowed_scopes and target is not an allowed test file.",
                        target_str, ctx.active_task_id
                    ),
                    reason_code: 4037,
                };
            }

            // 3. When allowed_scopes is defined, check allowed_scopes, then test_targets for test files
            if matches_any(target_str, &ctx.allowed_scopes) {
                return PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: if is_code {
                        format!(
                            "Writing to code permitted under active task '{}'.",
                            ctx.active_task_id
                        )
                    } else {
                        "Write operation permitted under active task.".into()
                    },
                    reason_code: if is_code { 2002 } else { 2003 },
                };
            }

            if is_test_file(target_str) && matches_any(target_str, &ctx.allowed_test_targets) {
                return PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: format!(
                        "Writing to test target permitted under active task '{}'.",
                        ctx.active_task_id
                    ),
                    reason_code: 2002,
                };
            }

            PolicyDecision {
                verdict: DecisionVerdict::Deny,
                reason: format!(
                    "Writing to '{}' prohibited: target is outside active task '{}' declared scope or test targets.",
                    target_str, ctx.active_task_id
                ),
                reason_code: 4037,
            }
        }
        ToolActionType::ExecuteCommand => {
            // Reject newline injection in commands (fail-closed against CRLF / newline injection)
            if req.target_resource.contains('\n') || req.target_resource.contains('\r') {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Newline injection in command execution is strictly prohibited.".into(),
                    reason_code: 4039,
                };
            }

            let cmd = req.target_resource.trim();
            if cmd.is_empty() {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "Empty command execution is prohibited.".into(),
                    reason_code: 4039,
                };
            }

            // Structured tokenization into command segments
            let segments = match tokenize_command_line(cmd) {
                Ok(segs) => segs,
                Err(TokenizerError::UnclosedQuote) => {
                    return PolicyDecision {
                        verdict: DecisionVerdict::Deny,
                        reason: "Unclosed quotes in command execution (fail-closed).".into(),
                        reason_code: 4039,
                    };
                }
            };

            if segments.is_empty() {
                return PolicyDecision {
                    verdict: DecisionVerdict::Deny,
                    reason: "No valid command tokens found (fail-closed).".into(),
                    reason_code: 4039,
                };
            }

            let has_chaining = segments.len() > 1
                || cmd.contains("&&")
                || cmd.contains(';')
                || cmd.contains("||")
                || cmd.contains('|');

            // Evaluate every command segment in the chain/pipeline
            for seg in &segments {
                if seg.is_empty() {
                    continue;
                }

                let norm_exe = normalize_executable(&seg[0]);

                // 1. Prohibit wrappers (env, sudo, eval, xargs, etc.)
                if is_forbidden_wrapper(&norm_exe) {
                    return PolicyDecision {
                        verdict: DecisionVerdict::Deny,
                        reason: format!(
                            "Execution wrapper '{}' is strictly prohibited (fail-closed).",
                            norm_exe
                        ),
                        reason_code: 4039,
                    };
                }

                // 2. Prohibit shell execution wrappers and shell invocations
                if is_shell_executable(&norm_exe) {
                    return PolicyDecision {
                        verdict: DecisionVerdict::Deny,
                        reason: format!(
                            "Shell execution wrapper '{}' is strictly prohibited (fail-closed).",
                            norm_exe
                        ),
                        reason_code: 4039,
                    };
                }

                // 3. Prohibit destructive git commands with robust subcommand resolution
                if norm_exe == "git" {
                    if let Some((subcmd, sub_args)) = resolve_git_subcommand(&seg[1..]) {
                        if subcmd == "push" {
                            return PolicyDecision {
                                verdict: DecisionVerdict::Deny,
                                reason: "Remote publication actions ('git push') are strictly prohibited.".into(),
                                reason_code: 4039,
                            };
                        }
                        if subcmd == "reset"
                            && sub_args.iter().any(|a| a == "--hard" || a == "--merge")
                        {
                            return PolicyDecision {
                                verdict: DecisionVerdict::Deny,
                                reason: "Destructive git reset operations ('--hard') are strictly prohibited.".into(),
                                reason_code: 4039,
                            };
                        }
                        if subcmd == "clean"
                            && sub_args.iter().any(|a| {
                                a == "-f"
                                    || a == "-fd"
                                    || a == "-fx"
                                    || a == "-df"
                                    || a == "-xf"
                                    || a == "--force"
                            })
                        {
                            return PolicyDecision {
                                verdict: DecisionVerdict::Deny,
                                reason: "Destructive git clean operations ('-f' / '--force') are strictly prohibited.".into(),
                                reason_code: 4039,
                            };
                        }
                        if subcmd == "branch"
                            && sub_args.iter().any(|a| a == "-D" || a == "--force")
                        {
                            return PolicyDecision {
                                verdict: DecisionVerdict::Deny,
                                reason: "Destructive git branch deletions ('-D') are strictly prohibited.".into(),
                                reason_code: 4039,
                            };
                        }
                    }
                }

                // 4. Prohibit destructive rm commands
                if norm_exe == "rm" {
                    let has_rf = seg.iter().any(|a| {
                        a == "-rf" || a == "-fr" || a == "-r" || a == "-f" || a == "--force"
                    });
                    let targets_root_or_home = seg
                        .iter()
                        .any(|a| a == "/" || a == "/*" || a == "~" || a == "~/*" || a == "$HOME");
                    if has_rf && targets_root_or_home {
                        return PolicyDecision {
                            verdict: DecisionVerdict::Deny,
                            reason: "Destructive rm operations on root or home directory are strictly prohibited.".into(),
                            reason_code: 4039,
                        };
                    }
                }
            }

            // Explicitly block destructive literal patterns as defense-in-depth
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

            // Command chaining with unsafe primitives
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

            // Safe verification and inspection commands (only when not chained)
            if !has_chaining && segments.len() == 1 && is_safe_command_segment(&segments[0]) {
                return PolicyDecision {
                    verdict: DecisionVerdict::Allow,
                    reason: "Read-only or verification command is permitted.".into(),
                    reason_code: 2004,
                };
            }

            // Backward compatibility with safe_prefixes strings
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
    serde_json::from_str(json).unwrap_or_default()
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
            ..Default::default()
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
            ..Default::default()
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
            allowed_scopes: vec!["crates/**".into()],
            ..Default::default()
        };
        let dec = evaluate(&req, &ctx);
        assert_eq!(dec.verdict, DecisionVerdict::Allow);
        assert_eq!(dec.reason_code, 2002);
    }

    #[test]
    fn test_empty_task_scope_fails_closed() {
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
            allowed_scopes: vec![],
            allowed_test_targets: vec![],
        };
        let dec = evaluate(&req, &ctx);
        assert_eq!(dec.verdict, DecisionVerdict::Deny);
        assert_eq!(dec.reason_code, 4037);
    }

    #[test]
    fn test_test_scope_isolation_blocks_source_code() {
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
            allowed_scopes: vec![],
            allowed_test_targets: vec!["crates/**".into()],
        };
        let dec = evaluate(&req, &ctx);
        assert_eq!(dec.verdict, DecisionVerdict::Deny);
        assert_eq!(dec.reason_code, 4037);

        let test_req = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write_to_file".into(),
            target_resource: "crates/xgauntlet-core/tests/policy_engine_test.rs".into(),
            payload_json: "{}".into(),
        };
        let test_dec = evaluate(&test_req, &ctx);
        assert_eq!(test_dec.verdict, DecisionVerdict::Allow);
        assert_eq!(test_dec.reason_code, 2002);
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
            ..Default::default()
        };
        let dec = evaluate(&req, &ctx);
        assert_eq!(dec.verdict, DecisionVerdict::Deny);
        assert_eq!(dec.reason_code, 4039);
    }

    #[test]
    fn test_destructive_command_evasions_denied() {
        let evasions = [
            "git   push",
            "git --no-pager push origin main",
            "git -C repo push",
            "git -c foo=bar push",
            "env git push",
            "sudo git push",
            "eval \"git push\"",
            "bash -c \"git push\"",
            "sh -c \"git push\"",
            "xargs git push",
        ];
        let ctx = EnforcementContext {
            workspace_id: "ws-1".into(),
            has_active_task: true,
            active_task_id: "001".into(),
            read_only: false,
            ..Default::default()
        };
        for cmd in evasions {
            let req = CapabilityRequest {
                action_type: ToolActionType::ExecuteCommand,
                raw_tool_name: "run_command".into(),
                target_resource: cmd.into(),
                payload_json: "{}".into(),
            };
            let dec = evaluate(&req, &ctx);
            assert_eq!(
                dec.verdict,
                DecisionVerdict::Deny,
                "Command '{}' must be denied",
                cmd
            );
            assert_eq!(dec.reason_code, 4039);
        }
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
