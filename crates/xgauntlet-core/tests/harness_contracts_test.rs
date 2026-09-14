use std::fs;
use std::path::PathBuf;
use std::process::Command;

use xgauntlet_core::features::adapters::get_adapter;
use xgauntlet_core::features::policy::{
    CapabilityRequest, DecisionVerdict, EnforcementContext, PolicyEvaluator, ToolActionType,
    WasmPolicyEngine,
};

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let unique = format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            count,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn find_xgauntlet_binary() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../../target/debug/xgauntlet"),
        manifest_dir.join("../../target/release/xgauntlet"),
        manifest_dir.join("../../target/debug/xgauntlet.exe"),
        manifest_dir.join("../../target/release/xgauntlet.exe"),
    ];
    for c in candidates {
        if c.is_file() {
            return c;
        }
    }
    PathBuf::from("xgauntlet")
}

// ============================================================================
// Klynge 2: Claude Code Gatekeeper Opdatering & Exit Code 2 (ADR 0006)
// ============================================================================

#[test]
fn test_claude_code_gatekeeper_denial_contract_exit_code_2() {
    let adapter = get_adapter("claude_code").expect("claude_code adapter");
    let temp = TempDir::new("claude_denial_contract");
    let ws = &temp.path;

    // 1. Dangerous git push command
    let push_payload = serde_json::json!({
        "name": "Bash",
        "input": { "command": "git push origin main" }
    })
    .to_string();

    let (exit_code, output) = adapter.handle_hook(ws, &push_payload);
    assert_eq!(
        exit_code, 2,
        "ClaudeCodeAdapter MUST return exit code 2 on tool denial"
    );

    let parsed: serde_json::Value =
        serde_json::from_str(&output).expect("Output must be valid JSON");
    let hook_output = parsed
        .get("hookSpecificOutput")
        .expect("Must contain hookSpecificOutput");
    assert_eq!(
        hook_output.get("hookEventName").and_then(|v| v.as_str()),
        Some("PreToolUse")
    );
    assert_eq!(
        hook_output.get("permissionDecision").and_then(|v| v.as_str()),
        Some("deny")
    );
    let reason = hook_output
        .get("permissionDecisionReason")
        .and_then(|v| v.as_str())
        .expect("Must have permissionDecisionReason");
    assert!(
        reason.contains("git push") || reason.contains("prohibited"),
        "Reason must explain the denial: {}",
        reason
    );

    // 2. Destructive rm -rf command
    let rm_payload = serde_json::json!({
        "name": "Bash",
        "input": { "command": "rm -rf /" }
    })
    .to_string();
    let (code_rm, out_rm) = adapter.handle_hook(ws, &rm_payload);
    assert_eq!(code_rm, 2);
    let parsed_rm: serde_json::Value = serde_json::from_str(&out_rm).unwrap();
    assert_eq!(
        parsed_rm["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );

    // 3. Modifying production code without active task
    let write_payload = serde_json::json!({
        "name": "FileEdit",
        "input": { "file_path": "src/main.rs" }
    })
    .to_string();
    let (code_write, out_write) = adapter.handle_hook(ws, &write_payload);
    assert_eq!(code_write, 2);
    let parsed_write: serde_json::Value = serde_json::from_str(&out_write).unwrap();
    assert_eq!(
        parsed_write["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );
    assert!(
        parsed_write["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap()
            .contains("active task"),
        "Reason must state active task requirement"
    );

    // 4. Path traversal attempt
    let escape_payload = serde_json::json!({
        "name": "FileEdit",
        "input": { "file_path": "../../etc/passwd" }
    })
    .to_string();
    let (code_escape, out_escape) = adapter.handle_hook(ws, &escape_payload);
    assert_eq!(code_escape, 2);
    let parsed_escape: serde_json::Value = serde_json::from_str(&out_escape).unwrap();
    assert_eq!(
        parsed_escape["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );
}

#[test]
fn test_claude_code_gatekeeper_allow_contract_exit_code_0() {
    let adapter = get_adapter("claude_code").expect("claude_code adapter");
    let temp = TempDir::new("claude_allow_contract");
    let ws = &temp.path;

    // 1. Reading documentation file
    let read_payload = serde_json::json!({
        "name": "FileRead",
        "input": { "file_path": "README.md" }
    })
    .to_string();

    let (exit_code, output) = adapter.handle_hook(ws, &read_payload);
    assert_eq!(
        exit_code, 0,
        "ClaudeCodeAdapter MUST return exit code 0 on tool approval"
    );

    let parsed: serde_json::Value =
        serde_json::from_str(&output).expect("Output must be valid JSON");
    let hook_output = parsed
        .get("hookSpecificOutput")
        .expect("Must contain hookSpecificOutput");
    assert_eq!(
        hook_output.get("hookEventName").and_then(|v| v.as_str()),
        Some("PreToolUse")
    );
    assert_eq!(
        hook_output.get("permissionDecision").and_then(|v| v.as_str()),
        Some("allow")
    );

    // 2. Safe inspection command
    let check_payload = serde_json::json!({
        "name": "Bash",
        "input": { "command": "cargo check" }
    })
    .to_string();
    let (code_cmd, out_cmd) = adapter.handle_hook(ws, &check_payload);
    assert_eq!(code_cmd, 0);
    let parsed_cmd: serde_json::Value = serde_json::from_str(&out_cmd).unwrap();
    assert_eq!(
        parsed_cmd["hookSpecificOutput"]["permissionDecision"],
        "allow"
    );

    // 3. Writing documentation / task specification without active task
    let doc_payload = serde_json::json!({
        "name": "FileWrite",
        "input": { "file_path": "tasks/027-test.md" }
    })
    .to_string();
    let (code_doc, out_doc) = adapter.handle_hook(ws, &doc_payload);
    assert_eq!(code_doc, 0);
    let parsed_doc: serde_json::Value = serde_json::from_str(&out_doc).unwrap();
    assert_eq!(
        parsed_doc["hookSpecificOutput"]["permissionDecision"],
        "allow"
    );
}

// ============================================================================
// Klynge 3: End-to-End Mock-Proces Livscyklus
// ============================================================================

#[test]
fn test_subprocess_hook_mock_execution_claude_code() {
    let bin = find_xgauntlet_binary();
    if !bin.is_file() {
        eprintln!("Binary not found at {:?}, skipping subprocess test", bin);
        return;
    }

    let temp = TempDir::new("subprocess_hook_test");
    let ws = &temp.path;

    use std::io::Write;

    // 1. Claude Code Allow -> subprocess exits with 0 and returns structured allow JSON
    let read_payload = serde_json::json!({
        "name": "FileRead",
        "input": { "file_path": "README.md" }
    })
    .to_string();

    let mut child = Command::new(&bin)
        .args([
            "hook",
            "--harness",
            "claude_code",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn xgauntlet hook process");

    child
        .stdin
        .as_mut()
        .expect("stdin available")
        .write_all(read_payload.as_bytes())
        .expect("write to stdin");

    let output = child.wait_with_output().expect("wait on child");
    assert_eq!(
        output.status.code(),
        Some(0),
        "Claude Code allowed tool MUST exit with code 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON stdout");
    assert_eq!(
        parsed["hookSpecificOutput"]["permissionDecision"],
        "allow"
    );

    // 2. Claude Code Deny -> subprocess exits with 2 (official blocking code)
    let push_payload = serde_json::json!({
        "name": "Bash",
        "input": { "command": "git push origin main" }
    })
    .to_string();

    let mut child_deny = Command::new(&bin)
        .args([
            "hook",
            "--harness",
            "claude_code",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn xgauntlet hook process");

    child_deny
        .stdin
        .as_mut()
        .expect("stdin available")
        .write_all(push_payload.as_bytes())
        .expect("write to stdin");

    let output_deny = child_deny.wait_with_output().expect("wait on child");
    assert_eq!(
        output_deny.status.code(),
        Some(2),
        "Claude Code denied tool MUST cause process to exit with code 2"
    );
    let stdout_deny = String::from_utf8_lossy(&output_deny.stdout);
    let parsed_deny: serde_json::Value =
        serde_json::from_str(stdout_deny.trim()).expect("valid JSON stdout");
    assert_eq!(
        parsed_deny["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );
}

#[test]
fn test_subprocess_hook_mock_execution_antigravity() {
    let bin = find_xgauntlet_binary();
    if !bin.is_file() {
        return;
    }

    let temp = TempDir::new("subprocess_antigravity_test");
    let ws = &temp.path;

    use std::io::Write;

    // 1. Antigravity Allow -> exit code 0
    let read_payload = serde_json::json!({
        "toolCall": {
            "name": "view_file",
            "args": { "AbsolutePath": "README.md" }
        }
    })
    .to_string();

    let mut child = Command::new(&bin)
        .args([
            "hook",
            "--harness",
            "antigravity",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(read_payload.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(parsed["decision"], "allow");

    // 2. Antigravity Deny -> exit code 1
    let push_payload = serde_json::json!({
        "toolCall": {
            "name": "run_command",
            "args": { "CommandLine": "git push origin main" }
        }
    })
    .to_string();

    let mut child_deny = Command::new(&bin)
        .args([
            "hook",
            "--harness",
            "antigravity",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child_deny.stdin.as_mut().unwrap().write_all(push_payload.as_bytes()).unwrap();
    let output_deny = child_deny.wait_with_output().unwrap();
    assert_eq!(output_deny.status.code(), Some(1));
    let stdout_deny = String::from_utf8_lossy(&output_deny.stdout);
    let parsed_deny: serde_json::Value = serde_json::from_str(stdout_deny.trim()).unwrap();
    assert_eq!(parsed_deny["decision"], "deny");
}

#[test]
fn test_subprocess_hook_via_native_shell_piping() {
    let bin = find_xgauntlet_binary();
    if !bin.is_file() {
        return;
    }

    let temp = TempDir::new("subprocess_shell_test");
    let ws = &temp.path;

    if cfg!(unix) {
        let bin_str = bin.to_str().unwrap();
        let ws_str = ws.to_str().unwrap();

        // Allow case -> exit 0
        let allow_cmd = format!(
            "echo '{{\"name\": \"FileRead\", \"input\": {{\"file_path\": \"README.md\"}}}}' | '{}' hook --harness claude_code --workspace '{}'",
            bin_str, ws_str
        );
        let out_allow = Command::new("sh").args(["-c", &allow_cmd]).output().unwrap();
        assert_eq!(out_allow.status.code(), Some(0));

        // Deny case -> exit 2
        let deny_cmd = format!(
            "echo '{{\"name\": \"Bash\", \"input\": {{\"command\": \"git push origin main\"}}}}' | '{}' hook --harness claude_code --workspace '{}'",
            bin_str, ws_str
        );
        let out_deny = Command::new("sh").args(["-c", &deny_cmd]).output().unwrap();
        assert_eq!(out_deny.status.code(), Some(2));
    } else if cfg!(windows) {
        let bin_str = bin.to_str().unwrap();
        let ws_str = ws.to_str().unwrap();

        let deny_cmd = format!(
            "echo {{\"name\": \"Bash\", \"input\": {{\"command\": \"git push origin main\"}}}} | \"{}\" hook --harness claude_code --workspace \"{}\"",
            bin_str, ws_str
        );
        let out_deny = Command::new("cmd").args(["/C", &deny_cmd]).output().unwrap();
        assert_eq!(out_deny.status.code(), Some(2));
    }
}

#[test]
fn test_subprocess_telemetry_lifecycle() {
    let bin = find_xgauntlet_binary();
    if !bin.is_file() {
        return;
    }

    let temp = TempDir::new("subprocess_telemetry_test");
    let ws = &temp.path;

    let tasks_dir = ws.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    fs::write(
        tasks_dir.join("001-init.md"),
        "---\ntype: Task Package\ntitle: Task 001\nstatus: active\n---\n# Task 001\n\n## Acceptance Criteria\n- [ ] Criteria 1\n",
    )
    .unwrap();

    let output = Command::new(&bin)
        .args([
            "telemetry",
            "--harness",
            "claude_code",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .output()
        .expect("execute telemetry");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON stdout");
    assert_eq!(
        parsed["hookSpecificOutput"]["hookEventName"],
        "PostToolUse"
    );
    assert!(
        parsed["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    );
}

// ============================================================================
// Klynge 4: Wasm Policy Engine Sti- og Linjeskifts-Determinisme
// ============================================================================

#[test]
fn test_wasm_policy_determinism_windows_absolute_paths() {
    let mut engine = WasmPolicyEngine::new().expect("WasmPolicyEngine initialize");
    let ctx = EnforcementContext {
        workspace_id: "test-workspace".into(),
        has_active_task: true,
        active_task_id: "027".into(),
        read_only: false,
    };

    let illegal_windows_paths = [
        r"C:\repo\src\main.rs",
        r"c:\Users\user\project\lib.rs",
        r"D:\workspace\crates\xgauntlet-core\src\lib.rs",
        r"Z:\network\drive\exploit.sh",
        r"\\server\share\file.txt",
        r"\\?\C:\repo\secret.key",
    ];

    for path in illegal_windows_paths {
        let req = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write_to_file".into(),
            target_resource: path.into(),
            payload_json: "{}".into(),
        };

        let decision = engine
            .evaluate(&req, &ctx)
            .expect("Wasm evaluation should succeed");

        assert_eq!(
            decision.verdict,
            DecisionVerdict::Deny,
            "Illegal Windows path '{}' MUST be denied",
            path
        );
        assert_eq!(
            decision.reason_code, 4036,
            "Illegal Windows path '{}' must trigger fail-closed reason code 4036",
            path
        );
    }
}

#[test]
fn test_wasm_policy_determinism_windows_backslashes_parity() {
    let mut engine = WasmPolicyEngine::new().expect("WasmPolicyEngine initialize");

    let pairs = [
        (
            r"crates\xgauntlet-core\src\lib.rs",
            "crates/xgauntlet-core/src/lib.rs",
            true, // is production code
        ),
        (
            r"src\features\adapters\mod.rs",
            "src/features/adapters/mod.rs",
            true,
        ),
        (
            r"tasks\027-test.md",
            "tasks/027-test.md",
            false, // is documentation
        ),
        (
            r"docs\adr\0006-multi-harness.md",
            "docs/adr/0006-multi-harness.md",
            false,
        ),
    ];

    // Case A: Without active task
    let ctx_no_task = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: false,
        active_task_id: "".into(),
        read_only: false,
    };

    for (win_path, unix_path, is_code) in &pairs {
        let req_win = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write".into(),
            target_resource: win_path.to_string(),
            payload_json: "{}".into(),
        };
        let req_unix = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write".into(),
            target_resource: unix_path.to_string(),
            payload_json: "{}".into(),
        };

        let dec_win = engine.evaluate(&req_win, &ctx_no_task).unwrap();
        let dec_unix = engine.evaluate(&req_unix, &ctx_no_task).unwrap();

        assert_eq!(
            dec_win.verdict, dec_unix.verdict,
            "Windows and Unix path must yield identical verdict for '{}'",
            win_path
        );
        assert_eq!(
            dec_win.reason_code, dec_unix.reason_code,
            "Windows and Unix path must yield identical reason_code for '{}'",
            win_path
        );

        if *is_code {
            assert_eq!(dec_win.verdict, DecisionVerdict::Deny);
            assert_eq!(dec_win.reason_code, 4031);
        } else {
            assert_eq!(dec_win.verdict, DecisionVerdict::Allow);
            assert_eq!(dec_win.reason_code, 2001);
        }
    }

    // Case B: With active task
    let ctx_active = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "027".into(),
        read_only: false,
    };

    for (win_path, unix_path, is_code) in &pairs {
        let req_win = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write".into(),
            target_resource: win_path.to_string(),
            payload_json: "{}".into(),
        };
        let req_unix = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write".into(),
            target_resource: unix_path.to_string(),
            payload_json: "{}".into(),
        };

        let dec_win = engine.evaluate(&req_win, &ctx_active).unwrap();
        let dec_unix = engine.evaluate(&req_unix, &ctx_active).unwrap();

        assert_eq!(dec_win.verdict, DecisionVerdict::Allow);
        assert_eq!(dec_win.verdict, dec_unix.verdict);
        assert_eq!(dec_win.reason_code, dec_unix.reason_code);

        if *is_code {
            assert_eq!(dec_win.reason_code, 2002);
        } else {
            assert_eq!(dec_win.reason_code, 2001);
        }
    }
}

#[test]
fn test_wasm_policy_determinism_crlf_newlines() {
    let mut engine = WasmPolicyEngine::new().expect("WasmPolicyEngine initialize");
    let ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "027".into(),
        read_only: false,
    };

    // 1. CRLF in ExecuteCommand target_resource -> must Deny (reason code 4039)
    let crlf_commands = [
        "git status\r\n",
        "echo hello\r\nrm -rf /",
        "cargo check\r\ngit push",
        "pytest\r",
    ];

    for cmd in crlf_commands {
        let req = CapabilityRequest {
            action_type: ToolActionType::ExecuteCommand,
            raw_tool_name: "Bash".into(),
            target_resource: cmd.into(),
            payload_json: "{}".into(),
        };
        let decision = engine.evaluate(&req, &ctx).unwrap();
        assert_eq!(
            decision.verdict,
            DecisionVerdict::Deny,
            "CRLF in command '{}' MUST be denied",
            cmd
        );
        assert_eq!(
            decision.reason_code, 4039,
            "CRLF command injection must trigger reason code 4039"
        );
    }

    // 2. CRLF inside payload_json file contents -> must evaluate cleanly and Allow
    let crlf_payload = serde_json::json!({
        "file_path": "crates/xgauntlet-core/src/lib.rs",
        "content": "pub mod test;\r\n\r\npub fn run() -> bool {\r\n    true\r\n}\r\n"
    })
    .to_string();

    let req_content_crlf = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "filewrite".into(),
        target_resource: "crates/xgauntlet-core/src/lib.rs".into(),
        payload_json: crlf_payload,
    };

    let decision_content = engine.evaluate(&req_content_crlf, &ctx).unwrap();
    assert_eq!(decision_content.verdict, DecisionVerdict::Allow);
    assert_eq!(decision_content.reason_code, 2002);

    // 3. Pretty-printed payload_json with CRLF indentation -> must evaluate cleanly
    let pretty_crlf_json =
        "{\r\n  \"file_path\": \"crates/xgauntlet-core/src/lib.rs\",\r\n  \"content\": \"test\"\r\n}";
    let req_pretty_crlf = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "filewrite".into(),
        target_resource: "crates/xgauntlet-core/src/lib.rs".into(),
        payload_json: pretty_crlf_json.into(),
    };

    let decision_pretty = engine.evaluate(&req_pretty_crlf, &ctx).unwrap();
    assert_eq!(decision_pretty.verdict, DecisionVerdict::Allow);
    assert_eq!(decision_pretty.reason_code, 2002);
}
