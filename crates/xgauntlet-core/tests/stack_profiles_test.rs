//! Integration tests for stack profiling and composite TypeScript project references detection (Task 029).

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::config::{
    detect_composite_typescript, is_composite_typescript_tsconfig, parse_toml,
};
use xgauntlet_core::features::scaffold::{run_scaffold, ScaffoldOptions};

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
        let path = std::env::temp_dir().join(format!("xgauntlet_stack_test_{}", unique));
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
fn test_detect_composite_typescript_tsconfig_content() {
    // 1. Classic Vite/Monorepo composite root tsconfig
    let composite_json = r#"{
      "files": [],
      "references": [
        { "path": "./tsconfig.app.json" },
        { "path": "./tsconfig.node.json" }
      ]
    }"#;
    assert!(
        is_composite_typescript_tsconfig(composite_json),
        "Expected composite tsconfig with empty files and references to be detected"
    );

    // 2. Composite with comments and trailing formatting
    let with_comments = r#"{
      // Project references root container
      "files": [],
      /* Sub-projects */
      "references": [
        { "path": "./packages/core" }
      ]
    }"#;
    assert!(
        is_composite_typescript_tsconfig(with_comments),
        "Expected composite tsconfig with comments to be parsed and detected"
    );

    // 3. Composite with no 'files' field declared at all (only references)
    let no_files = r#"{
      "references": [
        { "path": "./tsconfig.app.json" }
      ]
    }"#;
    assert!(
        is_composite_typescript_tsconfig(no_files),
        "Expected composite tsconfig without files field to be detected"
    );

    // 4. Standard flat tsconfig (no references)
    let flat_json = r#"{
      "compilerOptions": {
        "target": "ESNext",
        "module": "ESNext"
      },
      "include": ["src/**/*"]
    }"#;
    assert!(
        !is_composite_typescript_tsconfig(flat_json),
        "Standard flat tsconfig must not be detected as composite"
    );

    // 5. Empty references array
    let empty_refs = r#"{
      "files": [],
      "references": []
    }"#;
    assert!(
        !is_composite_typescript_tsconfig(empty_refs),
        "Empty references array must not be detected as composite"
    );

    // 6. Has references but also includes root source files directly
    let with_source_files = r#"{
      "files": ["src/main.ts"],
      "references": [
        { "path": "./tsconfig.node.json" }
      ]
    }"#;
    assert!(
        !is_composite_typescript_tsconfig(with_source_files),
        "Tsconfig with direct source files must not be treated as composite container"
    );

    // 7. Has references but also includes source include pattern
    let with_include = r#"{
      "include": ["src/**/*"],
      "references": [
        { "path": "./tsconfig.node.json" }
      ]
    }"#;
    assert!(
        !is_composite_typescript_tsconfig(with_include),
        "Tsconfig with direct include patterns must not be treated as composite container"
    );
}

#[test]
fn test_detect_composite_typescript_in_workspace() {
    let temp = TempDir::new("composite_ws");
    let ws = temp.path();

    // No tsconfig -> false
    assert!(!detect_composite_typescript(ws));

    // Standard flat tsconfig -> false
    let flat_content = r#"{"include": ["src/**/*"]}"#;
    fs::write(ws.join("tsconfig.json"), flat_content).unwrap();
    assert!(!detect_composite_typescript(ws));

    // Composite tsconfig -> true
    let composite_content = r#"{"files": [], "references": [{"path": "./tsconfig.app.json"}]}"#;
    fs::write(ws.join("tsconfig.json"), composite_content).unwrap();
    assert!(detect_composite_typescript(ws));
}

#[test]
fn test_scaffold_init_composite_typescript_generates_tsc_b() {
    let temp = TempDir::new("scaffold_composite");
    let ws = temp.path();

    // Pre-seed composite tsconfig.json
    let composite_content = r#"{
      "files": [],
      "references": [
        { "path": "./tsconfig.app.json" },
        { "path": "./tsconfig.node.json" }
      ]
    }"#;
    fs::write(ws.join("tsconfig.json"), composite_content).unwrap();

    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: Some("node".to_string()),
        force: false,
        dry_run: false,
        project_name: Some("composite-vite-app".to_string()),
    };

    let result = run_scaffold(&options).expect("Scaffold must succeed");
    assert!(result.is_success);

    let gauntlet_toml_path = ws.join("gauntlet.toml");
    assert!(gauntlet_toml_path.is_file(), "gauntlet.toml must be created");

    let content = fs::read_to_string(&gauntlet_toml_path).unwrap();
    let config = parse_toml(&content).expect("gauntlet.toml must be valid TOML");

    let types_layer = config
        .layers
        .iter()
        .find(|l| l.name == "types")
        .expect("types layer must exist");

    assert_eq!(
        types_layer.command,
        vec!["npx".to_string(), "tsc".to_string(), "-b".to_string()],
        "Types layer must use 'tsc -b' for composite TypeScript setups"
    );
}

#[test]
fn test_scaffold_init_standard_typescript_generates_tsc_no_emit() {
    let temp = TempDir::new("scaffold_standard");
    let ws = temp.path();

    // Pre-seed standard flat tsconfig.json
    let flat_content = r#"{
      "compilerOptions": { "strict": true },
      "include": ["src/**/*"]
    }"#;
    fs::write(ws.join("tsconfig.json"), flat_content).unwrap();

    let options = ScaffoldOptions {
        workspace: ws.to_path_buf(),
        stack: Some("node".to_string()),
        force: false,
        dry_run: false,
        project_name: Some("standard-ts-app".to_string()),
    };

    let result = run_scaffold(&options).expect("Scaffold must succeed");
    assert!(result.is_success);

    let content = fs::read_to_string(ws.join("gauntlet.toml")).unwrap();
    let config = parse_toml(&content).expect("gauntlet.toml must be valid TOML");

    let types_layer = config
        .layers
        .iter()
        .find(|l| l.name == "types")
        .expect("types layer must exist");

    assert_eq!(
        types_layer.command,
        vec![
            "npx".to_string(),
            "tsc".to_string(),
            "--noEmit".to_string()
        ],
        "Standard flat TypeScript setups must retain 'tsc --noEmit'"
    );
}
