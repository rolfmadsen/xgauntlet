use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::config::{validate_config, GauntletConfig};
use xgauntlet_core::features::scaffold::{
    run_scaffold, ScaffoldAction, ScaffoldError, ScaffoldOptions,
};
use xgauntlet_core::features::tasks::{check_task_specification, validate_context_glossary};

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
        let path = std::env::temp_dir().join(format!("xgauntlet_test_{}", unique));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_safe_non_destructive_guarantee_preserves_existing_files() {
    let temp = TempDir::new("scaffold_safe");
    let ws = temp.path();

    // 1. Pre-create existing files with custom content
    let custom_context = "MY PRE-EXISTING CONTEXT CONTENT";
    let custom_config = "# PRE-EXISTING GAUNTLET TOML";
    fs::write(ws.join("CONTEXT.md"), custom_context).unwrap();
    fs::write(ws.join("gauntlet.toml"), custom_config).unwrap();

    // 2. Run scaffold without force
    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: Some("rust".to_string()),
        force: false,
        dry_run: false,
        project_name: Some("test-project".to_string()),
    };

    let result = run_scaffold(&options).expect("scaffold should succeed");
    assert!(result.is_success);
    assert!(result.skipped_count >= 2);

    // Verify existing files were NOT overwritten
    let read_context = fs::read_to_string(ws.join("CONTEXT.md")).unwrap();
    assert_eq!(read_context, custom_context);

    let read_config = fs::read_to_string(ws.join("gauntlet.toml")).unwrap();
    assert_eq!(read_config, custom_config);

    // Verify previously missing files WERE created
    assert!(ws.join("spec.md").is_file());
    assert!(ws.join("CODING_STANDARDS.md").is_file());
    assert!(ws.join("tasks/001-bootstrap.md").is_file());
    assert!(ws.join(".agents/AGENTS.md").is_file());
    assert!(ws.join(".agents/hooks.json").is_file());
    assert!(ws.join("CLAUDE.md").is_file());
    assert!(ws
        .join("docs/adr/0001-package-by-feature-architecture.md")
        .is_file());
}

#[test]
fn test_force_flag_overwrites_existing_files() {
    let temp = TempDir::new("scaffold_force");
    let ws = temp.path();

    // Pre-create existing file
    let custom_context = "PRE-EXISTING OBSOLETE CONTEXT";
    fs::write(ws.join("CONTEXT.md"), custom_context).unwrap();

    // Run scaffold with force = true
    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: Some("rust".to_string()),
        force: true,
        dry_run: false,
        project_name: Some("test-force".to_string()),
    };

    let result = run_scaffold(&options).expect("scaffold should succeed with force");
    assert!(result.is_success);
    assert!(result.overwritten_count >= 1);

    // Verify the file was overwritten with template content
    let read_context = fs::read_to_string(ws.join("CONTEXT.md")).unwrap();
    assert_ne!(read_context, custom_context);
    assert!(read_context.contains("Context & Domain Glossary"));
}

#[test]
fn test_dry_run_does_not_modify_disk() {
    let temp = TempDir::new("scaffold_dry_run");
    let ws = temp.path();

    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: Some("rust".to_string()),
        force: false,
        dry_run: true,
        project_name: Some("test-dry-run".to_string()),
    };

    let result = run_scaffold(&options).expect("scaffold dry-run should succeed");
    assert!(result.is_success);
    assert_eq!(result.created_count, 0);
    assert!(!result.files.is_empty());

    // Check all reported actions are WouldCreate
    for file in &result.files {
        assert_eq!(file.action, ScaffoldAction::WouldCreate);
    }

    // Check that workspace directory remains empty
    let entries: Vec<_> = fs::read_dir(ws).unwrap().collect();
    assert!(
        entries.is_empty(),
        "dry-run must not create any files on disk"
    );
}

#[test]
fn test_stack_detection_and_custom_stacks() {
    let temp = TempDir::new("scaffold_stacks");
    let ws = temp.path();

    // Create a python indicator file
    fs::write(ws.join("pyproject.toml"), "[tool.poetry]").unwrap();

    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: None, // Auto-detect
        force: false,
        dry_run: false,
        project_name: None,
    };

    let result = run_scaffold(&options).expect("auto-detect scaffold should succeed");
    assert_eq!(result.stack, "python");

    // Check that gauntlet.toml contains python layers (ruff, pyright, pytest)
    let config_content = fs::read_to_string(ws.join("gauntlet.toml")).unwrap();
    assert!(config_content.contains("stack = \"python\""));
    assert!(config_content.contains("ruff"));
    assert!(config_content.contains("pytest"));
}

#[test]
fn test_invalid_stack_rejected() {
    let temp = TempDir::new("scaffold_invalid_stack");
    let ws = temp.path();

    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: Some("unsupported-lang-42".to_string()),
        force: false,
        dry_run: false,
        project_name: None,
    };

    let err = run_scaffold(&options).unwrap_err();
    match err {
        ScaffoldError::InvalidStack(name) => {
            assert_eq!(name, "unsupported-lang-42");
        }
        _ => panic!("Expected ScaffoldError::InvalidStack, got {err:?}"),
    }
}

#[test]
fn test_generated_templates_pass_check_spec_and_config_validation() {
    let temp = TempDir::new("scaffold_governance_validity");
    let ws = temp.path();

    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: Some("rust".to_string()),
        force: false,
        dry_run: false,
        project_name: Some("my-verified-app".to_string()),
    };

    let result = run_scaffold(&options).expect("scaffold should succeed");
    assert!(result.is_success);

    // 1. Validate CONTEXT.md format (Aristotle definitio per genus et differentiam + _Avoid_:)
    let context_diags = validate_context_glossary(ws);
    assert!(
        context_diags.is_empty(),
        "Generated CONTEXT.md must adhere 100% to Aristotle glossary rules: {context_diags:?}"
    );

    // 2. Validate tasks/001-bootstrap.md with check_task_specification
    let task_path = ws.join("tasks/001-bootstrap.md");
    let report = check_task_specification(&task_path, ws);
    assert!(
        report.is_valid,
        "Generated task package must pass spec readiness: {:?}",
        report.diagnostics
    );
    assert!(!report.acceptance_criteria.is_empty());
    assert!(!report.must_not_rules.is_empty());

    // 3. Validate gauntlet.toml with validate_config
    let config = GauntletConfig::load(ws).expect("gauntlet.toml should load cleanly");
    let config_report = validate_config(&config, Some(ws));
    assert!(
        config_report.is_valid,
        "Generated gauntlet.toml must be valid: {:?}",
        config_report.issues
    );

    // 4. Validate hooks.json structure
    let hooks_content = fs::read_to_string(ws.join(".agents/hooks.json")).unwrap();
    let hooks_val: serde_json::Value = serde_json::from_str(&hooks_content).unwrap();
    assert!(hooks_val.get("agent-gauntlet-gatekeeper").is_some());
}

#[test]
fn test_all_supported_stacks_generate_valid_configs() {
    let stacks = ["rust", "python", "node", "typescript", "javascript", "go"];

    for stack in stacks {
        let temp = TempDir::new(&format!("scaffold_stack_{stack}"));
        let ws = temp.path();

        let options = ScaffoldOptions {
            workspace: ws.to_path_buf(),
            stack: Some(stack.to_string()),
            force: false,
            dry_run: false,
            project_name: Some(format!("test-{stack}")),
        };

        let res =
            run_scaffold(&options).unwrap_or_else(|e| panic!("Failed for stack {stack}: {e}"));
        assert!(res.is_success);
        assert_eq!(res.created_count, 9);

        // Validate gauntlet.toml
        let config = GauntletConfig::load(ws).unwrap();
        let report = validate_config(&config, Some(ws));
        assert!(
            report.is_valid,
            "Config for stack '{stack}' must be valid: {:?}",
            report.issues
        );
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

#[test]
fn test_cli_init_integration() {
    let bin_path = find_xgauntlet_binary();
    if !bin_path.is_file() {
        return;
    }

    let temp = TempDir::new("scaffold_cli_test");
    let ws = temp.path();

    // 1. Run dry-run via CLI
    let output = std::process::Command::new(&bin_path)
        .args([
            "init",
            "--workspace",
            ws.to_str().unwrap(),
            "--stack",
            "node",
            "--dry-run",
        ])
        .output()
        .expect("failed to execute xgauntlet init --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[DRY RUN]"));
    assert!(stdout.contains("gauntlet.toml"));
    assert!(stdout.contains("9 would create"));

    // Verify no files on disk
    let count = fs::read_dir(ws).unwrap().count();
    assert_eq!(count, 0);

    // 2. Run actual init with --json
    let output_json = std::process::Command::new(&bin_path)
        .args([
            "init",
            "--workspace",
            ws.to_str().unwrap(),
            "--stack",
            "node",
            "--json",
        ])
        .output()
        .expect("failed to execute xgauntlet init --json");

    assert!(output_json.status.success());
    let stdout_json = String::from_utf8_lossy(&output_json.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout_json).expect("valid json output");
    assert_eq!(parsed["is_success"], true);
    assert_eq!(parsed["created_count"], 9);
    assert_eq!(parsed["skipped_count"], 0);

    // Verify all files created on disk
    assert!(ws.join("gauntlet.toml").is_file());
    assert!(ws.join("spec.md").is_file());
    assert!(ws.join("CONTEXT.md").is_file());
    assert!(ws.join("tasks/001-bootstrap.md").is_file());

    // 3. Re-run without --force: verify non-destructive guarantee (skipped = 9)
    let output_skip = std::process::Command::new(&bin_path)
        .args(["init", "--workspace", ws.to_str().unwrap(), "--json"])
        .output()
        .expect("failed to re-run xgauntlet init");

    assert!(output_skip.status.success());
    let stdout_skip = String::from_utf8_lossy(&output_skip.stdout);
    let parsed_skip: serde_json::Value = serde_json::from_str(&stdout_skip).unwrap();
    assert_eq!(parsed_skip["created_count"], 0);
    assert_eq!(parsed_skip["skipped_count"], 9);

    // 4. Re-run with --force: verify overwrite (overwritten = 9)
    let output_force = std::process::Command::new(&bin_path)
        .args([
            "init",
            "--workspace",
            ws.to_str().unwrap(),
            "--force",
            "--json",
        ])
        .output()
        .expect("failed to force re-run xgauntlet init");

    assert!(output_force.status.success());
    let stdout_force = String::from_utf8_lossy(&output_force.stdout);
    let parsed_force: serde_json::Value = serde_json::from_str(&stdout_force).unwrap();
    assert_eq!(parsed_force["overwritten_count"], 9);
}

#[test]
fn test_scaffold_agents_md_contains_dynamic_cockpit_hud_and_checkpoint_protocol() {
    let temp = TempDir::new("scaffold_cockpit_hud");
    let ws = temp.path();

    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: Some("rust".to_string()),
        force: false,
        dry_run: false,
        project_name: Some("test-cockpit".to_string()),
    };

    let result = run_scaffold(&options).expect("scaffold should succeed");
    assert!(result.is_success);

    let agents_md = fs::read_to_string(ws.join(".agents/AGENTS.md")).unwrap();

    // 1. Cockpit Response HUD telemetry sections
    assert!(
        agents_md.contains("Git:"),
        "agents.md should contain Git telemetry in HUD"
    );
    assert!(
        agents_md.contains("Criteria:"),
        "agents.md should contain Criteria progress in HUD"
    );
    assert!(
        agents_md.contains("Scope:"),
        "agents.md should contain Scope boundary in HUD"
    );
    assert!(
        agents_md.contains("Next Action:"),
        "agents.md should contain Next Action line in HUD"
    );
    assert!(
        agents_md.contains("[Task](tasks/)")
            && agents_md.contains("[Spec](spec.md)")
            && agents_md.contains("[Glossary](CONTEXT.md)")
            && agents_md.contains("[ADR](docs/adr/README.md)")
            && agents_md.contains("[Evidence](evidence.md)"),
        "agents.md must preserve all five central navigation links"
    );

    // 2. Local TDD Phase Checkpoint Protocol (ADR 0003)
    assert!(
        agents_md.contains("Phase Checkpoint"),
        "agents.md must specify Phase Checkpoint protocol"
    );
    assert!(
        agents_md.contains("SPEC") && agents_md.contains("task("),
        "agents.md must include SPEC checkpoint conventional commit format"
    );
    assert!(
        agents_md.contains("RED") && agents_md.contains("test("),
        "agents.md must include RED checkpoint conventional commit format"
    );
    assert!(
        agents_md.contains("GREEN") && agents_md.contains("feat("),
        "agents.md must include GREEN checkpoint conventional commit format"
    );
    assert!(
        agents_md.contains("REFACTOR") && agents_md.contains("refactor("),
        "agents.md must include REFACTOR checkpoint conventional commit format"
    );
    assert!(
        agents_md.contains("DONE") && agents_md.contains("chore("),
        "agents.md must include DONE checkpoint conventional commit format"
    );
    assert!(
        agents_md.contains("git push"),
        "agents.md must explicitly forbid git push jf. ADR 0003"
    );

    // 3. 4-step intent sparring in idea phase
    assert!(
        agents_md.contains("Intent-to-Task") || agents_md.contains("Sparring"),
        "agents.md must include structured intent sparring procedure"
    );
    assert!(
        agents_md.contains("Must NOT") || agents_md.contains("Invarianter"),
        "agents.md must include Must NOT / invariant formulation in sparring"
    );
}
