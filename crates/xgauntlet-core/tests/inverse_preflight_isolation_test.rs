//! Acceptance tests for Task 047: Worktree-Isolated Inverse Preflight.
//!
//! Inverse preflight must evaluate new tests against baseline (`HEAD`) production
//! code WITHOUT ever touching the user's working tree. These tests observe the
//! working tree from inside the test layer itself and after preflight completes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::checkpoint::provenance::{
    verify_inverse_test_failure, TestProvenanceStatus,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "xgauntlet_isolation_test_{prefix}_{}_{count}_{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        let canon = path.canonicalize().unwrap();
        #[cfg(windows)]
        let clean = {
            let s = canon.to_string_lossy();
            if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
                std::path::PathBuf::from(format!(r"\\{}", stripped))
            } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
                std::path::PathBuf::from(stripped)
            } else {
                canon
            }
        };
        #[cfg(not(windows))]
        let clean = canon;

        Self { path: clean }
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

fn git(ws: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(ws)
        .output()
        .expect("git must be available");
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn init_repo(ws: &Path) {
    let init = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(ws)
        .output()
        .unwrap();
    if !init.status.success() {
        git(ws, &["init"]);
    }
    git(ws, &["config", "user.name", "Test Craftsman"]);
    git(ws, &["config", "user.email", "craftsman@xgauntlet.dev"]);
    git(ws, &["config", "commit.gpgsign", "false"]);
}

fn write_test_layer(ws: &Path, command: &[&str]) {
    let quoted: Vec<String> = command
        .iter()
        .map(|c| format!("\"{}\"", c.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();
    fs::write(
        ws.join("gauntlet.toml"),
        format!(
            "[stack]\nname = \"custom\"\n\n[[layers]]\nname = \"test\"\ncommand = [{}]\n",
            quoted.join(", ")
        ),
    )
    .unwrap();
}

fn write(ws: &Path, rel: &str, content: &str) {
    let full = ws.join(rel);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

fn commit_all(ws: &Path, msg: &str) {
    git(ws, &["add", "-A"]);
    git(ws, &["commit", "-q", "-m", msg]);
}

fn add_new_test(ws: &Path, name: &str) {
    write(
        ws,
        &format!("tests/{name}_test.rs"),
        &format!("#[test]\nfn {name}() {{ assert!(true); }}\n"),
    );
}

/// AC1: production files in the user's workspace keep their working-tree content
/// while the test layer is running against baseline.
#[test]
fn test_preflight_leaves_working_tree_untouched_during_test_run() {
    let temp = TempDir::new("untouched");
    let observer = TempDir::new("untouched_observer");
    let ws = temp.path();
    init_repo(ws);

    let ws_file = ws.join("src/lib.rs").to_string_lossy().replace('\\', "/");
    let obs_file = observer
        .path()
        .join("observed.txt")
        .to_string_lossy()
        .replace('\\', "/");
    let script = format!(
        "const fs = require('fs'); fs.copyFileSync('{ws_file}', '{obs_file}'); const content = fs.readFileSync('src/lib.rs', 'utf8'); if (content.includes('worldmarker')) process.exit(0); else process.exit(1);"
    );
    write_test_layer(ws, &["node", "-e", &script]);
    write(
        ws,
        "src/lib.rs",
        "pub fn greet() -> &'static str { \"hello\" }\n",
    );
    commit_all(ws, "chore: baseline");

    write(
        ws,
        "src/lib.rs",
        "pub fn greet() -> &'static str { \"worldmarker\" }\n",
    );
    add_new_test(ws, "test_greet_world");

    let evidences = verify_inverse_test_failure(ws, &["test_greet_world".to_string()])
        .expect("legitimate test must be verified against baseline");
    assert_eq!(
        evidences[0].status,
        TestProvenanceStatus::InversePreflightVerified
    );

    let observed = fs::read_to_string(observer.path().join("observed.txt")).unwrap();
    assert!(
        observed.contains("worldmarker"),
        "user's working tree was overwritten during preflight; observed: {observed}"
    );
    let after = fs::read_to_string(ws.join("src/lib.rs")).unwrap();
    assert!(after.contains("worldmarker"));
}

/// AC2: the test layer runs outside the workspace, and the temporary worktree is
/// fully removed afterwards.
#[test]
fn test_preflight_runs_in_isolated_worktree_and_cleans_up() {
    let temp = TempDir::new("isolated");
    let observer = TempDir::new("isolated_observer");
    let ws = temp.path();
    init_repo(ws);

    let obs_file = observer
        .path()
        .join("cwd.txt")
        .to_string_lossy()
        .replace('\\', "/");
    let script = format!(
        "const fs = require('fs'); fs.writeFileSync('{obs_file}', process.cwd()); const content = fs.readFileSync('src/lib.rs', 'utf8'); if (content.includes('worldmarker')) process.exit(0); else process.exit(1);"
    );
    write_test_layer(ws, &["node", "-e", &script]);
    write(ws, "src/lib.rs", "hello\n");
    commit_all(ws, "chore: baseline");

    write(ws, "src/lib.rs", "worldmarker\n");
    add_new_test(ws, "test_isolated");

    verify_inverse_test_failure(ws, &["test_isolated".to_string()])
        .expect("legitimate test must be verified against baseline");

    let cwd_raw = fs::read_to_string(observer.path().join("cwd.txt")).unwrap();
    let cwd = PathBuf::from(cwd_raw.trim());
    let clean_cwd = {
        let canon = cwd.canonicalize().unwrap_or_else(|_| cwd.clone());
        let s = canon.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
            PathBuf::from(format!(r"\\{}", stripped))
        } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
            PathBuf::from(stripped)
        } else {
            canon
        }
    };
    let clean_ws = {
        let canon = ws.canonicalize().unwrap_or_else(|_| ws.to_path_buf());
        let s = canon.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
            PathBuf::from(format!(r"\\{}", stripped))
        } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
            PathBuf::from(stripped)
        } else {
            canon
        }
    };
    assert_ne!(
        clean_cwd, clean_ws,
        "test layer must not execute inside the user's workspace"
    );
    assert!(
        !cwd.exists(),
        "temporary worktree {} was not removed",
        cwd.display()
    );

    let worktrees = git(ws, &["worktree", "list", "--porcelain"]);
    let count = worktrees
        .lines()
        .filter(|l| l.starts_with("worktree "))
        .count();
    assert_eq!(count, 1, "stale worktree registrations: {worktrees}");
}

/// AC3: a production file deleted in the working tree must not reappear.
#[test]
fn test_preflight_does_not_resurrect_deleted_production_file() {
    let temp = TempDir::new("resurrect");
    let ws = temp.path();
    init_repo(ws);

    write_test_layer(ws, &["grep", "-q", "worldmarker", "src/lib.rs"]);
    write(ws, "src/lib.rs", "hello\n");
    write(ws, "src/legacy.rs", "pub fn legacy() {}\n");
    commit_all(ws, "chore: baseline");

    fs::remove_file(ws.join("src/legacy.rs")).unwrap();
    write(ws, "src/lib.rs", "worldmarker\n");
    add_new_test(ws, "test_without_legacy");

    verify_inverse_test_failure(ws, &["test_without_legacy".to_string()])
        .expect("legitimate test must be verified against baseline");

    assert!(
        !ws.join("src/legacy.rs").exists(),
        "deleted production file was resurrected by inverse preflight"
    );
}

/// AC4: untracked new production files are not part of the baseline.
#[test]
fn test_preflight_baseline_excludes_untracked_production_files() {
    let temp = TempDir::new("untracked");
    let ws = temp.path();
    init_repo(ws);

    write_test_layer(ws, &["grep", "-q", "featuremarker", "src/feature.rs"]);
    write(ws, "src/lib.rs", "hello\n");
    commit_all(ws, "chore: baseline");

    write(ws, "src/feature.rs", "// featuremarker\n");
    add_new_test(ws, "test_feature");

    let evidences = verify_inverse_test_failure(ws, &["test_feature".to_string()])
        .expect("test depending on a new untracked production file must fail against baseline");
    assert_eq!(
        evidences[0].status,
        TestProvenanceStatus::InversePreflightVerified
    );
    assert!(ws.join("src/feature.rs").exists());
}

/// AC5: if an isolated baseline cannot be established, preflight fails closed.
#[test]
fn test_preflight_fails_closed_without_head() {
    let temp = TempDir::new("no_head");
    let ws = temp.path();
    init_repo(ws);

    write_test_layer(ws, &["false"]);
    add_new_test(ws, "test_orphan");

    let res = verify_inverse_test_failure(ws, &["test_orphan".to_string()]);
    assert!(
        res.is_err(),
        "preflight without a HEAD baseline must fail closed, got: {res:?}"
    );
}
