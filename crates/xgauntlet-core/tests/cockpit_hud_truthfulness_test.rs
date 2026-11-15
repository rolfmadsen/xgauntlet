use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::adapters::antigravity::validation::validate_plugin;
use xgauntlet_core::features::adapters::AntigravityAdapter;
use xgauntlet_core::features::evidence::{
    save_verification_report, CheckStatus, CheckSummary, VerificationReport, VerificationVerdict,
};
use xgauntlet_core::features::scaffold::generate_templates;
use xgauntlet_core::features::tasks::inspect_task_telemetry;
use xgauntlet_core::features::telemetry::{SessionHealth, SessionHealthStatus};
use xgauntlet_core::HarnessAdapter;

static TEMP_SEQ: AtomicU64 = AtomicU64::new(1);

fn make_temp_dir(label: &str) -> PathBuf {
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "xgauntlet_t094_{label}_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn git_ok(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must spawn");
    assert!(
        out.status.success(),
        "git {:?} failed in {:?}: {}",
        args,
        dir,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init_git_repo(dir: &Path) {
    git_ok(dir, &["init", "-b", "main"]);
    git_ok(dir, &["config", "user.email", "test@example.com"]);
    git_ok(dir, &["config", "user.name", "Tester"]);
}

fn write_task(ws: &Path, id: &str, status: &str, scope: &[&str], ac_lines: &[&str]) {
    let tasks_dir = ws.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let scope_yaml = if scope.is_empty() {
        String::new()
    } else {
        let items: String = scope.iter().map(|s| format!("  - \"{s}\"\n")).collect();
        format!("scope:\n{items}")
    };
    let ac_body: String = ac_lines.iter().map(|l| format!("{l}\n")).collect();
    let status_upper = status.to_ascii_uppercase();
    let content = format!(
        "---\n\
         type: Task Package\n\
         title: \"Task {id}\"\n\
         description: \"HUD truthfulness test\"\n\
         status: {status}\n\
         intent: enhancement\n\
         generated: {{ by: process:test, at: \"2026-11-15T14:00:00Z\" }}\n\
         tags: [hud, telemetry]\n\
         {scope_yaml}\
         ---\n\n\
         # Task {id}\n\n\
         **Status**: `{status_upper}`\n\
         **Intent**: `🔄 ENHANCEMENT`\n\n\
         ## 🎯 Formål\nTest HUD truthfulness.\n\n\
         ## 📋 Acceptance Criteria\n{ac_body}\n\
         ## 🚫 Must NOT\n- Never fabricate pass.\n"
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).unwrap();
}

#[test]
fn test_ac1_session_health_token_saturation_thresholds() {
    assert_eq!(
        SessionHealth::new(3, 1_000, 1_000).status,
        SessionHealthStatus::Optimal
    );
    assert_eq!(
        SessionHealth::new(3, 1_000, 65_000).status,
        SessionHealthStatus::Aging,
        "65k estimated tokens must escalate SessionHealth from Optimal to Aging even at 3 turns"
    );
    assert_eq!(
        SessionHealth::new(3, 1_000, 125_000).status,
        SessionHealthStatus::Saturated,
        "125k estimated tokens must escalate SessionHealth to Saturated even at 3 turns"
    );
    assert_eq!(
        SessionHealth::new(11, 1_000, 1_000).status,
        SessionHealthStatus::Aging
    );
    assert_eq!(
        SessionHealth::new(19, 1_000, 1_000).status,
        SessionHealthStatus::Saturated
    );
}

#[test]
fn test_ac1_pre_invocation_reads_per_session_health_from_conversation_id() {
    let ws = make_temp_dir("session_conv_id");
    init_git_repo(&ws);
    write_task(
        &ws,
        "094-session-hud",
        "active",
        &["crates/telemetry/**"],
        &["- [ ] AC1"],
    );
    fs::write(ws.join("README.md"), "hello\n").unwrap();
    git_ok(&ws, &["add", "."]);
    git_ok(&ws, &["commit", "-m", "task(094): init spec"]);

    let payload = serde_json::json!({
        "hookEventName": "PreInvocation",
        "conversationId": "conv-hud-094-alpha",
        "invocationNum": 1,
        "cwd": ws.to_string_lossy()
    });

    let p1 = AntigravityAdapter::prepare_pre_invocation_payload(
        &ws,
        None,
        Some(&payload),
        &payload.to_string(),
    )
    .expect("PreInvocation #1 must succeed");
    let p2 = AntigravityAdapter::prepare_pre_invocation_payload(
        &ws,
        None,
        Some(&payload),
        &payload.to_string(),
    )
    .expect("PreInvocation #2 must succeed");

    let sys_msg = p2["injectSteps"][0]["systemMessage"]["systemMessage"]
        .as_str()
        .unwrap_or("");
    assert!(
        sys_msg.contains("Session: 🟢 Optimal (2 turns"),
        "PreInvocation with conversationId must read .xgauntlet/sessions/<hash>.json and show 2 turns, got p1={p1:?}\np2={p2:?}"
    );

    let telemetry = inspect_task_telemetry(&ws, Some("094")).expect("inspect must succeed");
    assert_eq!(
        telemetry.session_health.as_ref().map(|s| s.turns_count),
        Some(2),
        "inspect_task_telemetry fallback must locate active named session health when session_state.json is absent"
    );

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_ac1_truthful_verification_freshness_scope_hygiene_and_worktree() {
    let root = make_temp_dir("truthful_telemetry");
    let primary = root.join("repo");
    let worktree = primary.join(".worktrees").join("094");
    fs::create_dir_all(&primary).unwrap();
    init_git_repo(&primary);

    fs::create_dir_all(primary.join("crates/telemetry")).unwrap();
    fs::write(primary.join("crates/telemetry/lib.rs"), "pub fn v1() {}\n").unwrap();
    fs::write(primary.join("README.md"), "# Root\n").unwrap();
    fs::write(primary.join(".gitignore"), ".xgauntlet/\n.worktrees/\n").unwrap();
    write_task(
        &primary,
        "094-hud-truth",
        "active",
        &["crates/telemetry/**"],
        &["- [ ] Verify telemetry", "- [x] Done step"],
    );
    git_ok(&primary, &["add", "."]);
    git_ok(
        &primary,
        &["commit", "-m", "feat(094): init workspace [GREEN]"],
    );

    git_ok(
        &primary,
        &[
            "worktree",
            "add",
            "-b",
            "task/094",
            worktree.to_str().unwrap(),
        ],
    );
    fs::create_dir_all(worktree.join(".xgauntlet")).unwrap();
    fs::write(worktree.join(".xgauntlet/active_task"), "094-hud-truth\n").unwrap();

    // (a) No verification-report.json -> must NEVER claim PASS, 14/14 PASS, Mutation: 100%, Drift: 0%, or crates/*
    let telemetry_no_rep =
        inspect_task_telemetry(&worktree, None).expect("inspect in worktree must succeed");
    let eph_no_rep = AntigravityAdapter::format_ephemeral_telemetry(&telemetry_no_rep);
    assert!(
        !eph_no_rep.contains("14/14 PASS"),
        "Must not fabricate 14/14 PASS: {eph_no_rep}"
    );
    assert!(
        !eph_no_rep.contains("Mutation: 100%"),
        "Must not fabricate Mutation: 100%: {eph_no_rep}"
    );
    assert!(
        !eph_no_rep.contains("Drift: 0%"),
        "Must not fabricate Drift: 0%: {eph_no_rep}"
    );
    assert!(
        !eph_no_rep.contains("Verdict: PASS"),
        "Must not guess Verdict: PASS from GREEN phase without verification-report.json: {eph_no_rep}"
    );
    assert_eq!(
        telemetry_no_rep.scope.as_deref(),
        Some("crates/telemetry/**"),
        "TaskTelemetry must populate real task YAML scope"
    );
    let hud_no_rep = AntigravityAdapter::render_blockquote_hud(&telemetry_no_rep, None);
    assert!(
        hud_no_rep.contains("Gauntlet: NONE"),
        "Missing verification-report.json must render Gauntlet: NONE, got:\n{hud_no_rep}"
    );
    assert!(
        hud_no_rep.contains("wt:") || hud_no_rep.contains("worktree"),
        "Worktree checkout must be surfaced in HUD, got:\n{hud_no_rep}"
    );

    // (b) Fresh matching verification-report.json -> PASS with real checks (not Mutation: 100% when skipped)
    let head_oid = git_ok(&worktree, &["rev-parse", "HEAD"]);
    let mut rep = VerificationReport::new_local("094-hud-truth");
    rep.verdict = VerificationVerdict::Passed;
    rep.workspace_state.vcs = Some(xgauntlet_core::features::evidence::VcsMetadata {
        vcs_type: "git".to_string(),
        head: "task/094".to_string(),
        commit: head_oid,
        is_dirty: false,
    });
    rep.checks = vec![
        CheckSummary {
            name: "unit".to_string(),
            status: CheckStatus::Passed,
            exit_code: 0,
            duration_seconds: 1.0,
            optional: false,
            passed: true,
            log_digest: "abc".to_string(),
        },
        CheckSummary {
            name: "mutation-testing-gauntlet".to_string(),
            status: CheckStatus::Skipped,
            exit_code: 0,
            duration_seconds: 0.0,
            optional: true,
            passed: true,
            log_digest: String::new(),
        },
    ];
    save_verification_report(&worktree, &rep).unwrap();

    let telemetry_fresh = inspect_task_telemetry(&worktree, None).unwrap();
    let hud_fresh = AntigravityAdapter::render_blockquote_hud(&telemetry_fresh, None);
    assert!(
        hud_fresh.contains("Gauntlet: PASS"),
        "Fresh report must render Gauntlet: PASS, got:\n{hud_fresh}"
    );
    let eph_fresh = AntigravityAdapter::format_ephemeral_telemetry(&telemetry_fresh);
    assert!(
        !eph_fresh.contains("Mutation: 100%"),
        "Skipped mutation layer must not display Mutation: 100%: {eph_fresh}"
    );

    // (c) Untracked/modified out-of-scope file -> verification becomes STALE and Scope Hygiene flags OOS file
    fs::write(worktree.join("README.md"), "# Modified out of scope\n").unwrap();
    let telemetry_dirty = inspect_task_telemetry(&worktree, None).unwrap();
    let hud_dirty = AntigravityAdapter::render_blockquote_hud(&telemetry_dirty, None);
    assert!(
        hud_dirty.contains("Gauntlet: STALE"),
        "Dirty tree must mark sealed report as STALE, got:\n{hud_dirty}"
    );
    assert!(
        hud_dirty.contains("OOS") || hud_dirty.contains("out-of-scope"),
        "Out-of-scope dirty edit must be flagged by Scope Hygiene, got:\n{hud_dirty}"
    );

    // (d) Wrong task_id in report on clean tree -> STALE
    fs::write(worktree.join("README.md"), "# Root\n").unwrap();
    let mut wrong_task_rep = rep.clone();
    wrong_task_rep.task_contract.task_id = "093-other-task".to_string();
    save_verification_report(&worktree, &wrong_task_rep).unwrap();
    let telemetry_wrong_task = inspect_task_telemetry(&worktree, None).unwrap();
    let hud_wrong_task = AntigravityAdapter::render_blockquote_hud(&telemetry_wrong_task, None);
    assert!(
        hud_wrong_task.contains("Gauntlet: STALE"),
        "VerificationReport from task 093 must be STALE for task 094, got:\n{hud_wrong_task}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_ac2_antigravity_system_message_user_markdown_smart_cadence() {
    let ws = make_temp_dir("smart_cadence");
    init_git_repo(&ws);
    write_task(
        &ws,
        "094-cadence",
        "active",
        &["crates/telemetry/**"],
        &["- [ ] Step 1", "- [ ] Step 2"],
    );
    fs::write(ws.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
    git_ok(&ws, &["add", "."]);
    git_ok(&ws, &["commit", "-m", "task(094): spec"]);

    // Verify hooks.json validator accepts SessionStart
    let plugin_dir = ws.join(".agents");
    fs::create_dir_all(&plugin_dir).unwrap();
    let hooks_cfg = serde_json::json!({
        "hooks": {
            "SessionStart": [{ "command": "xgauntlet telemetry --format antigravity-hook" }]
        }
    });
    fs::write(plugin_dir.join("hooks.json"), hooks_cfg.to_string()).unwrap();
    let val_res = validate_plugin(&plugin_dir);
    assert!(
        val_res.valid,
        "Antigravity hook schema validation must recognize SessionStart: {:?}",
        val_res.issues
    );

    // 1. Prompt 1 (invocationNum: 1, session start) -> Full Start-HUD in metadata.user_markdown
    let p1_req = serde_json::json!({
        "hookEventName": "PreInvocation",
        "conversationId": "conv-cadence-1",
        "invocationNum": 1,
        "lastUserInput": "Start task",
        "cwd": ws.to_string_lossy()
    });
    let p1 = AntigravityAdapter::prepare_pre_invocation_payload(
        &ws,
        None,
        Some(&p1_req),
        &p1_req.to_string(),
    )
    .unwrap();
    assert!(
        p1["injectSteps"][0].get("ephemeralMessage").is_none(),
        "Deprecated ephemeralMessage must not be emitted: {p1}"
    );
    let sys1 = &p1["injectSteps"][0]["systemMessage"];
    assert!(
        sys1["systemMessage"]
            .as_str()
            .unwrap_or("")
            .contains("[XGAUNTLET COCKPIT TELEMETRY]"),
        "systemMessage must carry model telemetry: {p1}"
    );
    let md1 = sys1["metadata"]["user_markdown"].as_str().unwrap_or("");
    assert!(
        md1.contains("### 🛡️") && md1.lines().count() >= 4,
        "First prompt must emit full Start-HUD in metadata.user_markdown, got:\n{md1}"
    );

    // 2. Prompt 2 (invocationNum: 1, unchanged structural state) -> Compact 1-line status chip
    let p2_req = serde_json::json!({
        "hookEventName": "PreInvocation",
        "conversationId": "conv-cadence-1",
        "invocationNum": 1,
        "lastUserInput": "Continue",
        "cwd": ws.to_string_lossy()
    });
    let p2 = AntigravityAdapter::prepare_pre_invocation_payload(
        &ws,
        None,
        Some(&p2_req),
        &p2_req.to_string(),
    )
    .unwrap();
    let md2 = p2["injectSteps"][0]["systemMessage"]["metadata"]["user_markdown"]
        .as_str()
        .unwrap_or("");
    assert_eq!(
        md2.lines().count(),
        1,
        "Unchanged subsequent user prompt must collapse user_markdown to a 1-line compact chip, got:\n{md2}"
    );
    assert!(
        md2.starts_with("> 🛡️"),
        "Compact chip must start with blockquote shield prefix, got: {md2}"
    );

    // 3. Internal tool loop (invocationNum: 2, unchanged state) -> user_markdown NEVER emitted
    let loop_same_req = serde_json::json!({
        "hookEventName": "PreInvocation",
        "conversationId": "conv-cadence-1",
        "invocationNum": 2,
        "cwd": ws.to_string_lossy()
    });
    let p_loop_same = AntigravityAdapter::prepare_pre_invocation_payload(
        &ws,
        None,
        Some(&loop_same_req),
        &loop_same_req.to_string(),
    )
    .unwrap();
    assert!(
        p_loop_same["injectSteps"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["systemMessage"]["metadata"]["user_markdown"].is_null()),
        "Internal tool loops (invocationNum > 1) must suppress metadata.user_markdown: {p_loop_same}"
    );

    // 4. Criteria progress transitions (0/2 -> 1/2), then tool loop (invocationNum: 3) -> model systemMessage updates, user_markdown STILL suppressed!
    write_task(
        &ws,
        "094-cadence",
        "active",
        &["crates/telemetry/**"],
        &["- [x] Step 1", "- [ ] Step 2"],
    );
    let loop_changed_req = serde_json::json!({
        "hookEventName": "PreInvocation",
        "conversationId": "conv-cadence-1",
        "invocationNum": 3,
        "cwd": ws.to_string_lossy()
    });
    let p_loop_changed = AntigravityAdapter::prepare_pre_invocation_payload(
        &ws,
        None,
        Some(&loop_changed_req),
        &loop_changed_req.to_string(),
    )
    .unwrap();
    assert!(
        p_loop_changed["injectSteps"][0]["systemMessage"]["metadata"]["user_markdown"].is_null(),
        "Tool loop even with state delta must omit user_markdown: {p_loop_changed}"
    );

    // 5. Next user prompt (invocationNum: 1) after another structural delta (1/2 -> 2/2) -> re-expands full Start-HUD
    write_task(
        &ws,
        "094-cadence",
        "active",
        &["crates/telemetry/**"],
        &["- [x] Step 1", "- [x] Step 2"],
    );
    let p5_req = serde_json::json!({
        "hookEventName": "PreInvocation",
        "conversationId": "conv-cadence-1",
        "invocationNum": 1,
        "lastUserInput": "Check progress",
        "cwd": ws.to_string_lossy()
    });
    let p5 = AntigravityAdapter::prepare_pre_invocation_payload(
        &ws,
        None,
        Some(&p5_req),
        &p5_req.to_string(),
    )
    .unwrap();
    let md5 = p5["injectSteps"][0]["systemMessage"]["metadata"]["user_markdown"]
        .as_str()
        .unwrap_or("");
    assert!(
        md5.contains("### 🛡️") && md5.lines().count() >= 4,
        "State transition on new user prompt must re-expand full Start-HUD, got:\n{md5}"
    );

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_ac2_task_none_renders_read_only_banner_everywhere() {
    let ws = make_temp_dir("task_none");
    init_git_repo(&ws);
    write_task(
        &ws,
        "094-draft-only",
        "draft",
        &["crates/**"],
        &["- [ ] Draft step"],
    );
    git_ok(&ws, &["add", "."]);
    git_ok(&ws, &["commit", "-m", "chore: draft task"]);

    let req = serde_json::json!({
        "hookEventName": "PreInvocation",
        "conversationId": "conv-none-1",
        "invocationNum": 1,
        "cwd": ws.to_string_lossy()
    });

    let prepared =
        AntigravityAdapter::prepare_pre_invocation_payload(&ws, None, Some(&req), &req.to_string())
            .expect("Task: NONE must produce Read-Only telemetry payload instead of Err");

    let sys = prepared["injectSteps"][0]["systemMessage"]["systemMessage"]
        .as_str()
        .unwrap_or("");
    let user_md = prepared["injectSteps"][0]["systemMessage"]["metadata"]["user_markdown"]
        .as_str()
        .unwrap_or("");

    assert!(
        sys.contains("Task: NONE") && sys.contains("Read-Only"),
        "NoActiveTask model context must state Task: NONE and Read-Only policy, got:\n{sys}"
    );
    assert!(
        !sys.contains("14/14 PASS"),
        "Task: NONE must not fabricate 14/14 PASS: {sys}"
    );
    assert!(
        user_md.contains("Read-Only") && user_md.contains("main@"),
        "Task: NONE must display compact Read-Only banner in user_markdown, got:\n{user_md}"
    );

    let (code, hook_out) = AntigravityAdapter::new().handle_hook(&ws, &req.to_string());
    assert_eq!(code, 0);
    assert!(
        !hook_out.contains("14/14 PASS") && !hook_out.contains("ephemeralMessage"),
        "handle_hook on Task: NONE must share truthful Read-Only systemMessage path, got:\n{hook_out}"
    );

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_ac3_render_box_card_and_agents_md_cadence_truthfulness() {
    let box_ws = make_temp_dir("box_truth");
    init_git_repo(&box_ws);
    write_task(
        &box_ws,
        "094-hud-truth",
        "active",
        &[],
        &["- [ ] One", "- [x] Two"],
    );
    git_ok(&box_ws, &["add", "."]);
    git_ok(&box_ws, &["commit", "-m", "feat(094): green [GREEN]"]);
    let telemetry = inspect_task_telemetry(&box_ws, None).unwrap();

    let card = telemetry.render_box_card();
    let lines: Vec<&str> = card.lines().collect();
    assert_eq!(lines.len(), 6);
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(
            line.chars().count(),
            64,
            "Box card line {} must be 64 cols: {line}",
            idx + 1
        );
    }
    assert!(
        !card.contains("14/14 PASS"),
        "render_box_card must not default to fake '14/14 PASS':\n{card}"
    );
    assert!(
        !card.contains("crates/*"),
        "render_box_card must not default to fake 'crates/*':\n{card}"
    );

    // Check scaffolded AGENTS.md and repo .agents/AGENTS.md cadence documentation
    let templates = generate_templates("rust", "hud-demo");
    let scaffolded_agents = templates
        .into_iter()
        .find(|t| t.relative_path == ".agents/AGENTS.md")
        .map(|t| t.content)
        .expect("Scaffold templates must include AGENTS.md");
    let repo_agents = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(".agents/AGENTS.md"),
    )
    .unwrap();

    for (label, doc) in [
        ("scaffolded AGENTS.md", scaffolded_agents),
        (".agents/AGENTS.md", repo_agents),
    ] {
        assert!(
            doc.contains("STALE") && doc.contains("NONE"),
            "{label} must document truthful Gauntlet statuses (<PASS | FAIL | STALE | NONE>)"
        );
        assert!(
            doc.contains("user_markdown") || doc.contains("Start-HUD"),
            "{label} must document that Start-HUD is delivered via PreInvocation and agent replies only emit HUD on state delta / escalation / handoff"
        );
    }

    let _ = fs::remove_dir_all(&box_ws);
}
