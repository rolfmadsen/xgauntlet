//! Acceptance and contract tests for Task 049: Mechanical Monotonic Ratchet Enforcement.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xgauntlet_core::features::architecture::{
    run_architecture_check, ArchitectureFindingCategory,
};
use xgauntlet_core::features::config::load_config;

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
        let path = std::env::temp_dir().join(format!("xgauntlet_ratchet_test_{}", unique));
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

struct GitTestRepo {
    dir: TempDir,
}

impl GitTestRepo {
    fn new(prefix: &str) -> Self {
        let dir = TempDir::new(prefix);
        let ws = dir.path();
        run_git(ws, &["init", "-b", "main"]);
        run_git(ws, &["config", "user.name", "Ratchet Test"]);
        run_git(ws, &["config", "user.email", "ratchet@example.com"]);
        Self { dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn commit_all(&self, msg: &str) {
        let ws = self.path();
        run_git(ws, &["add", "-A"]);
        run_git(ws, &["commit", "-m", msg, "--allow-empty"]);
    }
}

fn run_git(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute git {:?}: {}", args, e));
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "git {:?} failed (exit code {:?}):\nstdout: {}\nstderr: {}",
            args,
            output.status.code(),
            stdout,
            stderr
        );
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn write_file(ws: &Path, rel: &str, content: &str) {
    let p = ws.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, content).unwrap();
}

fn make_adr_content(status: &str, extra_body: &str) -> String {
    format!(
        "---\ntype: Architectural Decision Record\ntitle: 'ADR 0010: Test ADR'\nstatus: {}\ntags: [architecture]\n---\n\n# 10. Test ADR\n\n## Context\nTesting ratchets.\n\n## Decision\n{}\n",
        status, extra_body
    )
}

#[test]
fn test_ratchet_enforcement_rejects_unauthorized_ceiling_inflation() {
    let repo = GitTestRepo::new("ratchet_reject_unauth");
    let ws = repo.path();

    // Baseline commit with max_lines = 2100
    let baseline_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2100
"#;
    write_file(ws, "gauntlet.toml", baseline_toml);
    write_file(ws, "docs/adr/0010-test.md", &make_adr_content("accepted", "No ratchet line"));
    write_file(ws, "src/big.rs", &"// line\n".repeat(1500));
    repo.commit_all("initial baseline with ceiling 2100");

    // Working tree inflates ceiling from 2100 to 2190 without ADR authorization
    let inflated_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2190
"#;
    write_file(ws, "gauntlet.toml", inflated_toml);

    let config = load_config(ws).expect("load config");
    let report = run_architecture_check(ws, &config.architecture).expect("run architecture check");

    assert!(
        !report.is_valid,
        "Expected architecture check to FAIL due to unauthorized ratchet inflation"
    );
    let finding = report
        .findings
        .iter()
        .find(|f| f.category == ArchitectureFindingCategory::RatchetCeilingInflated)
        .expect("Expected RatchetCeilingInflated finding");

    assert_eq!(finding.file_path, "src/big.rs");
    assert!(
        finding.message.contains("inflated from 2100 to 2190"),
        "Unexpected message: {}",
        finding.message
    );
}

#[test]
fn test_ratchet_enforcement_accepts_authorized_ceiling_inflation() {
    let repo = GitTestRepo::new("ratchet_accept_auth");
    let ws = repo.path();

    // Baseline commit with max_lines = 2100
    let baseline_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2100
"#;
    write_file(ws, "gauntlet.toml", baseline_toml);
    write_file(ws, "docs/adr/0010-test.md", &make_adr_content("accepted", "No ratchet line"));
    write_file(ws, "src/big.rs", &"// line\n".repeat(1500));
    repo.commit_all("initial baseline with ceiling 2100");

    // Working tree inflates ceiling from 2100 to 2180, AND ADR contains explicit ratchet line
    let inflated_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2180
"#;
    write_file(ws, "gauntlet.toml", inflated_toml);
    write_file(
        ws,
        "docs/adr/0010-test.md",
        &make_adr_content("accepted", "Autoriseret loft:\n- ratchet: src/big.rs <= 2180\n"),
    );

    let config = load_config(ws).expect("load config");
    let report = run_architecture_check(ws, &config.architecture).expect("run architecture check");

    assert!(
        report.is_valid,
        "Expected architecture check to PASS with authorized ratchet line. Findings: {:?}",
        report.findings
    );
}

#[test]
fn test_ratchet_enforcement_rejects_inflation_exceeding_authorized_limit() {
    let repo = GitTestRepo::new("ratchet_reject_exceeding");
    let ws = repo.path();

    let baseline_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2100
"#;
    write_file(ws, "gauntlet.toml", baseline_toml);
    write_file(ws, "docs/adr/0010-test.md", &make_adr_content("accepted", ""));
    write_file(ws, "src/big.rs", &"// line\n".repeat(1500));
    repo.commit_all("initial baseline with ceiling 2100");

    // ADR authorizes up to 2180, but working tree inflates to 2190!
    let inflated_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2190
"#;
    write_file(ws, "gauntlet.toml", inflated_toml);
    write_file(
        ws,
        "docs/adr/0010-test.md",
        &make_adr_content("accepted", "- ratchet: src/big.rs <= 2180\n"),
    );

    let config = load_config(ws).expect("load config");
    let report = run_architecture_check(ws, &config.architecture).expect("run architecture check");

    assert!(
        !report.is_valid,
        "Expected architecture check to FAIL when ceiling exceeds authorized limit"
    );
    assert!(report
        .findings
        .iter()
        .any(|f| f.category == ArchitectureFindingCategory::RatchetCeilingInflated));
}

#[test]
fn test_ratchet_enforcement_allows_lowered_or_unchanged_ceilings() {
    let repo = GitTestRepo::new("ratchet_allow_lowered");
    let ws = repo.path();

    let baseline_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2100
"#;
    write_file(ws, "gauntlet.toml", baseline_toml);
    write_file(ws, "docs/adr/0010-test.md", &make_adr_content("accepted", "No ratchet line"));
    write_file(ws, "src/big.rs", &"// line\n".repeat(1500));
    repo.commit_all("initial baseline with ceiling 2100");

    // Lowered ceiling from 2100 to 2050 (tightening ratchet)
    let lowered_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2050
"#;
    write_file(ws, "gauntlet.toml", lowered_toml);

    let config = load_config(ws).expect("load config");
    let report = run_architecture_check(ws, &config.architecture).expect("run architecture check");

    assert!(
        report.is_valid,
        "Expected lowered ceiling to pass without ADR ratchet line. Findings: {:?}",
        report.findings
    );
}

#[test]
fn test_ratchet_enforcement_allows_new_exemption_with_valid_adr() {
    let repo = GitTestRepo::new("ratchet_allow_new_exemption");
    let ws = repo.path();

    let baseline_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800
"#;
    write_file(ws, "gauntlet.toml", baseline_toml);
    write_file(ws, "src/lib.rs", "pub fn dummy() {}\n");
    repo.commit_all("initial baseline without exemptions");

    // Introduce brand new exemption
    let new_ex_toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/new_big.rs"
reason = "Newly documented legacy module"
adr = "docs/adr/0011-new.md"
max_lines = 1500
"#;
    write_file(ws, "gauntlet.toml", new_ex_toml);
    write_file(ws, "docs/adr/0011-new.md", &make_adr_content("accepted", ""));
    write_file(ws, "src/new_big.rs", &"// line\n".repeat(1200));

    let config = load_config(ws).expect("load config");
    let report = run_architecture_check(ws, &config.architecture).expect("run architecture check");

    assert!(
        report.is_valid,
        "Expected new exemption with valid ADR to PASS. Findings: {:?}",
        report.findings
    );
}

#[test]
fn test_ratchet_enforcement_skips_in_non_git_workspace_with_notice() {
    let temp = TempDir::new("ratchet_non_git");
    let ws = temp.path();

    let toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2190
"#;
    write_file(ws, "gauntlet.toml", toml);
    write_file(ws, "docs/adr/0010-test.md", &make_adr_content("accepted", ""));
    write_file(ws, "src/big.rs", &"// line\n".repeat(1500));

    let config = load_config(ws).expect("load config");
    let report = run_architecture_check(ws, &config.architecture).expect("run architecture check");

    assert!(report.is_valid);
    assert!(
        report.notices.iter().any(|n| n.contains("No Git repository")),
        "Expected notice about skipping git baseline comparison. Notices: {:?}",
        report.notices
    );
}

#[test]
fn test_ratchet_enforcement_fails_closed_when_git_baseline_unreadable() {
    let temp = TempDir::new("ratchet_unreadable_baseline");
    let ws = temp.path();

    // Initialize git repo but make NO commits (HEAD is unresolvable)
    run_git(ws, &["init", "-b", "main"]);

    let toml = r#"
stack = "rust"
[architecture]
max_file_lines = 800

[[architecture.exemptions]]
path = "src/big.rs"
reason = "Legacy module"
adr = "docs/adr/0010-test.md"
max_lines = 2100
"#;
    write_file(ws, "gauntlet.toml", toml);
    write_file(ws, "docs/adr/0010-test.md", &make_adr_content("accepted", ""));
    write_file(ws, "src/big.rs", &"// line\n".repeat(500));

    let config = load_config(ws).expect("load config");
    let report = run_architecture_check(ws, &config.architecture).expect("run architecture check");

    assert!(
        !report.is_valid,
        "Expected check to fail closed when baseline cannot be read"
    );
    assert!(report
        .findings
        .iter()
        .any(|f| f.category == ArchitectureFindingCategory::RatchetBaselineUnreadable));
}
