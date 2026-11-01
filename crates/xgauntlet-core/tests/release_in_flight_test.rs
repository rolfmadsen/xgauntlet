//! Tests for transactional in-flight release gatekeeper and strict release readiness validation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use xgauntlet_core::features::release::{
    check_release_readiness, ReleaseFindingCategory, ReleaseReadinessOptions,
};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("xg_rel_test_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn init_git_repo(path: &Path) {
    let _ = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output()
        .or_else(|_| {
            Command::new("git")
                .args(["init"])
                .current_dir(path)
                .output()
        });

    let _ = Command::new("git")
        .args(["config", "user.name", "Test Committer"])
        .current_dir(path)
        .output();

    let _ = Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output();
}

fn scaffold_minimal_release_repo(ws: &Path, version: &str) {
    init_git_repo(ws);
    fs::create_dir_all(ws.join("src")).unwrap();

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "my-project"
version = "{version}"
edition = "2021"

[dependencies]
"#
    );
    fs::write(ws.join("Cargo.toml"), cargo_toml).unwrap();

    // CHANGELOG.md
    let changelog = format!(
        r#"# Changelog

## [{version}] - 2026-11-01

### Added
- Initial release
"#
    );
    fs::write(ws.join("CHANGELOG.md"), changelog).unwrap();

    // Commit baseline so git repo has a HEAD
    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(ws)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(ws)
        .output();

    // Uncommitted file to make worktree dirty
    fs::write(ws.join("uncommitted.txt"), "dirty worktree content").unwrap();
}

#[test]
fn test_in_flight_staging_skips_dirty_worktree_but_enforces_strict_verification() {
    let temp = TempDir::new("in_flight_staging");
    let ws = &temp.path;
    scaffold_minimal_release_repo(ws, "1.0.0");

    // Standard strict mode: Must fail with DirtyWorktree AND MissingManifest (verification-report.json)
    let strict_options = ReleaseReadinessOptions {
        workspace: ws.to_path_buf(),
        allow_unreleased: false,
        strict: true,
        in_flight_staging: false,
    };
    let report_strict =
        check_release_readiness(&strict_options).expect("release evaluation failed");
    assert!(!report_strict.is_ready);
    assert!(
        report_strict
            .diagnostics
            .iter()
            .any(|d| d.category == ReleaseFindingCategory::DirtyWorktree),
        "Strict mode without in_flight_staging must flag DirtyWorktree"
    );
    assert!(
        report_strict
            .diagnostics
            .iter()
            .any(|d| d.file_path == "verification-report.json"),
        "Strict mode must require verification-report.json"
    );

    // In-flight staging mode (strict: true, in_flight_staging: true):
    // Must SKIP DirtyWorktree, but STILL require verification-report.json and 0% drift
    let in_flight_options = ReleaseReadinessOptions {
        workspace: ws.to_path_buf(),
        allow_unreleased: false,
        strict: true,
        in_flight_staging: true,
    };
    let report_in_flight =
        check_release_readiness(&in_flight_options).expect("release evaluation failed");
    assert!(!report_in_flight.is_ready);
    assert!(
        !report_in_flight
            .diagnostics
            .iter()
            .any(|d| d.category == ReleaseFindingCategory::DirtyWorktree),
        "In-flight staging mode MUST skip DirtyWorktree"
    );
    assert!(
        report_in_flight
            .diagnostics
            .iter()
            .any(|d| d.file_path == "verification-report.json"),
        "In-flight staging mode MUST enforce verification-report.json presence"
    );
}
