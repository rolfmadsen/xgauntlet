use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xgauntlet_core::features::adapters::{
    get_adapter, AntigravityAdapter, ClaudeCodeAdapter, CodexAdapter, MistralAdapter,
};
use xgauntlet_core::features::checkpoint::{
    run_checkpoint, stage_workspace_changes, CheckpointError, CheckpointOptions, CheckpointPhase,
};
use xgauntlet_core::features::policy::{
    CapabilityRequest, DecisionVerdict, EnforcementContext, PolicyEvaluator, ToolActionType,
    WasmPolicyEngine,
};
use xgauntlet_core::features::tasks::{CriteriaProgress, GitTelemetry, TaskStatus, TaskTelemetry};

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

    fn path(&self) -> &Path {
        &self.path
    }
}

fn init_git_repo(path: &Path) {
    let _ = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output()
        .or_else(|_| {
            Command::new("git")
                .args(["init"])
                .current_dir(path)
                .output()
        });

    let _ = Command::new("git")
        .args(["config", "user.name", "Test Committer"])
        .current_dir(path)
        .output();

    let _ = Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output();
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
        hook_output
            .get("permissionDecision")
            .and_then(|v| v.as_str()),
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
        hook_output
            .get("permissionDecision")
            .and_then(|v| v.as_str()),
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
    assert_eq!(parsed["hookSpecificOutput"]["permissionDecision"], "allow");

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

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(read_payload.as_bytes())
        .unwrap();
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

    child_deny
        .stdin
        .as_mut()
        .unwrap()
        .write_all(push_payload.as_bytes())
        .unwrap();
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
        let out_allow = Command::new("sh")
            .args(["-c", &allow_cmd])
            .output()
            .unwrap();
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

        let shell = if Command::new("powershell").arg("-v").output().is_ok() {
            "powershell"
        } else if Command::new("pwsh").arg("-v").output().is_ok() {
            "pwsh"
        } else {
            "cmd"
        };

        if shell == "powershell" || shell == "pwsh" {
            // Allow case -> exit 0
            let allow_payload = r#"{"name": "FileRead", "input": {"file_path": "README.md"}}"#;
            let allow_cmd = format!(
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Write-Output '{}' | & '{}' hook --harness claude_code --workspace '{}'; exit $LASTEXITCODE",
                allow_payload.replace('\'', "''"),
                bin_str.replace('\'', "''"),
                ws_str.replace('\'', "''"),
            );
            let out_allow = Command::new(shell)
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &allow_cmd,
                ])
                .output()
                .unwrap();
            assert_eq!(
                out_allow.status.code(),
                Some(0),
                "PowerShell allow stdout: {}, stderr: {}",
                String::from_utf8_lossy(&out_allow.stdout),
                String::from_utf8_lossy(&out_allow.stderr)
            );

            // Deny case -> exit 2
            let deny_payload = r#"{"name": "Bash", "input": {"command": "git push origin main"}}"#;
            let deny_cmd = format!(
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Write-Output '{}' | & '{}' hook --harness claude_code --workspace '{}'; exit $LASTEXITCODE",
                deny_payload.replace('\'', "''"),
                bin_str.replace('\'', "''"),
                ws_str.replace('\'', "''"),
            );
            let out_deny = Command::new(shell)
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &deny_cmd,
                ])
                .output()
                .unwrap();
            assert_eq!(
                out_deny.status.code(),
                Some(2),
                "PowerShell deny stdout: {}, stderr: {}",
                String::from_utf8_lossy(&out_deny.stdout),
                String::from_utf8_lossy(&out_deny.stderr)
            );
        }
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
    assert_eq!(parsed["hookSpecificOutput"]["hookEventName"], "PostToolUse");
    assert!(parsed["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .map(|s| !s.is_empty())
        .unwrap_or(false));
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

// ============================================================================
// Klynge 1: Kontrakt-, Schema- og Snapshot-Validering (Alle 4 Adaptere)
// ============================================================================

#[test]
fn test_harness_scaffold_config_snapshots_all_four_adapters() {
    let temp = TempDir::new("harness_scaffold_snapshots");
    let ws = &temp.path;

    // 1. Google Antigravity: .agents/hooks.json
    let ag_path = AntigravityAdapter::scaffold_hooks(ws).expect("scaffold antigravity hooks");
    assert!(ag_path.is_file(), ".agents/hooks.json must be created");
    let ag_raw = fs::read_to_string(&ag_path).expect("read .agents/hooks.json");
    let ag_json: serde_json::Value =
        serde_json::from_str(&ag_raw).expect(".agents/hooks.json must be valid JSON");

    let xgauntlet_obj = ag_json
        .get("xgauntlet")
        .expect("must contain top-level 'xgauntlet' key");
    assert_eq!(
        xgauntlet_obj.get("enabled"),
        Some(&serde_json::Value::Bool(true))
    );
    let pre_tool = xgauntlet_obj
        .get("PreToolUse")
        .and_then(|v| v.as_array())
        .expect("PreToolUse array");
    assert_eq!(pre_tool[0]["matcher"], "*");
    assert_eq!(
        pre_tool[0]["hooks"][0]["command"],
        "xgauntlet hook antigravity"
    );
    let pre_inv = xgauntlet_obj
        .get("PreInvocation")
        .and_then(|v| v.as_array())
        .expect("PreInvocation array");
    assert_eq!(pre_inv[0]["matcher"], ".*");
    assert_eq!(
        pre_inv[0]["hooks"][0]["command"],
        "xgauntlet telemetry --format antigravity-hook"
    );

    // 2. Claude Code: .claude/settings.json
    let claude_path = ClaudeCodeAdapter::scaffold_settings(ws).expect("scaffold claude settings");
    assert!(
        claude_path.is_file(),
        ".claude/settings.json must be created"
    );
    let claude_raw = fs::read_to_string(&claude_path).expect("read .claude/settings.json");
    let claude_json: serde_json::Value =
        serde_json::from_str(&claude_raw).expect(".claude/settings.json must be valid JSON");

    let claude_post = claude_json["hooks"]["PostToolUse"]
        .as_array()
        .expect("hooks.PostToolUse array");
    assert_eq!(claude_post[0]["matcher"], "Edit|Write");
    assert_eq!(
        claude_post[0]["hooks"][0]["command"],
        "xgauntlet telemetry --format claude-hook"
    );

    // 3. OpenAI Codex: .codex/hooks.json
    let codex_path = CodexAdapter::scaffold_hooks(ws).expect("scaffold codex hooks");
    assert!(codex_path.is_file(), ".codex/hooks.json must be created");
    let codex_raw = fs::read_to_string(&codex_path).expect("read .codex/hooks.json");
    let codex_json: serde_json::Value =
        serde_json::from_str(&codex_raw).expect(".codex/hooks.json must be valid JSON");

    let codex_post = codex_json["hooks"]["PostToolUse"]
        .as_array()
        .expect("hooks.PostToolUse array");
    assert_eq!(codex_post[0]["matcher"], "apply_patch|Edit|Write|Bash");
    assert_eq!(
        codex_post[0]["hooks"][0]["command"],
        "xgauntlet telemetry --format codex-hook"
    );

    // 4. Mistral Vibe: .vibe/hooks.toml
    let vibe_path = MistralAdapter::scaffold_hooks(ws).expect("scaffold mistral hooks");
    assert!(vibe_path.is_file(), ".vibe/hooks.toml must be created");
    let vibe_raw = fs::read_to_string(&vibe_path).expect("read .vibe/hooks.toml");

    assert!(
        vibe_raw.contains("[[hooks]]"),
        "Must contain TOML [[hooks]] tables"
    );
    assert!(
        vibe_raw.contains("name = \"xgauntlet-gatekeeper\""),
        "Must declare xgauntlet-gatekeeper hook"
    );
    assert!(
        vibe_raw.contains("type = \"pre_tool\""),
        "Must declare pre_tool type"
    );
    assert!(
        vibe_raw.contains("command = \"xgauntlet hook --harness mistral\""),
        "Must declare gatekeeper command"
    );
    assert!(
        vibe_raw.contains("strict = true"),
        "Gatekeeper hook must be strict"
    );
    assert!(
        vibe_raw.contains("name = \"xgauntlet-hud\""),
        "Must declare xgauntlet-hud hook"
    );
    assert!(
        vibe_raw.contains("type = \"post_tool\""),
        "Must declare post_tool type"
    );
    assert!(
        vibe_raw.contains("command = \"xgauntlet telemetry --format mistral-hook\""),
        "Must declare HUD telemetry command"
    );
}

#[test]
fn test_telemetry_and_gatekeeper_payload_schema_compliance_all_four_adapters() {
    let temp = TempDir::new("payload_schema_test");
    let ws = &temp.path;
    let card = "┌─── xgauntlet: Task 027 ───┐\n│ Status: ACTIVE             │\n└───────────────────────────┘";

    // 1. Google Antigravity
    // PreInvocation payload
    let ag_pre_inv = AntigravityAdapter::format_pre_invocation_payload(card);
    assert!(ag_pre_inv.is_object());
    let inject_steps = ag_pre_inv["injectSteps"]
        .as_array()
        .expect("injectSteps array");
    assert_eq!(inject_steps[0]["ephemeralMessage"].as_str(), Some(card));

    // PreToolUse gatekeeper payload
    let ag_adapter = get_adapter("antigravity").unwrap();
    let (ag_code_allow, ag_out_allow) = ag_adapter.handle_hook(
        ws,
        &serde_json::json!({
            "toolCall": { "name": "view_file", "args": { "AbsolutePath": "README.md" } }
        })
        .to_string(),
    );
    assert_eq!(ag_code_allow, 0);
    let ag_json_allow: serde_json::Value = serde_json::from_str(&ag_out_allow).unwrap();
    assert_eq!(ag_json_allow["decision"], "allow");

    let (ag_code_deny, ag_out_deny) = ag_adapter.handle_hook(
        ws,
        &serde_json::json!({
            "toolCall": { "name": "run_command", "args": { "CommandLine": "git push origin main" } }
        })
        .to_string(),
    );
    assert_eq!(ag_code_deny, 1);
    let ag_json_deny: serde_json::Value = serde_json::from_str(&ag_out_deny).unwrap();
    assert_eq!(ag_json_deny["decision"], "deny");
    assert!(ag_json_deny["reason"].as_str().is_some());

    // 2. Claude Code
    // PostToolUse telemetry payload
    let claude_post = ClaudeCodeAdapter::format_post_tool_use_payload(card);
    assert_eq!(
        claude_post["hookSpecificOutput"]["hookEventName"],
        "PostToolUse"
    );
    assert_eq!(claude_post["hookSpecificOutput"]["additionalContext"], card);

    // PreToolUse gatekeeper payload
    let claude_adapter = get_adapter("claude_code").unwrap();
    let (claude_code_allow, claude_out_allow) = claude_adapter.handle_hook(
        ws,
        &serde_json::json!({
            "name": "FileRead",
            "input": { "file_path": "README.md" }
        })
        .to_string(),
    );
    assert_eq!(claude_code_allow, 0);
    let claude_json_allow: serde_json::Value = serde_json::from_str(&claude_out_allow).unwrap();
    assert_eq!(
        claude_json_allow["hookSpecificOutput"]["hookEventName"],
        "PreToolUse"
    );
    assert_eq!(
        claude_json_allow["hookSpecificOutput"]["permissionDecision"],
        "allow"
    );

    let (claude_code_deny, claude_out_deny) = claude_adapter.handle_hook(
        ws,
        &serde_json::json!({
            "name": "Bash",
            "input": { "command": "git push origin main" }
        })
        .to_string(),
    );
    assert_eq!(claude_code_deny, 2);
    let claude_json_deny: serde_json::Value = serde_json::from_str(&claude_out_deny).unwrap();
    assert_eq!(
        claude_json_deny["hookSpecificOutput"]["hookEventName"],
        "PreToolUse"
    );
    assert_eq!(
        claude_json_deny["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );
    assert!(
        claude_json_deny["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .is_some()
    );

    // 3. Mistral Vibe
    // PostToolUse telemetry payload
    let mistral_post = MistralAdapter::format_post_tool_use_payload(card);
    assert_eq!(
        mistral_post["hook_specific_output"]["additional_context"],
        card
    );

    // pre_tool gatekeeper payload
    let mistral_adapter = get_adapter("mistral").unwrap();
    let (mistral_code_allow, mistral_out_allow) = mistral_adapter.handle_hook(
        ws,
        &serde_json::json!({
            "hook_event_name": "pre_tool",
            "tool_name": "read_file",
            "tool_input": { "path": "README.md" }
        })
        .to_string(),
    );
    assert_eq!(mistral_code_allow, 0);
    let mistral_json_allow: serde_json::Value = serde_json::from_str(&mistral_out_allow).unwrap();
    assert_eq!(mistral_json_allow["decision"], "allow");

    let (mistral_code_deny, mistral_out_deny) = mistral_adapter.handle_hook(
        ws,
        &serde_json::json!({
            "hook_event_name": "pre_tool",
            "tool_name": "bash",
            "tool_input": { "command": "git push origin main" }
        })
        .to_string(),
    );
    assert_eq!(mistral_code_deny, 0);
    let mistral_json_deny: serde_json::Value = serde_json::from_str(&mistral_out_deny).unwrap();
    assert_eq!(mistral_json_deny["decision"], "deny");
    assert!(mistral_json_deny["reason"].as_str().is_some());

    // 4. OpenAI Codex
    // PostToolUse telemetry payload
    let codex_post = CodexAdapter::format_post_tool_use_payload(card);
    assert_eq!(
        codex_post["hookSpecificOutput"]["hookEventName"],
        "PostToolUse"
    );
    assert_eq!(codex_post["hookSpecificOutput"]["additionalContext"], card);

    // PreToolUse gatekeeper payload
    let codex_adapter = get_adapter("codex").unwrap();
    let (codex_code_allow, codex_out_allow) = codex_adapter.handle_hook(
        ws,
        &serde_json::json!({
            "type": "function",
            "function": {
                "name": "read_file",
                "arguments": "{\"path\":\"README.md\"}"
            }
        })
        .to_string(),
    );
    assert_eq!(codex_code_allow, 0);
    let codex_json_allow: serde_json::Value = serde_json::from_str(&codex_out_allow).unwrap();
    assert_eq!(codex_json_allow["decision"], "allow");

    let (codex_code_deny, codex_out_deny) = codex_adapter.handle_hook(
        ws,
        &serde_json::json!({
            "type": "function",
            "function": {
                "name": "execute_command",
                "arguments": "{\"command\":\"git push origin main\"}"
            }
        })
        .to_string(),
    );
    assert_eq!(codex_code_deny, 1);
    let codex_json_deny: serde_json::Value = serde_json::from_str(&codex_out_deny).unwrap();
    assert_eq!(codex_json_deny["decision"], "deny");
}

// ============================================================================
// Klynge 5: Filsystem- og Checkpoint-Resilience
// ============================================================================

fn setup_mock_glossary(workspace: &Path) {
    let glossary = r#"---
type: Knowledge Bundle Index
title: "xGauntlet Context & Domain Glossary"
description: "Test glossary"
status: stable
---

# xGauntlet Context & Domain Glossary

**Task**:
An executable unit of engineering work, that has bounded acceptance criteria.
_Avoid_: Ticket, issue.

**Phase Checkpoint**:
A verified local Git commit, that binds the workspace to a lifecycle phase.
_Avoid_: Quick save, commit hook.
"#;
    fs::write(workspace.join("CONTEXT.md"), glossary).unwrap();
}

fn setup_mock_task(workspace: &Path, task_id: &str, status: &str) {
    let tasks_dir = workspace.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let content = format!(
        r#"---
type: Task Package
title: "Task {task_id}: Test Task"
description: "Test task description"
status: {status}
---

# Task {task_id}: Test Task

**Status**: `{status}`

## 🎯 Formål
Test task purpose.

## 📋 Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## 🚫 Must NOT
- Must not violate invariants.
"#
    );
    fs::write(tasks_dir.join(format!("{task_id}.md")), content).unwrap();
}

#[tokio::test]
async fn test_git_checkpoint_lockfile_conflict_fails_closed() {
    let temp = TempDir::new("checkpoint_lockfile_conflict");
    let ws = temp.path();
    init_git_repo(ws);

    // Create an initial commit so repo is initialized
    fs::write(ws.join("README.md"), "# Test Repo\n").unwrap();
    let initial_files = stage_workspace_changes(ws).expect("initial staging");
    assert!(!initial_files.is_empty());

    let _ = Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(ws)
        .output()
        .unwrap();

    // Setup mock glossary and active task
    setup_mock_glossary(ws);
    setup_mock_task(ws, "027-test-task", "active");

    // Modify a file
    fs::write(ws.join("README.md"), "# Test Repo Updated\n").unwrap();

    // Simulate concurrent git process or lockfile conflict by creating .git/index.lock
    let lockfile = ws.join(".git").join("index.lock");
    fs::write(&lockfile, "locked").unwrap();
    assert!(lockfile.is_file(), ".git/index.lock must exist");

    // Attempting to stage MUST fail closed with CheckpointError::GitError
    let stage_res = stage_workspace_changes(ws);
    assert!(
        stage_res.is_err(),
        "Staging MUST fail closed when .git/index.lock is present"
    );
    match stage_res {
        Err(CheckpointError::GitError(msg)) => {
            assert!(
                msg.contains("failed") || msg.contains("exit code"),
                "Error message must indicate git failure: {}",
                msg
            );
        }
        other => panic!("Expected GitError, got: {:?}", other),
    }

    // Attempting checkpoint directly must also fail closed without panic
    let opts = CheckpointOptions::new(CheckpointPhase::Spec, ws);
    let cp_res = run_checkpoint(&opts).await;
    assert!(
        cp_res.is_err(),
        "Checkpoint MUST fail closed when index is locked"
    );

    // Clean up lockfile and verify checkpoint now succeeds cleanly
    fs::remove_file(&lockfile).unwrap();
    let cp_success = run_checkpoint(&opts).await;
    assert!(
        cp_success.is_ok(),
        "Checkpoint must succeed after removing index.lock: {:?}",
        cp_success.err()
    );
    let record = cp_success.unwrap();
    assert!(record.commit_oid.is_some());
}

#[tokio::test]
async fn test_preflight_verification_failure_leaves_clean_workspace() {
    let temp = TempDir::new("checkpoint_preflight_clean");
    let ws = temp.path();
    init_git_repo(ws);

    fs::write(ws.join("README.md"), "# Preflight Cleanliness\n").unwrap();
    let _ = Command::new("git")
        .args(["add", "-A"])
        .current_dir(ws)
        .output()
        .unwrap();
    let _ = Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(ws)
        .output()
        .unwrap();

    // Create an uncommitted change
    fs::write(ws.join("uncommitted.txt"), "fresh data\n").unwrap();

    // Try to run a Spec checkpoint without active task in tasks/ -> NoActiveTask error
    let opts = CheckpointOptions::new(CheckpointPhase::Spec, ws);
    let res = run_checkpoint(&opts).await;
    assert!(res.is_err(), "Checkpoint without active task MUST fail");
    match res {
        Err(CheckpointError::NoActiveTask) => {}
        other => panic!("Expected NoActiveTask error, got: {:?}", other),
    }

    // Workspace must not have corrupted commits or modified HEAD
    let log_out = Command::new("git")
        .args(["log", "-n", "1", "--oneline"])
        .current_dir(ws)
        .output()
        .unwrap();
    let log_str = String::from_utf8_lossy(&log_out.stdout);
    assert!(
        log_str.contains("chore: initial commit"),
        "HEAD commit must remain untouched after failed preflight"
    );
    assert!(
        ws.join("uncommitted.txt").is_file(),
        "User file must remain intact"
    );
}

// ============================================================================
// Klynge 6: HUD & Terminal Rendering Resilience
// ============================================================================

#[test]
fn test_render_box_card_multibyte_utf8_exact_64_columns_and_no_panic() {
    let telemetry = TaskTelemetry {
        task_id: "027-harness-æøå-ünicöde-🚀".to_string(),
        title: "Test multi-byte: ÆØÅ, üöä, 日本語, 🛡️, 📦".to_string(),
        status: TaskStatus::Active,
        intent: Some("🚀 NY FUNKTION".to_string()),
        criteria: CriteriaProgress {
            total: 10,
            completed: 4,
            pending: 6,
            percentage: 40,
            bar: "[████░░░░░░]".to_string(),
        },
        git: GitTelemetry {
            branch: "feature/æøå-branch-長いブランチ名".to_string(),
            head_oid: "abcdef0".to_string(),
            is_clean: false,
            dirty_count: 5,
        },
        file_path: "tasks/027-æøå.md".to_string(),
        scope: Some("crates/xgauntlet-core/æøå".to_string()),
        invariants: Some("14/14 PASS (æøå)".to_string()),
        evidence: Some("forseglet".to_string()),
        phase: Some("GREEN".to_string()),
    };

    // Rendering must NEVER panic on multi-byte UTF-8
    let card = telemetry.render_box_card();
    let lines: Vec<&str> = card.lines().collect();

    assert_eq!(lines.len(), 6, "Box card must have exactly 6 lines");

    for (idx, line) in lines.iter().enumerate() {
        let char_count = line.chars().count();
        assert_eq!(
            char_count, 64,
            "Line {} character count must be exactly 64 (got {}, content: '{}')",
            idx, char_count, line
        );
    }
}

#[test]
fn test_render_blockquote_hud_multibyte_and_redirected_resilience() {
    let telemetry = TaskTelemetry {
        task_id: "027".to_string(),
        title: "Platform Hardening & Resilience".to_string(),
        status: TaskStatus::Active,
        intent: Some("NEW FEATURE".to_string()),
        criteria: CriteriaProgress {
            total: 24,
            completed: 13,
            pending: 11,
            percentage: 54,
            bar: "[█████░░░░░]".to_string(),
        },
        git: GitTelemetry {
            branch: "main".to_string(),
            head_oid: "add3b98".to_string(),
            is_clean: true,
            dirty_count: 0,
        },
        file_path: "tasks/027.md".to_string(),
        scope: Some("crates/*".to_string()),
        invariants: Some("14/14 PASS".to_string()),
        evidence: Some("pending".to_string()),
        phase: Some("GREEN".to_string()),
    };

    // Very long next action with multi-byte characters
    let long_action = "Fortsæt venligst med Klynge 5 & 6: afprøvning af filsystem-låse (.git/index.lock) og validering af UTF-8 (æøå, emojis 🛡️ 🚀 🧪) uden panics.".repeat(5);

    let hud = AntigravityAdapter::render_blockquote_hud(&telemetry, Some(&long_action));
    let lines: Vec<&str> = hud.lines().collect();

    assert_eq!(lines.len(), 5, "Blockquote HUD must have exactly 5 lines");
    assert!(lines[0].starts_with("> ### 🛡️ [Task: "));
    assert!(lines[1].starts_with("> **Status**: "));
    assert!(lines[2].starts_with("> **Progress**: "));
    assert!(lines[3].starts_with("> **Links**: "));
    assert!(lines[4].starts_with("> 💡 **Next Action:** "));

    // Safe to write to file / redirect to pipe without ANSI escapes or non-UTF-8 bytes
    assert!(
        !hud.contains("\x1b["),
        "Markdown HUD must not contain raw ANSI escape sequences"
    );
}
