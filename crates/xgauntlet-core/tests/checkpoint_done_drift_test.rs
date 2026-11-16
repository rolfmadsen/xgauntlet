use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::checkpoint::{run_checkpoint, CheckpointOptions, CheckpointPhase};
use xgauntlet_core::features::evidence::{
    compute_workspace_manifest, load_verification_report, verify_workspace_state_match,
};

static COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempRepo {
    path: PathBuf,
}

impl TempRepo {
    fn new(prefix: &str) -> Self {
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "xgauntlet_done_drift_{prefix}_{}_{}",
            std::process::id(),
            seq
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self { path: dir }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[tokio::test]
async fn test_done_checkpoint_refreshes_evidence_digests_without_drift() {
    let temp = TempRepo::new("zero_drift");
    let ws = temp.path();

    assert!(Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(ws)
        .status()
        .unwrap()
        .success());
    Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(ws)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(ws)
        .status()
        .unwrap();

    fs::write(
        ws.join("CONTEXT.md"),
        "**Task**:\nExecutable unit of work.\n_Avoid_: Ticket.\n",
    )
    .unwrap();
    fs::write(
        ws.join("gauntlet.toml"),
        "stack = \"rust\"\nsave_evidence = true\n[[layers]]\nname = \"unit\"\ncommand = [\"true\"]\noptional = false\n",
    )
    .unwrap();

    fs::create_dir_all(ws.join("tasks")).unwrap();
    fs::write(
        ws.join("tasks/001-drift-check.md"),
        "---\ntype: Task Package\ntitle: \"Task 001: Drift\"\nstatus: active\nscope:\n  - \"crates/**\"\ntest_refs:\n  - \"tests/**\"\n---\n\n# Task 001: Drift\n\n**Status**: `ACTIVE`\n\n## 🎯 Formål\nTest.\n\n## 📋 Acceptance Criteria\n- [x] Done\n\n## 🚫 Must NOT\n- None.\n",
    )
    .unwrap();

    fs::create_dir_all(ws.join("crates")).unwrap();
    fs::write(ws.join("crates/lib.rs"), "// red\n").unwrap();
    fs::write(ws.join("verification-report.json"), "{}\n").unwrap();
    fs::write(ws.join("evidence.md"), "# Evidence\n").unwrap();

    Command::new("git")
        .args(["add", "-A"])
        .current_dir(ws)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "test(001): add failing test [RED]"])
        .current_dir(ws)
        .status()
        .unwrap();

    fs::write(ws.join("crates/lib.rs"), "// green\n").unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Done, ws);
    run_checkpoint(&opts)
        .await
        .expect("DONE checkpoint should succeed");

    let report = load_verification_report(ws).expect("verification-report.json must load");
    let manifest = compute_workspace_manifest(ws, None).expect("manifest must compute");

    verify_workspace_state_match(&report, &manifest)
        .expect("verification-report.json must match post-DONE workspace manifest with zero drift");
}
