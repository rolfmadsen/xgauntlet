use std::fs;
use std::path::{Path, PathBuf};
use xgauntlet_core::features::release::{
    check_release_readiness, parse_changelog_versions, update_manifest_versions,
    ReleaseFindingCategory, ReleaseReadinessOptions,
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
        let path = std::env::temp_dir().join(format!("xgauntlet_rel_test_{}", unique));
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
    let temp_dir = TempDir::new("release_ws");
    let path = temp_dir.path().to_path_buf();
    (temp_dir, path)
}

#[test]
fn test_manifest_version_extraction_and_harmony() {
    let (_guard, ws) = create_temp_workspace();

    // 1. Cargo.toml (workspace)
    fs::write(
        ws.join("Cargo.toml"),
        r#"
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
version = "1.2.3"
edition = "2021"
"#,
    )
    .unwrap();

    // 2. Member crate with workspace inheritance
    fs::create_dir_all(ws.join("crates/member-a")).unwrap();
    fs::write(
        ws.join("crates/member-a/Cargo.toml"),
        r#"
[package]
name = "member-a"
version.workspace = true
"#,
    )
    .unwrap();

    // 3. Member crate with explicit matching version
    fs::create_dir_all(ws.join("crates/member-b")).unwrap();
    fs::write(
        ws.join("crates/member-b/Cargo.toml"),
        r#"
[package]
name = "member-b"
version = "1.2.3"
"#,
    )
    .unwrap();

    // 4. Root package.json
    fs::write(
        ws.join("package.json"),
        r#"{
  "name": "root-pkg",
  "version": "1.2.3"
}"#,
    )
    .unwrap();

    // 5. Subpackage packages/cli/package.json
    fs::create_dir_all(ws.join("packages/cli")).unwrap();
    fs::write(
        ws.join("packages/cli/package.json"),
        r#"{
  "name": "@scope/cli",
  "version": "1.2.3"
}"#,
    )
    .unwrap();

    // 6. pyproject.toml
    fs::write(
        ws.join("pyproject.toml"),
        r#"
[project]
name = "python-pkg"
version = "1.2.3"
"#,
    )
    .unwrap();

    // 7. CHANGELOG.md matching version
    fs::write(
        ws.join("CHANGELOG.md"),
        r#"# Changelog
## [1.2.3] - 2026-09-06
### Added
- Feature x
"#,
    )
    .unwrap();

    // 8. Minimal docs and ADR
    fs::create_dir_all(ws.join("docs/adr")).unwrap();
    fs::write(
        ws.join("docs/adr/0001-init.md"),
        "# 1. Initial Architecture\n",
    )
    .unwrap();
    fs::write(
        ws.join("README.md"),
        "# Test Workspace\nSee [ADR 0001](docs/adr/0001-init.md).\n",
    )
    .unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).expect("release readiness evaluation failed");

    assert!(report.is_ready, "Expected workspace to be release ready");
    assert_eq!(report.declared_version, "1.2.3");
    assert_eq!(report.diagnostics.len(), 0);
    assert_eq!(
        report.versions_by_source.get("Cargo.toml").unwrap(),
        "1.2.3"
    );
    assert_eq!(
        report.versions_by_source.get("package.json").unwrap(),
        "1.2.3"
    );
    assert_eq!(
        report
            .versions_by_source
            .get("packages/cli/package.json")
            .unwrap(),
        "1.2.3"
    );
    assert_eq!(
        report.versions_by_source.get("pyproject.toml").unwrap(),
        "1.2.3"
    );
}

#[test]
fn test_manifest_version_mismatch_detection() {
    let (_guard, ws) = create_temp_workspace();

    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "mismatch-crate"
version = "1.0.0"
"#,
    )
    .unwrap();

    fs::write(
        ws.join("package.json"),
        r#"{
  "name": "mismatch-pkg",
  "version": "2.0.0"
}"#,
    )
    .unwrap();

    fs::write(
        ws.join("CHANGELOG.md"),
        "## [1.0.0] - 2026-09-06\n## [2.0.0] - 2026-09-06\n",
    )
    .unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).unwrap();
    assert!(!report.is_ready);
    let has_mismatch = report
        .diagnostics
        .iter()
        .any(|d| d.category == ReleaseFindingCategory::ConfigVersionMismatch);
    assert!(
        has_mismatch,
        "Expected CONFIG_VERSION_MISMATCH diagnostic when manifests disagree"
    );
}

#[test]
fn test_missing_changelog_detection() {
    let (_guard, ws) = create_temp_workspace();

    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "test-pkg"
version = "0.5.0"
"#,
    )
    .unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).unwrap();
    assert!(!report.is_ready);
    let has_missing = report
        .diagnostics
        .iter()
        .any(|d| d.category == ReleaseFindingCategory::MissingChangelog);
    assert!(
        has_missing,
        "Expected MISSING_CHANGELOG diagnostic when CHANGELOG.md is absent"
    );
}

#[test]
fn test_changelog_version_matching_and_allow_unreleased() {
    let (_guard, ws) = create_temp_workspace();

    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "test-pkg"
version = "0.9.0"
"#,
    )
    .unwrap();

    // 1. CHANGELOG only has older version
    fs::write(
        ws.join("CHANGELOG.md"),
        r#"# Changelog
## [0.8.0] - 2026-08-01
- Older release
"#,
    )
    .unwrap();

    let options_strict = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };
    let report1 = check_release_readiness(&options_strict).unwrap();
    assert!(!report1.is_ready);
    assert!(report1
        .diagnostics
        .iter()
        .any(|d| d.category == ReleaseFindingCategory::ChangelogVersionMismatch));

    // 2. CHANGELOG has [Unreleased] section
    fs::write(
        ws.join("CHANGELOG.md"),
        r#"# Changelog
## [Unreleased]
- Ongoing work for 0.9.0
## [0.8.0] - 2026-08-01
"#,
    )
    .unwrap();

    // With allow_unreleased = false -> should still fail
    let report2 = check_release_readiness(&options_strict).unwrap();
    assert!(!report2.is_ready);

    // With allow_unreleased = true -> allowed advisory mode
    let options_allow = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: true,
        strict: false,
    };
    let report3 = check_release_readiness(&options_allow).unwrap();
    assert!(report3.is_ready);

    // 3. CHANGELOG has exact match [0.9.0]
    fs::write(
        ws.join("CHANGELOG.md"),
        r#"# Changelog
## [0.9.0] - 2026-09-06
- Release 0.9.0
"#,
    )
    .unwrap();

    let report4 = check_release_readiness(&options_strict).unwrap();
    assert!(report4.is_ready);
}

#[test]
fn test_adr_coverage_detection() {
    let (_guard, ws) = create_temp_workspace();

    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "adr-test"
version = "1.0.0"
"#,
    )
    .unwrap();

    fs::write(ws.join("CHANGELOG.md"), "## [1.0.0] - 2026-09-06\n").unwrap();

    fs::create_dir_all(ws.join("docs/adr")).unwrap();
    fs::write(
        ws.join("docs/adr/0001-package-by-feature.md"),
        "# ADR 1\nContent\n",
    )
    .unwrap();
    fs::write(
        ws.join("docs/adr/0002-unreferenced-decision.md"),
        "# ADR 2\nContent\n",
    )
    .unwrap();
    fs::write(ws.join("docs/adr/README.md"), "# ADR Index\nIgnored file\n").unwrap();

    // README only references 0001
    fs::write(
        ws.join("README.md"),
        "# Workspace\nSee [ADR 0001](docs/adr/0001-package-by-feature.md)\n",
    )
    .unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).unwrap();
    assert!(!report.is_ready);
    assert_eq!(report.unreferenced_adrs.len(), 1);
    assert_eq!(
        report.unreferenced_adrs[0],
        "docs/adr/0002-unreferenced-decision.md"
    );

    let adr_diag = report
        .diagnostics
        .iter()
        .find(|d| d.category == ReleaseFindingCategory::UnreferencedAdr);
    assert!(adr_diag.is_some());
    assert!(adr_diag
        .unwrap()
        .message
        .contains("0002-unreferenced-decision.md"));

    // Now reference 0002 in spec.md
    fs::write(
        ws.join("spec.md"),
        "# Specification\nGoverned by ADR 0002.\n",
    )
    .unwrap();

    let report_fixed = check_release_readiness(&options).unwrap();
    assert!(report_fixed.is_ready);
    assert_eq!(report_fixed.unreferenced_adrs.len(), 0);
}

#[test]
fn test_changelog_parser_helper() {
    let (_guard, ws) = create_temp_workspace();
    let cl_path = ws.join("CHANGELOG.md");
    fs::write(
        &cl_path,
        r#"
# Changelog
Some text

## [Unreleased]
- foo

## [1.2.0] - 2026-09-01
- bar

## [v1.1.0] - 2026-08-15
- baz

## 1.0.0
- initial
"#,
    )
    .unwrap();

    let versions = parse_changelog_versions(&cl_path).unwrap();
    assert_eq!(versions, vec!["Unreleased", "1.2.0", "v1.1.0", "1.0.0"]);
}

#[test]
fn test_real_workspace_release_readiness() {
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf();

    let options = ReleaseReadinessOptions {
        workspace: ws,
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).unwrap();
    assert_eq!(report.declared_version, env!("CARGO_PKG_VERSION"));
    assert!(report.versions_by_source.contains_key("Cargo.toml"));
    assert!(report.versions_by_source.contains_key("package.json"));
    assert!(
        report.is_ready,
        "Real workspace must be release ready! Diagnostics: {:?}",
        report.diagnostics
    );
    assert_eq!(report.diagnostics.len(), 0);
    assert_eq!(report.unreferenced_adrs.len(), 0);
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
fn test_cli_check_release_integration() {
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf();

    let bin = find_xgauntlet_binary();
    if !bin.is_file() {
        return;
    }

    // 1. Run check-release standard
    let output = std::process::Command::new(&bin)
        .args(["check-release", "--workspace", ws.to_str().unwrap()])
        .output()
        .expect("CLI execution failed");

    assert!(
        output.status.success(),
        "CLI check-release should succeed on real workspace. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("xGauntlet Release Readiness Gatekeeper"));
    assert!(stdout.contains("RELEASE READY"));

    // 2. Run check-release with --json
    let json_output = std::process::Command::new(&bin)
        .args([
            "check-release",
            "--workspace",
            ws.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("CLI execution failed");

    assert!(json_output.status.success());
    let json_str = String::from_utf8_lossy(&json_output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Valid JSON expected from --json");
    assert_eq!(parsed["is_ready"], true);
    assert_eq!(parsed["declared_version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_unformatted_code_detection_in_release_gatekeeper() {
    let (_guard, ws) = create_temp_workspace();

    // 1. Cargo.toml
    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "dummy"
version = "1.0.0"
edition = "2021"
"#,
    )
    .unwrap();

    // 2. CHANGELOG.md
    fs::write(
        ws.join("CHANGELOG.md"),
        "# Changelog\n\n## [1.0.0] - 2026-10-16\n- Initial release\n",
    )
    .unwrap();

    // 3. gauntlet.toml with a format layer that deliberately fails
    fs::write(
        ws.join("gauntlet.toml"),
        r#"
stack = "rust"
save_evidence = false

[[layers]]
name = "format"
command = ["false"]
optional = false
timeout_seconds = 10.0
"#,
    )
    .unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).expect("evaluation failed");
    assert!(!report.is_ready);
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.category == ReleaseFindingCategory::UnformattedCode));
}

#[test]
fn test_wasm_policy_version_mismatch_detection_in_release_gatekeeper() {
    let (_guard, ws) = create_temp_workspace();

    // 1. Cargo.toml declaring version 9.9.9
    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "dummy"
version = "9.9.9"
edition = "2021"
"#,
    )
    .unwrap();

    // 2. CHANGELOG.md with entry for 9.9.9
    fs::write(
        ws.join("CHANGELOG.md"),
        "# Changelog\n\n## [9.9.9] - 2026-10-16\n- Test version\n",
    )
    .unwrap();

    // 3. Fake gauntlet-policy-engine crate directory
    fs::create_dir_all(ws.join("crates/gauntlet-policy-engine")).unwrap();

    // 4. Place valid wasm bytecode (from real current crate, which is version 0.8.2 != 9.9.9)
    let wasm_dir = ws.join("crates/xgauntlet-core/wasm");
    fs::create_dir_all(&wasm_dir).unwrap();
    let current_wasm = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wasm/gauntlet_policy.wasm");
    fs::copy(&current_wasm, wasm_dir.join("gauntlet_policy.wasm")).unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).expect("evaluation failed");
    assert!(!report.is_ready);
    assert!(report.diagnostics.iter().any(|d| {
        d.file_path == "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
            && d.category == ReleaseFindingCategory::ConfigVersionMismatch
            && d.message
                .contains("Embedded WebAssembly policy bytecode version")
    }));
}

#[test]
fn test_format_gate_fails_closed_when_formatter_missing() {
    let (_guard, ws) = create_temp_workspace();
    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "dummy"
version = "1.0.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::write(
        ws.join("CHANGELOG.md"),
        "# Changelog\n\n## [1.0.0] - 2026-10-17\n- Initial release\n",
    )
    .unwrap();
    fs::write(
        ws.join("gauntlet.toml"),
        r#"
stack = "rust"
[[layers]]
name = "format"
command = ["nonexistent-formatter-binary-xyz", "--check"]
optional = false
timeout_seconds = 5.0
"#,
    )
    .unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).expect("evaluation failed");
    assert!(
        !report.is_ready,
        "Gatekeeper must fail closed when format binary is missing"
    );
    assert!(report.diagnostics.iter().any(|d| {
        d.category == ReleaseFindingCategory::UnformattedCode
            && (d.message.contains("failed")
                || d.message.contains("error")
                || d.message.contains("spawn")
                || d.message.contains("not found"))
    }));
}

#[test]
fn test_wasm_policy_fails_closed_on_corrupt_bytecode() {
    let (_guard, ws) = create_temp_workspace();
    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "dummy"
version = "1.0.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::write(
        ws.join("CHANGELOG.md"),
        "# Changelog\n\n## [1.0.0] - 2026-10-17\n- Initial release\n",
    )
    .unwrap();
    fs::create_dir_all(ws.join("crates/gauntlet-policy-engine")).unwrap();
    let wasm_dir = ws.join("crates/xgauntlet-core/wasm");
    fs::create_dir_all(&wasm_dir).unwrap();
    fs::write(
        wasm_dir.join("gauntlet_policy.wasm"),
        b"CORRUPT_NON_WASM_BYTECODE",
    )
    .unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: false,
    };

    let report = check_release_readiness(&options).expect("evaluation failed");
    assert!(
        !report.is_ready,
        "Gatekeeper must fail closed on corrupt wasm policy bytecode"
    );
    assert!(report
        .diagnostics
        .iter()
        .any(|d| { d.file_path == "crates/xgauntlet-core/wasm/gauntlet_policy.wasm" }));
}

#[test]
fn test_strict_release_gate_fails_closed_when_evidence_report_missing() {
    let (_guard, ws) = create_temp_workspace();
    fs::write(
        ws.join("Cargo.toml"),
        r#"
[package]
name = "dummy"
version = "1.0.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::write(
        ws.join("CHANGELOG.md"),
        "# Changelog\n\n## [1.0.0] - 2026-10-17\n- Initial release\n",
    )
    .unwrap();

    let options = ReleaseReadinessOptions {
        workspace: ws.clone(),
        allow_unreleased: false,
        strict: true,
    };

    let report = check_release_readiness(&options).expect("evaluation failed");
    assert!(
        !report.is_ready,
        "Strict release gatekeeper must fail closed when verification-report.json is missing"
    );
    assert!(report
        .diagnostics
        .iter()
        .any(|d| { d.file_path == "verification-report.json" }));
}

#[test]
fn test_package_json_key_order_preservation_on_update() {
    let (_guard, ws) = create_temp_workspace();
    let pkg_json_content = r#"{
  "name": "sample-pkg",
  "description": "sample description",
  "version": "1.0.0",
  "scripts": {
    "test": "node test.js"
  },
  "dependencies": {
    "foo": "1.0.0"
  },
  "license": "MIT"
}
"#;
    fs::write(ws.join("package.json"), pkg_json_content).unwrap();

    let updated = update_manifest_versions(&ws, "1.0.1").unwrap();
    assert_eq!(updated, vec!["package.json"]);

    let new_content = fs::read_to_string(ws.join("package.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&new_content).unwrap();
    let keys: Vec<&str> = val
        .as_object()
        .unwrap()
        .keys()
        .map(|s| s.as_str())
        .collect();

    assert_eq!(
        keys,
        vec![
            "name",
            "description",
            "version",
            "scripts",
            "dependencies",
            "license"
        ]
    );
}
