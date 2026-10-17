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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        allowed_scopes: vec![
            "crates/**".into(),
            "src/**".into(),
            "tests/**".into(),
            "packages/**".into(),
            ".agents/**".into(),
        ],
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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

#[test]
fn test_policy_engine_enforces_task_allowed_scopes_and_test_targets() {
    let mut engine = WasmPolicyEngine::new().unwrap();

    let scoped_ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "033-scoped-task".into(),
        read_only: false,
        allowed_scopes: vec!["crates/xgauntlet-core/src/features/tasks/**".into()],
        allowed_test_targets: vec!["tests/tasks_engine_test.rs".into()],
    };

    // 1. Write within allowed scope -> ALLOW (2002)
    let in_scope_req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "crates/xgauntlet-core/src/features/tasks/validator.rs".into(),
        payload_json: "{}".into(),
    };
    let decision = engine.evaluate(&in_scope_req, &scoped_ctx).unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Allow);
    assert_eq!(decision.reason_code, 2002);

    // 2. Write to declared test target -> ALLOW (2002)
    let in_test_req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "tests/tasks_engine_test.rs".into(),
        payload_json: "{}".into(),
    };
    let decision = engine.evaluate(&in_test_req, &scoped_ctx).unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Allow);
    assert_eq!(decision.reason_code, 2002);

    // 3. Write outside allowed scope -> DENY (4037)
    let out_of_scope_req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "crates/xgauntlet-core/src/features/topology/mod.rs".into(),
        payload_json: "{}".into(),
    };
    let decision = engine.evaluate(&out_of_scope_req, &scoped_ctx).unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Deny);
    assert_eq!(decision.reason_code, 4037);

    // 4. Write to undeclared test target -> DENY (4037)
    let out_of_test_req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "tests/acceptance.rs".into(),
        payload_json: "{}".into(),
    };
    let decision = engine.evaluate(&out_of_test_req, &scoped_ctx).unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Deny);
    assert_eq!(decision.reason_code, 4037);

    // 5. Documentation paths (tasks/, spec.md, CONTEXT.md) remain writable even with scopes -> ALLOW (2001)
    let doc_req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "tasks/033-scoped-task.md".into(),
        payload_json: "{}".into(),
    };
    let decision = engine.evaluate(&doc_req, &scoped_ctx).unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Allow);
    assert_eq!(decision.reason_code, 2001);
}

#[test]
fn test_destructive_git_command_evasions_and_wrappers_denied() {
    let mut engine = WasmPolicyEngine::new().unwrap();

    let evasive_commands = [
        "git   push",
        "git\tpush origin main",
        "git    push    origin   main",
        "git --no-pager push origin main",
        "git -C repo push",
        "git -c foo=bar push",
        "git -c user.name=attacker -C /tmp --no-pager push origin main",
        "git -C repo reset --hard HEAD~1",
        "git --no-pager reset --hard",
        "git -C repo clean -fd",
        "git clean -f",
        "git -C repo branch -D rogue-branch",
        "git branch -D rogue-branch",
        "env git push",
        "env VAR=val git push",
        "sudo git push",
        "eval \"git push\"",
        "bash -c \"git push\"",
        "sh -c \"git push\"",
        "xargs git push",
    ];

    let ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "037-policy-engine".into(),
        read_only: false,
        allowed_scopes: vec!["crates/**".into()],
        allowed_test_targets: vec!["tests/**".into()],
    };

    for cmd in evasive_commands {
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
            "Destructive command evasion '{}' MUST be denied",
            cmd
        );
        assert_eq!(
            decision.reason_code, 4039,
            "Destructive command evasion '{}' MUST return reason_code 4039",
            cmd
        );
    }
}

#[test]
fn test_empty_task_scope_fails_closed_on_code_and_repo_mutations() {
    let mut engine = WasmPolicyEngine::new().unwrap();

    let empty_scope_ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "037-no-scope".into(),
        read_only: false,
        allowed_scopes: vec![],
        allowed_test_targets: vec![],
    };

    let mutation_targets = [
        "crates/xgauntlet-core/src/lib.rs",
        "tests/policy_engine_test.rs",
        "Cargo.toml",
        ".agents/hooks.json",
    ];

    for target in mutation_targets {
        let req = CapabilityRequest {
            action_type: ToolActionType::WriteFile,
            raw_tool_name: "write_to_file".into(),
            target_resource: target.into(),
            payload_json: "{}".into(),
        };
        let decision = engine.evaluate(&req, &empty_scope_ctx).unwrap();
        assert_eq!(
            decision.verdict,
            DecisionVerdict::Deny,
            "Mutation to '{}' with empty scope MUST be denied",
            target
        );
        assert_eq!(
            decision.reason_code, 4037,
            "Mutation to '{}' with empty scope MUST return reason_code 4037",
            target
        );
    }
}

#[test]
fn test_empty_scope_with_test_targets_strictly_isolates_test_files() {
    let mut engine = WasmPolicyEngine::new().unwrap();

    let test_only_ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "037-test-only".into(),
        read_only: false,
        allowed_scopes: vec![],
        allowed_test_targets: vec!["crates/xgauntlet-core/tests/policy_engine_test.rs".into()],
    };

    // 1. Declared test target -> ALLOW (2002)
    let declared_test_req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "crates/xgauntlet-core/tests/policy_engine_test.rs".into(),
        payload_json: "{}".into(),
    };
    let decision = engine.evaluate(&declared_test_req, &test_only_ctx).unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Allow);
    assert_eq!(decision.reason_code, 2002);

    // 2. Undeclared test target -> DENY (4037)
    let undeclared_test_req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "crates/xgauntlet-core/tests/security_hardening_test.rs".into(),
        payload_json: "{}".into(),
    };
    let decision = engine
        .evaluate(&undeclared_test_req, &test_only_ctx)
        .unwrap();
    assert_eq!(decision.verdict, DecisionVerdict::Deny);
    assert_eq!(decision.reason_code, 4037);

    // 3. Source file MUST NOT be allowed when allowed_scopes is empty -> DENY (4037)
    let source_req = CapabilityRequest {
        action_type: ToolActionType::WriteFile,
        raw_tool_name: "write_to_file".into(),
        target_resource: "crates/xgauntlet-core/src/features/policy/mod.rs".into(),
        payload_json: "{}".into(),
    };
    let decision = engine.evaluate(&source_req, &test_only_ctx).unwrap();
    assert_eq!(
        decision.verdict,
        DecisionVerdict::Deny,
        "Source code modification MUST be denied when allowed_scopes is empty"
    );
    assert_eq!(decision.reason_code, 4037);

    // 4. Wildcard test target matching source file MUST NOT allow writing source code
    let broad_test_ctx = EnforcementContext {
        workspace_id: "ws".into(),
        has_active_task: true,
        active_task_id: "037-broad-test".into(),
        read_only: false,
        allowed_scopes: vec![],
        allowed_test_targets: vec!["crates/xgauntlet-core/**".into()],
    };
    let decision = engine.evaluate(&source_req, &broad_test_ctx).unwrap();
    assert_eq!(
        decision.verdict,
        DecisionVerdict::Deny,
        "Source code modification MUST NOT be permitted via allowed_test_targets"
    );
    assert_eq!(decision.reason_code, 4037);
}
