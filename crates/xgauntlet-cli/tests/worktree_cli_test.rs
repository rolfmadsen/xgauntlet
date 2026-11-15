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
        .args([
            "worktree",
            "create",
            "093",
            "--workspace",
            repo.to_str().unwrap(),
        ])
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

#[test]
fn test_ac5_cli_worktree_create_fails_closed_without_git_pollution_on_non_active_or_untracked_task()
{
    let repo = make_temp_repo("rollback");
    let bin = env!("CARGO_BIN_EXE_xgauntlet");

    // 1. Committed DONE task must be rejected before creating .worktrees/094 or branch task/094
    fs::write(
        repo.join("tasks/094-done-task.md"),
        "---\n\
         type: Task Package\n\
         title: \"Task 094: Done\"\n\
         description: \"Completed task\"\n\
         status: done\n\
         generated: { by: process:test, at: \"2026-11-15T12:00:00Z\" }\n\
         tags: [test]\n\
         scope: [\"src/**\"]\n\
         test_refs: [\"tests/done_test.rs\"]\n\
         ---\n\n\
         # Task 094: Done\n\n\
         **Status**: `DONE`\n",
    )
    .unwrap();
    run_git(&repo, &["add", "tasks/094-done-task.md"]);
    run_git(&repo, &["commit", "-m", "add done task"]);

    let out_done = Command::new(bin)
        .args([
            "worktree",
            "create",
            "094",
            "--workspace",
            repo.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute worktree create for done task");
    assert!(
        !out_done.status.success(),
        "DONE task must be rejected by worktree create"
    );
    assert!(
        !repo.join(".worktrees/094").exists(),
        "Rejected DONE task must not leave .worktrees/094 behind"
    );
    let branch_done = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", "refs/heads/task/094"])
        .current_dir(&repo)
        .status()
        .unwrap();
    assert!(
        !branch_done.success(),
        "Rejected DONE task must not create branch task/094"
    );

    // 2. Untracked ACTIVE task must also fail closed without leaving .worktrees/095 or branch task/095
    fs::write(
        repo.join("tasks/095-untracked-task.md"),
        "---\n\
         type: Task Package\n\
         title: \"Task 095: Untracked\"\n\
         description: \"Untracked task\"\n\
         status: active\n\
         generated: { by: process:test, at: \"2026-11-15T12:00:00Z\" }\n\
         tags: [test]\n\
         scope: [\"src/**\"]\n\
         test_refs: [\"tests/untracked_test.rs\"]\n\
         ---\n\n\
         # Task 095: Untracked\n\n\
         **Status**: `ACTIVE`\n",
    )
    .unwrap();

    let out_untracked = Command::new(bin)
        .args([
            "worktree",
            "create",
            "095",
            "--workspace",
            repo.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute worktree create for untracked task");
    assert!(
        !out_untracked.status.success(),
        "Untracked task must be rejected by worktree create"
    );
    assert!(
        !repo.join(".worktrees/095").exists(),
        "Rejected untracked task must not leave .worktrees/095 behind"
    );
    let branch_untracked = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", "refs/heads/task/095"])
        .current_dir(&repo)
        .status()
        .unwrap();
    assert!(
        !branch_untracked.success(),
        "Rejected untracked task must not create branch task/095"
    );

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_strip_windows_verbatim_prefix_normalizes_git_worktree_paths() {
    use xgauntlet_core::features::tasks::strip_windows_verbatim_prefix;

    assert_eq!(
        strip_windows_verbatim_prefix(
            r"\\?\C:\Users\runneradmin\AppData\Local\Temp\repo\.worktrees\093"
        ),
        r"C:\Users\runneradmin\AppData\Local\Temp\repo\.worktrees\093"
    );
    assert_eq!(
        strip_windows_verbatim_prefix(r"\\?\UNC\server\share\repo\.worktrees\093"),
        r"\\server\share\repo\.worktrees\093"
    );
    assert_eq!(
        strip_windows_verbatim_prefix("/tmp/repo/.worktrees/093"),
        "/tmp/repo/.worktrees/093"
    );
}
