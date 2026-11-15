use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

fn make_temp_repo(prefix: &str) -> PathBuf {
    let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "xgauntlet_wt_cli_{prefix}_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    run_git(&dir, &["init"]);
    run_git(&dir, &["config", "user.email", "test@example.com"]);
    run_git(&dir, &["config", "user.name", "Tester"]);

    fs::create_dir_all(dir.join("tasks")).unwrap();
    fs::write(dir.join("gauntlet.toml"), "stack = \"rust\"\n").unwrap();
    fs::write(
        dir.join("tasks/093-sample-task.md"),
        "---\n\
         type: Task Package\n\
         title: \"Task 093: Sample\"\n\
         description: \"Sample task\"\n\
         status: active\n\
         generated: { by: process:test, at: \"2026-11-15T12:00:00Z\" }\n\
         tags: [test]\n\
         scope:\n\
           - \"src/**\"\n\
         test_refs:\n\
           - \"tests/sample_test.rs\"\n\
         ---\n\n\
         # Task 093: Sample\n\n\
         **Status**: `ACTIVE`\n\n\
         ## 🎯 Formål\nSample.\n\n\
         ## 📋 Acceptance Criteria\n- [ ] AC1\n\n\
         ## 🚫 Must NOT\n- None.\n",
    )
    .unwrap();

    run_git(&dir, &["add", "."]);
    run_git(&dir, &["commit", "-m", "init"]);
    dir
}

fn run_git(dir: &Path, args: &[&str]) {
    let st = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git failed");
    assert!(st.success(), "git {:?} failed", args);
}

#[test]
fn test_ac5_cli_worktree_create_creates_in_repo_worktree_and_binds_task() {
    let repo = make_temp_repo("create");
    let bin = env!("CARGO_BIN_EXE_xgauntlet");

    let out = Command::new(bin)
        .args(["worktree", "create", "093", "--workspace", repo.to_str().unwrap()])
        .output()
        .expect("failed to execute xgauntlet worktree create");

    assert!(
        out.status.success(),
        "xgauntlet worktree create 093 must succeed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let wt_dir = repo.join(".worktrees/093");
    assert!(
        wt_dir.is_dir(),
        "Worktree directory .worktrees/093 must be created inside workspace"
    );
    let active_task = fs::read_to_string(wt_dir.join(".xgauntlet/active_task"))
        .expect(".xgauntlet/active_task must be automatically bound inside created worktree");
    assert_eq!(active_task.trim(), "093-sample-task");

    let _ = fs::remove_dir_all(&repo);
}
