use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::config::{
    detect_stack, get_stack_profile, list_supported_stacks, validate_config, ConfigError,
    ConfigValidationReport, GauntletConfig, LayerConfig, PathsConfig, StackProfile,
    ValidationSeverity,
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
fn test_parse_full_valid_gauntlet_toml() {
    let toml = r#"
# Core configuration
stack = "rust"
save_evidence = true
evidence_file = "custom-evidence.json"
evidence_markdown_file = "custom-evidence.md"

[paths]
tasks_dir = "my-tasks"
spec_file = "my-spec.md"
context_file = "my-context.md"
coding_standards_file = "my-standards.md"

[[layers]]
name = "lint"
command = ["cargo", "clippy", "--", "-D", "warnings"]
optional = true
timeout_seconds = 45.0

[[layers]]
name = "unit"
command = ["cargo", "test", "--bin", "my_app"]
optional = false
timeout_seconds = 90.0
"#;

    let config = GauntletConfig::parse_toml(toml).expect("Failed to parse valid gauntlet.toml");
    assert_eq!(config.stack, "rust");
    assert!(config.save_evidence);
    assert_eq!(config.evidence_file, "custom-evidence.json");
    assert_eq!(config.evidence_markdown_file, "custom-evidence.md");

    assert_eq!(config.paths.tasks_dir, "my-tasks");
    assert_eq!(config.paths.spec_file, "my-spec.md");
    assert_eq!(config.paths.context_file, "my-context.md");
    assert_eq!(config.paths.coding_standards_file, "my-standards.md");

    assert_eq!(config.layers.len(), 2);
    assert_eq!(config.layers[0].name, "lint");
    assert_eq!(
        config.layers[0].command,
        vec!["cargo", "clippy", "--", "-D", "warnings"]
    );
    assert!(config.layers[0].optional);
    assert_eq!(config.layers[0].timeout_seconds, 45.0);

    assert_eq!(config.layers[1].name, "unit");
    assert_eq!(
        config.layers[1].command,
        vec!["cargo", "test", "--bin", "my_app"]
    );
    assert!(!config.layers[1].optional);
    assert_eq!(config.layers[1].timeout_seconds, 90.0);

    let defs = config.to_layer_definitions();
    assert_eq!(defs.len(), 2);
    assert_eq!(defs[0].name, "lint");
    assert_eq!(defs[1].name, "unit");
}

#[test]
fn test_parse_valid_gauntlet_json() {
    let json = r#"{
  "stack": "python",
  "save_evidence": false,
  "evidence_file": "report.json",
  "evidence_markdown_file": "report.md",
  "paths": {
    "tasks_dir": "tasks",
    "spec_file": "spec.md",
    "context_file": "CONTEXT.md",
    "coding_standards_file": "CODING_STANDARDS.md"
  },
  "layers": [
    {
      "name": "lint",
      "command": ["ruff", "check"],
      "optional": true,
      "timeout_seconds": 30.0
    },
    {
      "name": "unit",
      "command": ["pytest"],
      "optional": false,
      "timeout_seconds": 120.0
    }
  ]
}"#;

    let config = GauntletConfig::parse_json(json).expect("Failed to parse gauntlet.json");
    assert_eq!(config.stack, "python");
    assert!(!config.save_evidence);
    assert_eq!(config.layers.len(), 2);
    assert_eq!(config.layers[0].name, "lint");
    assert_eq!(config.layers[1].name, "unit");
}

#[test]
fn test_stack_profiles_registry_and_presets() {
    let stacks = list_supported_stacks();
    assert!(stacks.contains(&"rust"));
    assert!(stacks.contains(&"python"));
    assert!(stacks.contains(&"node"));
    assert!(stacks.contains(&"typescript"));
    assert!(stacks.contains(&"go"));

    let rust_profile: StackProfile = get_stack_profile("rust").expect("Rust profile missing");
    assert_eq!(rust_profile.name, "rust");
    assert!(rust_profile.default_layers.iter().any(|l| l.name == "lint"));
    assert!(rust_profile.default_layers.iter().any(|l| l.name == "unit"));
    assert!(rust_profile
        .default_layers
        .iter()
        .any(|l| l.name == "invariants"));

    let py_profile = get_stack_profile("python").expect("Python profile missing");
    assert_eq!(py_profile.name, "python");
    assert!(py_profile.default_layers.iter().any(|l| l.name == "lint"));
    assert!(py_profile.default_layers.iter().any(|l| l.name == "types"));
    assert!(py_profile.default_layers.iter().any(|l| l.name == "unit"));

    let go_profile = get_stack_profile("go").expect("Go profile missing");
    assert_eq!(go_profile.name, "go");
    assert!(go_profile.default_layers.iter().any(|l| l.name == "lint"));
    assert!(go_profile.default_layers.iter().any(|l| l.name == "unit"));
}

#[test]
fn test_detect_stack_from_workspace() {
    let tmp = TempDir::new("detect_stack");
    let base = tmp.path();

    let rust_dir = base.join("rust_proj");
    fs::create_dir_all(&rust_dir).unwrap();
    fs::write(rust_dir.join("Cargo.toml"), "[package]").unwrap();
    assert_eq!(detect_stack(&rust_dir), "rust");

    let py_dir = base.join("py_proj");
    fs::create_dir_all(&py_dir).unwrap();
    fs::write(py_dir.join("pyproject.toml"), "[project]").unwrap();
    assert_eq!(detect_stack(&py_dir), "python");

    let node_dir = base.join("node_proj");
    fs::create_dir_all(&node_dir).unwrap();
    fs::write(node_dir.join("package.json"), "{}").unwrap();
    assert_eq!(detect_stack(&node_dir), "node");

    let go_dir = base.join("go_proj");
    fs::create_dir_all(&go_dir).unwrap();
    fs::write(go_dir.join("go.mod"), "module example.com/app").unwrap();
    assert_eq!(detect_stack(&go_dir), "go");
}

#[test]
fn test_validation_engine_catches_schema_defects() {
    // 1. Valid configuration passes
    let valid_config = GauntletConfig::default_for_stack("rust");
    let report: ConfigValidationReport = validate_config(&valid_config, None);
    assert!(report.is_valid);
    assert!(report.issues.is_empty());

    // 2. Duplicate layer names
    let mut dup_config = valid_config.clone();
    dup_config.layers.push(LayerConfig {
        name: "lint".to_string(),
        command: vec!["echo".to_string(), "duplicate".to_string()],
        optional: true,
        timeout_seconds: 30.0,
    });
    let report = validate_config(&dup_config, None);
    assert!(!report.is_valid);
    let dup_issue = report
        .issues
        .iter()
        .find(|i| i.code == "DUPLICATE_LAYER_NAME")
        .expect("Missing DUPLICATE_LAYER_NAME issue");
    assert_eq!(dup_issue.severity, ValidationSeverity::Error);
    assert!(dup_issue.remediation.is_some());

    // 3. Empty command
    let mut empty_cmd_config = valid_config.clone();
    empty_cmd_config.layers.push(LayerConfig {
        name: "empty_cmd".to_string(),
        command: vec![],
        optional: false,
        timeout_seconds: 30.0,
    });
    let report = validate_config(&empty_cmd_config, None);
    assert!(!report.is_valid);
    assert!(report
        .issues
        .iter()
        .any(|i| i.code == "EMPTY_LAYER_COMMAND"));

    // 4. Invalid timeout (<= 0 or > 3600)
    let mut bad_timeout_config = valid_config.clone();
    bad_timeout_config.layers.push(LayerConfig {
        name: "bad_timeout".to_string(),
        command: vec!["cargo".to_string(), "test".to_string()],
        optional: false,
        timeout_seconds: -10.0,
    });
    let report = validate_config(&bad_timeout_config, None);
    assert!(!report.is_valid);
    assert!(report.issues.iter().any(|i| i.code == "INVALID_TIMEOUT"));

    // 5. Path traversal in paths configuration
    let mut traversal_config = valid_config.clone();
    traversal_config.paths = PathsConfig {
        tasks_dir: "../escape_tasks".to_string(),
        ..PathsConfig::default()
    };
    let report = validate_config(&traversal_config, None);
    assert!(!report.is_valid);
    assert!(report.issues.iter().any(|i| i.code == "PATH_TRAVERSAL"));
}

#[test]
fn test_toml_rendering_and_roundtrip() {
    let original = GauntletConfig::default_for_stack("rust");
    let rendered = original.render_toml();
    let parsed = GauntletConfig::parse_toml(&rendered).expect("Roundtrip parse failed");

    assert_eq!(original.stack, parsed.stack);
    assert_eq!(original.save_evidence, parsed.save_evidence);
    assert_eq!(original.paths.tasks_dir, parsed.paths.tasks_dir);
    assert_eq!(original.layers.len(), parsed.layers.len());
    for (orig_l, parsed_l) in original.layers.iter().zip(parsed.layers.iter()) {
        assert_eq!(orig_l.name, parsed_l.name);
        assert_eq!(orig_l.command, parsed_l.command);
        assert_eq!(orig_l.optional, parsed_l.optional);
        assert_eq!(orig_l.timeout_seconds, parsed_l.timeout_seconds);
    }
}

#[test]
fn test_load_config_from_workspace_file_or_fallback() {
    let tmp = TempDir::new("load_config");
    let dir = tmp.path();

    // 1. Fallback to auto-detected rust stack
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    let cfg = GauntletConfig::load(dir).expect("Failed to load auto-detected config");
    assert_eq!(cfg.stack, "rust");
    assert!(!cfg.layers.is_empty());

    // 2. Load explicit gauntlet.toml
    let custom_toml = r#"
stack = "custom"
save_evidence = false

[[layers]]
name = "custom-layer"
command = ["custom-tool", "run"]
optional = false
timeout_seconds = 15.0
"#;
    fs::write(dir.join("gauntlet.toml"), custom_toml).unwrap();
    let loaded = GauntletConfig::load(dir).expect("Failed to load gauntlet.toml");
    assert_eq!(loaded.stack, "custom");
    assert!(!loaded.save_evidence);
    assert_eq!(loaded.layers.len(), 1);
    assert_eq!(loaded.layers[0].name, "custom-layer");
}

#[test]
fn test_malformed_syntax_errors() {
    let bad_toml = "stack = \"rust\"\nthis is not valid toml\n";
    let err = GauntletConfig::parse_toml(bad_toml).expect_err("Should fail on invalid syntax");
    match err {
        ConfigError::TomlError(msg) => assert!(msg.contains("Syntax error")),
        _ => panic!("Expected TomlError, got: {:?}", err),
    }

    let bad_json = "{\"stack\": \"rust\",}";
    let err = GauntletConfig::parse_json(bad_json).expect_err("Should fail on invalid JSON");
    match err {
        ConfigError::JsonError(_) => {}
        _ => panic!("Expected JsonError, got: {:?}", err),
    }
}

#[test]
fn test_real_workspace_gauntlet_toml_passes_validation() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let gauntlet_path = workspace.join("gauntlet.toml");
    if gauntlet_path.is_file() {
        let config = GauntletConfig::load(workspace).expect("Failed to load root gauntlet.toml");
        assert_eq!(config.stack, "rust");
        assert!(!config.layers.is_empty());

        let report = validate_config(&config, Some(workspace));
        assert!(
            report.is_valid,
            "Real workspace gauntlet.toml failed validation: {:?}",
            report.issues
        );
    }
}
