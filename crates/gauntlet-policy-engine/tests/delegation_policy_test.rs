use gauntlet_policy_engine::{
    evaluate, parse_capability_request, DecisionVerdict, EnforcementContext,
};

fn eval_json_req(req_json: &str, ctx: &EnforcementContext) -> gauntlet_policy_engine::PolicyDecision {
    let req = parse_capability_request(req_json);
    evaluate(&req, ctx)
}

#[test]
fn test_read_only_delegation_allowed_without_active_task() {
    let ctx = EnforcementContext {
        workspace_id: "xgauntlet".into(),
        has_active_task: false,
        active_task_id: String::new(),
        read_only: false,
        ..Default::default()
    };

    let req_json = r#"{
        "action_type": "delegate",
        "raw_tool_name": "invoke_subagent",
        "target_resource": "read_only",
        "payload_json": "{\"TypeName\":\"research\",\"Workspace\":\"inherit\"}"
    }"#;

    let dec = eval_json_req(req_json, &ctx);
    assert_eq!(
        dec.verdict,
        DecisionVerdict::Allow,
        "Read-only delegation must be allowed even without active task: {:?}",
        dec
    );
}

#[test]
fn test_isolated_mutating_delegation_requires_active_task_and_allows_when_active() {
    let no_task_ctx = EnforcementContext {
        workspace_id: "xgauntlet".into(),
        has_active_task: false,
        active_task_id: String::new(),
        read_only: false,
        ..Default::default()
    };

    let active_ctx = EnforcementContext {
        workspace_id: "xgauntlet".into(),
        has_active_task: true,
        active_task_id: "088-worktree-isolated-multi-agent-supervision".into(),
        read_only: false,
        allowed_scopes: vec!["crates/**".into()],
        allowed_test_targets: vec![],
    };

    let req_json = r#"{
        "action_type": "delegate",
        "raw_tool_name": "swarm",
        "target_resource": "isolated",
        "payload_json": "{\"action\":\"add\",\"name\":\"worker-1\",\"workspace\":\"share\"}"
    }"#;

    let denied = eval_json_req(req_json, &no_task_ctx);
    assert_eq!(denied.verdict, DecisionVerdict::Deny);
    assert_eq!(denied.reason_code, 4034);

    let allowed = eval_json_req(req_json, &active_ctx);
    assert_eq!(
        allowed.verdict,
        DecisionVerdict::Allow,
        "Isolated worktree swarm worker must be allowed with active task: {:?}",
        allowed
    );
}

#[test]
fn test_shared_inherit_mutating_delegation_denied_fail_closed() {
    let active_ctx = EnforcementContext {
        workspace_id: "xgauntlet".into(),
        has_active_task: true,
        active_task_id: "088-worktree-isolated-multi-agent-supervision".into(),
        read_only: false,
        allowed_scopes: vec!["crates/**".into()],
        allowed_test_targets: vec![],
    };

    let req_json = r#"{
        "action_type": "delegate",
        "raw_tool_name": "swarm",
        "target_resource": "shared_inherit",
        "payload_json": "{\"action\":\"add\",\"name\":\"worker-1\",\"workspace\":\"inherit\"}"
    }"#;

    let dec = eval_json_req(req_json, &active_ctx);
    assert_eq!(dec.verdict, DecisionVerdict::Deny);
    assert_eq!(
        dec.reason_code, 4040,
        "Concurrent mutating worker in shared 'inherit' workspace must be denied with 4040: {:?}",
        dec
    );
}

#[test]
fn test_read_only_context_blocks_isolated_mutating_delegation() {
    let read_only_ctx = EnforcementContext {
        workspace_id: "xgauntlet".into(),
        has_active_task: true,
        active_task_id: "088".into(),
        read_only: true,
        allowed_scopes: vec!["crates/**".into()],
        allowed_test_targets: vec![],
    };

    let req_json = r#"{
        "action_type": "delegate",
        "raw_tool_name": "swarm",
        "target_resource": "isolated",
        "payload_json": "{\"action\":\"add\",\"name\":\"worker-1\",\"workspace\":\"share\"}"
    }"#;

    let dec = eval_json_req(req_json, &read_only_ctx);
    assert_eq!(dec.verdict, DecisionVerdict::Deny);
    assert_eq!(dec.reason_code, 4030);
}
