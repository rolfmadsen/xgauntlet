use std::fs;
use std::path::{Path, PathBuf};
use xgauntlet_core::{
    bump_semver, generate_release_git_instructions, prepare_changelog_release,
    update_manifest_versions, CheckpointOptions, CheckpointPhase, ReleaseError, SemverBump,
    SessionHandoff,
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
        let path = std::env::temp_dir().join(format!("xgauntlet_handoff_test_{}", unique));
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
fn test_semver_bumping_rules() {
    assert_eq!(bump_semver("0.3.4", SemverBump::Patch).unwrap(), "0.3.5");
    assert_eq!(bump_semver("0.3.4", SemverBump::Minor).unwrap(), "0.4.0");
    assert_eq!(bump_semver("0.3.4", SemverBump::Major).unwrap(), "1.0.0");
    assert_eq!(bump_semver("v1.2.3", SemverBump::Patch).unwrap(), "1.2.4");
    assert!(bump_semver("invalid.semver", SemverBump::Patch).is_err());
}

#[test]
fn test_manifest_version_bumping() {
    let dir = TempDir::new("manifest_bump");
    let root_cargo = dir.path().join("Cargo.toml");
    let initial_cargo = r#"[workspace]
members = ["crates/*"]

[workspace.package]
name = "demo"
version = "0.3.4"
edition = "2021"
"#;
    fs::write(&root_cargo, initial_cargo).unwrap();

    let updated_files = update_manifest_versions(dir.path(), "0.4.0").unwrap();
    assert!(!updated_files.is_empty());

    let content = fs::read_to_string(&root_cargo).unwrap();
    assert!(content.contains("version = \"0.4.0\""));
    assert!(!content.contains("version = \"0.3.4\""));
}

#[test]
fn test_changelog_release_rotation() {
    let dir = TempDir::new("changelog_rot");
    let changelog_path = dir.path().join("CHANGELOG.md");
    let initial_changelog = r#"# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Super feature X
"#;
    fs::write(&changelog_path, initial_changelog).unwrap();

    let rotated = prepare_changelog_release(dir.path(), "0.4.0", "2026-10-16").unwrap();
    assert!(rotated);

    let updated = fs::read_to_string(&changelog_path).unwrap();
    assert!(updated.contains("## [Unreleased]"));
    assert!(updated.contains("## [0.4.0] - 2026-10-16"));
    assert!(updated.contains("### Added"));
}

#[test]
fn test_generate_release_git_instructions() {
    let instructions = generate_release_git_instructions("0.4.0", "main");
    assert!(instructions.contains("git add Cargo.toml Cargo.lock CHANGELOG.md"));
    assert!(instructions.contains("git commit -m \"chore(release): prepare v0.4.0\""));
    assert!(instructions.contains("git tag -a v0.4.0 -m \"Release v0.4.0\""));
    assert!(instructions.contains("git push origin main --tags"));
}

#[test]
fn test_session_handoff_card_generation() {
    let dir = TempDir::new("handoff_card");
    let ws = dir.path();

    // Initialize git repository
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(ws)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(ws)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "tester@example.com"])
        .current_dir(ws)
        .output()
        .unwrap();

    // Create an initial commit
    let dummy_file = ws.join("README.md");
    fs::write(&dummy_file, "# Demo\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(ws)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(ws)
        .output()
        .unwrap();

    let handoff = SessionHandoff::generate(ws, "035-mechanical-session-handoff", Some("Architect"))
        .expect("should generate session handoff");

    assert_eq!(handoff.task_id, "035-mechanical-session-handoff");
    assert_eq!(handoff.branch, "main");
    assert_eq!(handoff.next_role, "Architect");
    assert!(handoff.formatted_card.contains("### 🏁 SESSION HANDOFF • 035"));
    assert!(handoff.formatted_card.contains("git push origin main"));
    assert!(handoff.formatted_card.contains("Kopiér og indsæt følgende starter-prompt"));
}
