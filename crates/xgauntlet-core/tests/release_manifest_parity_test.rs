use std::fs;
use std::path::{Path, PathBuf};
use xgauntlet_core::features::release::{extract_declared_versions, update_manifest_versions};

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
        let path = std::env::temp_dir().join(format!("xgauntlet_parity_test_{}", unique));
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

fn create_temp_workspace() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new("parity_ws");
    let path = temp_dir.path().to_path_buf();
    (temp_dir, path)
}

#[test]
fn test_extract_declared_versions_includes_plugin_and_policy() {
    let (_guard, ws) = create_temp_workspace();

    // 1. Cargo.toml
    fs::write(
        ws.join("Cargo.toml"),
        r#"[workspace.package]
version = "0.12.0"
"#,
    )
    .unwrap();

    // 2. .agents/plugins/xgauntlet/plugin.json
    fs::create_dir_all(ws.join(".agents/plugins/xgauntlet")).unwrap();
    fs::write(
        ws.join(".agents/plugins/xgauntlet/plugin.json"),
        r#"{
  "name": "xgauntlet",
  "version": "0.12.0",
  "description": "Verification engine"
}
"#,
    )
    .unwrap();

    // 3. .agents/plugins/xgauntlet/policy.json
    fs::write(
        ws.join(".agents/plugins/xgauntlet/policy.json"),
        r#"{
  "project_name": "xgauntlet",
  "version": "0.12.0"
}
"#,
    )
    .unwrap();

    let summary = extract_declared_versions(&ws).expect("extraction should succeed");

    assert!(
        summary
            .inspected_files
            .contains(&".agents/plugins/xgauntlet/plugin.json".to_string()),
        "inspected_files should include .agents/plugins/xgauntlet/plugin.json, got: {:?}",
        summary.inspected_files
    );
    assert_eq!(
        summary
            .versions_by_source
            .get(".agents/plugins/xgauntlet/plugin.json"),
        Some(&"0.12.0".to_string())
    );

    assert!(
        summary
            .inspected_files
            .contains(&".agents/plugins/xgauntlet/policy.json".to_string()),
        "inspected_files should include .agents/plugins/xgauntlet/policy.json, got: {:?}",
        summary.inspected_files
    );
    assert_eq!(
        summary
            .versions_by_source
            .get(".agents/plugins/xgauntlet/policy.json"),
        Some(&"0.12.0".to_string())
    );
}

#[test]
fn test_update_manifest_versions_bumps_plugin_and_policy() {
    let (_guard, ws) = create_temp_workspace();

    // 1. Cargo.toml
    fs::write(
        ws.join("Cargo.toml"),
        r#"[workspace.package]
version = "0.12.0"
"#,
    )
    .unwrap();

    // 2. .agents/plugins/xgauntlet/plugin.json
    fs::create_dir_all(ws.join(".agents/plugins/xgauntlet")).unwrap();
    fs::write(
        ws.join(".agents/plugins/xgauntlet/plugin.json"),
        r#"{
  "name": "xgauntlet",
  "version": "0.12.0",
  "skills": ["tdd"]
}
"#,
    )
    .unwrap();

    // 3. .agents/plugins/xgauntlet/policy.json
    fs::write(
        ws.join(".agents/plugins/xgauntlet/policy.json"),
        r#"{
  "project_name": "xgauntlet",
  "version": "0.12.0"
}
"#,
    )
    .unwrap();

    let modified = update_manifest_versions(&ws, "0.13.0").expect("update should succeed");

    assert!(
        modified.contains(&".agents/plugins/xgauntlet/plugin.json".to_string()),
        "modified should include .agents/plugins/xgauntlet/plugin.json, got: {:?}",
        modified
    );
    assert!(
        modified.contains(&".agents/plugins/xgauntlet/policy.json".to_string()),
        "modified should include .agents/plugins/xgauntlet/policy.json, got: {:?}",
        modified
    );

    let updated_plugin: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ws.join(".agents/plugins/xgauntlet/plugin.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(updated_plugin["version"].as_str(), Some("0.13.0"));

    let updated_policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ws.join(".agents/plugins/xgauntlet/policy.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(updated_policy["version"].as_str(), Some("0.13.0"));
}

#[test]
fn test_workspace_manifests_synchronized_to_version_0_13_2() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let summary = extract_declared_versions(repo_root).expect("extract declared versions");
    for (file, ver) in &summary.versions_by_source {
        assert_eq!(
            ver, "0.13.2",
            "Manifest '{file}' must be synchronized to release version 0.13.2"
        );
    }
}

#[test]
fn test_npm_cli_readme_identical_to_root_readme() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let root_readme_path = repo_root.join("README.md");
    let cli_readme_path = repo_root.join("packages/cli/README.md");

    assert!(root_readme_path.is_file(), "root README.md must exist");
    assert!(
        cli_readme_path.is_file(),
        "packages/cli/README.md must exist"
    );

    let root_content = fs::read_to_string(&root_readme_path).expect("read root README.md");
    let cli_content = fs::read_to_string(&cli_readme_path).expect("read packages/cli/README.md");

    assert_eq!(
        root_content, cli_content,
        "packages/cli/README.md must be byte-for-byte identical to root README.md to avoid npm registry drift"
    );
}
