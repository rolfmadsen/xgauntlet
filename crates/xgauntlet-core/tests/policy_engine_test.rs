use xgauntlet_core::features::policy::{
    CapabilityRequest, DecisionVerdict, EnforcementContext, PolicyEvaluator, ToolActionType,
    WasmPolicyEngine,
};

#[test]
fn test_wasm_policy_engine_cold_start() {
    let mut engine = WasmPolicyEngine::new().expect("WasmPolicyEngine should initialize cleanly");
    let req = CapabilityRequest {
        action_type: ToolActionType::ReadFile,
        raw_tool_name: "view_file".into(),
        target_resource: "crates/xgauntlet-core/src/lib.rs".into(),
        payload_json: "{}".into(),
    };
    let ctx = EnforcementContext {
        workspace_id: "test-workspace".into(),
        has_active_task: false,
        active_task_id: "".into(),
        read_only: false,
    };

    let start = std::time::Instant::now();
    let decision = engine
        .evaluate(&req, &ctx)
        .expect("Evaluation should succeed");
    let elapsed = start.elapsed();

    assert_eq!(decision.verdict, DecisionVerdict::Allow);
    assert_eq!(decision.reason_code, 2000);
    // Sub-5ms execution requirement
    assert!(
        elapsed.as_millis() < 50,
        "Cold execution took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_read_operations_unrestricted() {
    let mut engine = WasmPolicyEngine::new().unwrap();
    let req = CapabilityRequest {
        action_type: ToolActionType::ReadFile,
        raw_tool_name: "grep_search".into(),
        target_resource: "src/main.rs".into(),
        payload_json: "{}".into(),
    };
    let ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: false,
        active_task_id: "".into(),
        read_only: false,
    };

    let decision = engine.evaluate(&req, &ctx).unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Allow);
    assert_eq!(decision.reason_code, 2000);
}

#[test]
fn test_read_only_workspace_blocks_modifications() {
    let mut engine = WasmPolicyEngine::new().unwrap();
    let req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "tasks/001-bootstrap.md".into(),
        payload_json: "{}".into(),
    };
    let ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "001".into(),
        read_only: true, // Locked!
    };

    let decision = engine.evaluate(&req, &ctx).unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Deny);
    assert_eq!(decision.reason_code, 4030);
    assert!(decision.reason.contains("read-only mode"));
}

#[test]
fn test_documentation_and_tasks_writable_without_active_task() {
    let mut engine = WasmPolicyEngine::new().unwrap();

    let safe_targets = [
        "tasks/002-test.md",
        "docs/adr/0008-new.md",
        "spec.md",
        "CONTEXT.md",
        "CODING_STANDARDS.md",
        "README.md",
    ];

    let ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: false,
        active_task_id: "".into(),
        read_only: false,
    };

    for target in safe_targets {
        let req = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write_to_file".into(),
            target_resource: target.into(),
            payload_json: "{}".into(),
        };
        let decision = engine.evaluate(&req, &ctx).unwrap();
        assert_eq!(
            decision.verdict,
            DecisionVerdict::Allow,
            "Target {} should be writable without active task",
            target
        );
        assert_eq!(decision.reason_code, 2001);
    }
}

#[test]
fn test_production_code_write_requires_active_task() {
    let mut engine = WasmPolicyEngine::new().unwrap();

    let protected_targets = [
        "crates/xgauntlet-core/src/lib.rs",
        "src/main.rs",
        "tests/integration.rs",
        "packages/cli/bin/xgauntlet.js",
        ".agents/hooks.json",
    ];

    let no_task_ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: false,
        active_task_id: "".into(),
        read_only: false,
    };

    for target in protected_targets {
        let req = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write_to_file".into(),
            target_resource: target.into(),
            payload_json: "{}".into(),
        };
        let decision = engine.evaluate(&req, &no_task_ctx).unwrap();
        assert_eq!(
            decision.verdict,
            DecisionVerdict::Deny,
            "Target {} should be denied without active task",
            target
        );
        assert_eq!(decision.reason_code, 4031);
    }

    let active_task_ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "002-wasmtime-policy-engine".into(),
        read_only: false,
    };

    for target in protected_targets {
        let req = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write_to_file".into(),
            target_resource: target.into(),
            payload_json: "{}".into(),
        };
        let decision = engine.evaluate(&req, &active_task_ctx).unwrap();
        assert_eq!(
            decision.verdict,
            DecisionVerdict::Allow,
            "Target {} should be allowed under active task",
            target
        );
        assert_eq!(decision.reason_code, 2002);
    }
}

#[test]
fn test_destructive_commands_strictly_denied() {
    let mut engine = WasmPolicyEngine::new().unwrap();

    let dangerous_commands = [
        "git push origin main",
        "git push --force",
        "git reset --hard HEAD~1",
        "git clean -fd",
        "git branch -D feature-branch",
        "rm -rf /",
    ];

    let ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "002".into(),
        read_only: false,
    };

    for cmd in dangerous_commands {
        let req = CapabilityRequest {
            action_type: ToolActionType::ExecuteCommand,
            raw_tool_name: "run_command".into(),
            target_resource: cmd.into(),
            payload_json: "{}".into(),
        };
        let decision = engine.evaluate(&req, &ctx).unwrap();
        assert_eq!(
            decision.verdict,
            DecisionVerdict::Deny,
            "Command '{}' must be denied",
            cmd
        );
        assert_eq!(decision.reason_code, 4039);
        assert!(decision.reason.contains("strictly prohibited"));
    }
}

#[test]
fn test_verification_and_inspection_commands_whitelisted() {
    let mut engine = WasmPolicyEngine::new().unwrap();

    let safe_commands = [
        "git status",
        "git diff",
        "git log -n 5",
        "cargo test --workspace",
        "cargo clippy --workspace",
        "cargo check",
        "cargo fmt --check",
        "npm test",
        "npm run test",
        "xgauntlet verify",
        "xgauntlet check-evidence",
        "xgauntlet check-spec",
        "xgauntlet doctor",
        "ls -la",
        "pwd",
        "which cargo",
    ];

    let no_task_ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: false,
        active_task_id: "".into(),
        read_only: false,
    };

    for cmd in safe_commands {
        let req = CapabilityRequest {
            action_type: ToolActionType::ExecuteCommand,
            raw_tool_name: "run_command".into(),
            target_resource: cmd.into(),
            payload_json: "{}".into(),
        };
        let decision = engine.evaluate(&req, &no_task_ctx).unwrap();
        assert_eq!(
            decision.verdict,
            DecisionVerdict::Allow,
            "Safe command '{}' should be allowed without active task",
            cmd
        );
        assert_eq!(decision.reason_code, 2004);
    }
}
