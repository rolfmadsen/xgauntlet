use std::fs;
use std::path::{Path, PathBuf};
use xgauntlet_core::features::release::{
    check_release_readiness, parse_changelog_versions, ReleaseFindingCategory,
    ReleaseReadinessOptions,
};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
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
    assert_eq!(report.declared_version, "0.1.0");
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
    assert_eq!(parsed["declared_version"], "0.1.0");
}
