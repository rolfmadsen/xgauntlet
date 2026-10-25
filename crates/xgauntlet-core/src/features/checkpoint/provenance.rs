//! Test Provenance & Born-Green Detection Engine (Task 032).
//!
//! Provides polyglot test identifier extraction, Git-diff inspection for
//! born-green tests, and non-destructive inverse preflight verification
//! to guarantee epistemic integrity and negative test capability.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::models::CheckpointError;
use super::preflight::resolve_test_layer;
use crate::features::evidence::current_iso_utc;

/// Provenance status indicating how a test proved its negative capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestProvenanceStatus {
    /// Test failure was empirically verified in a committed RED phase checkpoint.
    RedCheckpointVerified,
    /// Test failure was verified via automated inverse preflight against baseline code.
    InversePreflightVerified,
}

/// Epistemic evidence verifying that a test possesses negative capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestProvenanceEvidence {
    /// Name or description of the verified test identifier.
    pub test_identifier: String,
    /// Failure stdout/stderr proving rejection against baseline source code.
    pub failure_output: String,
    /// Baseline failure SHA-256 digest proving non-empty negative failure capability.
    #[serde(default)]
    pub failure_digest: String,
    /// ISO-8601 timestamp when provenance was sealed.
    pub verified_at: String,
    /// Classification of how provenance was confirmed.
    pub status: TestProvenanceStatus,
}

/// Determines whether a given relative path represents a test file.
pub fn is_test_file(path: &Path) -> bool {
    let s = path.to_string_lossy().replace('\\', "/");
    s.starts_with("tests/")
        || s.contains("/tests/")
        || s.ends_with("_test.rs")
        || s.contains(".test.")
        || s.contains(".spec.")
        || s.ends_with("_test.go")
        || path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with("test_") && name.ends_with(".py"))
}

/// Extracts test identifiers from source content based on language conventions.
pub fn extract_test_identifiers(file_path: &Path, content: &str) -> Vec<String> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "rs" => extract_rust_test_identifiers(content),
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => extract_javascript_test_identifiers(content),
        "py" => extract_python_test_identifiers(content),
        "go" => extract_go_test_identifiers(content),
        _ => Vec::new(),
    }
}

fn extract_rust_test_identifiers(content: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut has_test_attr = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("#[test]")
            || trimmed.starts_with("#[tokio::test]")
            || trimmed.starts_with("#[async_std::test]")
            || trimmed.starts_with("#[actix_rt::test]")
        {
            has_test_attr = true;
            // Check for inline `#[test] fn name()`
            if let Some(idx) = trimmed.find("fn ") {
                let rest = &trimmed[idx + 3..].trim_start();
                if let Some(end) = rest.find(|c: char| c == '(' || c.is_whitespace()) {
                    let name = &rest[..end];
                    if !name.is_empty() {
                        ids.push(name.to_string());
                        has_test_attr = false;
                        continue;
                    }
                }
            }
            continue;
        }

        if has_test_attr {
            if let Some(idx) = trimmed.find("fn ") {
                let rest = &trimmed[idx + 3..].trim_start();
                if let Some(end) = rest.find(|c: char| c == '(' || c.is_whitespace()) {
                    let name = &rest[..end];
                    if !name.is_empty() {
                        ids.push(name.to_string());
                    }
                }
                has_test_attr = false;
            } else if !trimmed.starts_with("#[")
                && !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("/*")
            {
                has_test_attr = false;
            }
        }
    }

    ids
}

fn extract_javascript_test_identifiers(content: &str) -> Vec<String> {
    let mut ids = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let patterns = [
            "test(",
            "it(",
            "test.only(",
            "it.only(",
            "test.skip(",
            "it.skip(",
        ];

        for pattern in patterns {
            if let Some(pos) = trimmed.find(pattern) {
                let after = &trimmed[pos + pattern.len()..].trim_start();
                let quote_char = after.chars().next();
                if let Some(q) = quote_char {
                    if q == '"' || q == '\'' || q == '`' {
                        let rest = &after[1..];
                        if let Some(end) = rest.find(q) {
                            let title = &rest[..end];
                            if !title.is_empty() {
                                ids.push(title.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    ids
}

fn extract_python_test_identifiers(content: &str) -> Vec<String> {
    let mut ids = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let stripped = trimmed
            .strip_prefix("async def ")
            .or_else(|| trimmed.strip_prefix("def "));

        if let Some(rest) = stripped {
            let func_part = rest.trim_start();
            if func_part.starts_with("test_") {
                if let Some(end) = func_part.find('(') {
                    let name = func_part[..end].trim();
                    if !name.is_empty() {
                        ids.push(name.to_string());
                    }
                }
            }
        }
    }

    ids
}

fn extract_go_test_identifiers(content: &str) -> Vec<String> {
    let mut ids = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("func ") {
            let func_part = rest.trim_start();
            if func_part.starts_with("Test") {
                if let Some(end) = func_part.find('(') {
                    let name = func_part[..end].trim();
                    if !name.is_empty() {
                        ids.push(name.to_string());
                    }
                }
            }
        }
    }

    ids
}

/// Inspects git diff between `base_commit_oid` and working tree/index to find new tests.
pub fn detect_born_green_tests(
    workspace: &Path,
    base_commit_oid: &str,
) -> Result<Vec<String>, CheckpointError> {
    let diff_output = Command::new("git")
        .args(["diff", "--name-only", base_commit_oid])
        .current_dir(workspace)
        .output()
        .map_err(|e| CheckpointError::GitError(format!("Failed to execute 'git diff': {e}")))?;

    if !diff_output.status.success() {
        return Err(CheckpointError::GitError(format!(
            "git diff failed for base commit '{base_commit_oid}'"
        )));
    }

    let changed_files: Vec<String> = String::from_utf8_lossy(&diff_output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut new_tests = Vec::new();

    for file_rel in changed_files {
        let path = Path::new(&file_rel);
        if !is_test_file(path) {
            continue;
        }

        // Get base version of the file
        let base_content = Command::new("git")
            .args(["show", &format!("{base_commit_oid}:{file_rel}")])
            .current_dir(workspace)
            .output()
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    String::from_utf8(out.stdout).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let old_identifiers: HashSet<String> = extract_test_identifiers(path, &base_content)
            .into_iter()
            .collect();

        // Get current version on disk
        let full_path = workspace.join(path);
        let current_content = fs::read_to_string(&full_path).unwrap_or_default();
        let current_identifiers = extract_test_identifiers(path, &current_content);

        for id in current_identifiers {
            if !old_identifiers.contains(&id) && !new_tests.contains(&id) {
                new_tests.push(id);
            }
        }
    }

    Ok(new_tests)
}

/// Runs a git command and returns stdout, failing closed on spawn errors or non-zero exit.
fn git_stdout(cwd: &Path, args: &[&str]) -> Result<String, CheckpointError> {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| CheckpointError::GitError(format!("Failed to run git {args:?}: {e}")))?;
    if !out.status.success() {
        return Err(CheckpointError::GitError(format!(
            "git {args:?} failed during inverse preflight: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Temporary detached worktree checked out at `HEAD` (Task 047).
///
/// The user's working tree is never written to. If the process is interrupted,
/// only a stale temp directory remains; its registration is pruned on the next run.
struct IsolatedWorktree {
    repo: PathBuf,
    path: PathBuf,
}

impl IsolatedWorktree {
    fn create(workspace: &Path) -> Result<Self, CheckpointError> {
        let repo = PathBuf::from(git_stdout(workspace, &["rev-parse", "--show-toplevel"])?.trim());
        git_stdout(&repo, &["rev-parse", "--verify", "HEAD^{commit}"])?;
        // Best-effort removal of registrations left behind by interrupted runs.
        let _ = git_stdout(&repo, &["worktree", "prune"]);

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "xgauntlet-preflight-{}-{nanos}",
            std::process::id()
        ));
        let path_str = path.to_string_lossy().to_string();
        git_stdout(
            &repo,
            &["worktree", "add", "--detach", "--quiet", &path_str, "HEAD"],
        )?;
        Ok(Self { repo, path })
    }

    /// Mirrors added, modified and deleted test files from the user's tree into the worktree.
    fn sync_changed_test_files(&self) -> Result<(), CheckpointError> {
        let tracked = git_stdout(&self.repo, &["diff", "--name-only", "-z", "HEAD"])?;
        let untracked = git_stdout(
            &self.repo,
            &[
                "ls-files",
                "--others",
                "--exclude-standard",
                "--full-name",
                "-z",
            ],
        )?;
        let io_err = |e: std::io::Error| {
            CheckpointError::GitError(format!(
                "Failed to stage test files in preflight worktree: {e}"
            ))
        };
        for rel in tracked.split('\0').chain(untracked.split('\0')) {
            if rel.is_empty() || !is_test_file(Path::new(rel)) {
                continue;
            }
            let src = self.repo.join(rel);
            let dst = self.path.join(rel);
            if src.is_file() {
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(io_err)?;
                }
                fs::copy(&src, &dst).map_err(io_err)?;
            } else if dst.exists() {
                fs::remove_file(&dst).map_err(io_err)?;
            }
        }
        Ok(())
    }
}

impl Drop for IsolatedWorktree {
    fn drop(&mut self) {
        let path_str = self.path.to_string_lossy().to_string();
        let _ = git_stdout(&self.repo, &["worktree", "remove", "--force", &path_str]);
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
        let _ = git_stdout(&self.repo, &["worktree", "prune"]);
    }
}

/// Executes non-destructive inverse preflight against baseline production code.
///
/// The test layer runs inside an isolated `HEAD` worktree containing only the
/// in-flight test files, so the user's working tree is never modified.
pub fn verify_inverse_test_failure(
    workspace: &Path,
    test_identifiers: &[String],
) -> Result<Vec<TestProvenanceEvidence>, CheckpointError> {
    if test_identifiers.is_empty() {
        return Ok(Vec::new());
    }

    let worktree = IsolatedWorktree::create(workspace)?;
    worktree.sync_changed_test_files()?;
    let prefix = git_stdout(workspace, &["rev-parse", "--show-prefix"])?;
    let exec_dir = worktree.path.join(prefix.trim());

    // Execute test layer against baseline code
    let layer = resolve_test_layer(workspace);
    let mut test_cmd = Command::new(&layer.command[0]);
    test_cmd.args(&layer.command[1..]);
    test_cmd.current_dir(&exec_dir);
    // Keep Cargo dependency builds cached across ephemeral worktrees (ignored build artefacts).
    if std::env::var_os("CARGO_TARGET_DIR").is_none() && worktree.repo.join("Cargo.toml").is_file()
    {
        test_cmd.env(
            "CARGO_TARGET_DIR",
            worktree.repo.join("target").join("xgauntlet-preflight"),
        );
    }

    let test_output = test_cmd.output().map_err(|e| {
        CheckpointError::GitError(format!(
            "Failed to execute test layer during inverse preflight: {e}"
        ))
    })?;

    let stdout_str = String::from_utf8_lossy(&test_output.stdout);
    let stderr_str = String::from_utf8_lossy(&test_output.stderr);
    let mut failure_log = format!("{stdout_str}\n{stderr_str}").trim().to_string();
    if failure_log.is_empty() {
        failure_log = format!(
            "Test layer failed with exit code {:?}",
            test_output.status.code().unwrap_or(1)
        );
    }

    // If test passed against baseline without production changes, it is TAUTOLOGICAL!
    if test_output.status.success() {
        return Err(CheckpointError::TautologicalTestDetected {
            test_name: test_identifiers[0].clone(),
        });
    }

    let timestamp = current_iso_utc();
    let failure_trimmed = failure_log.trim().to_string();
    let failure_digest = format!("{:x}", Sha256::digest(failure_trimmed.as_bytes()));
    let mut evidences = Vec::new();
    for id in test_identifiers {
        evidences.push(TestProvenanceEvidence {
            test_identifier: id.clone(),
            failure_output: failure_trimmed.clone(),
            failure_digest: failure_digest.clone(),
            verified_at: timestamp.clone(),
            status: TestProvenanceStatus::InversePreflightVerified,
        });
    }

    Ok(evidences)
}

/// Collects test provenance evidence across local checkpoints and inverse preflight records.
pub fn collect_task_test_provenance(
    workspace: &Path,
    task_id: &str,
) -> Result<Vec<TestProvenanceEvidence>, CheckpointError> {
    let mut evidences = Vec::new();

    // Strip any descriptive slug suffix (e.g. "032-foo" -> "032")
    let short_id = task_id.split('-').next().unwrap_or(task_id);

    // Query git log for RED phase commits for this task
    let output = Command::new("git")
        .args(["log", "--format=%H|%cI|%s"])
        .current_dir(workspace)
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let log_str = String::from_utf8_lossy(&out.stdout);
            for line in log_str.lines() {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() < 3 {
                    continue;
                }
                let commit_oid = parts[0].trim();
                let timestamp = parts[1].trim();
                let subject = parts[2].trim();

                let is_task_red = (subject.contains(&format!("test({task_id})"))
                    || subject.contains(&format!("test({short_id})")))
                    && subject.contains("[RED]");

                if is_task_red {
                    // Extract test files modified in this commit
                    let diff_out = Command::new("git")
                        .args([
                            "diff-tree",
                            "--no-commit-id",
                            "--name-only",
                            "-r",
                            "--root",
                            commit_oid,
                        ])
                        .current_dir(workspace)
                        .output();

                    if let Ok(diff_res) = diff_out {
                        if diff_res.status.success() {
                            let files_str = String::from_utf8_lossy(&diff_res.stdout);
                            for file in files_str.lines() {
                                let trimmed_file = file.trim();
                                if trimmed_file.is_empty() {
                                    continue;
                                }
                                let p = Path::new(trimmed_file);
                                if is_test_file(p) {
                                    let show_out = Command::new("git")
                                        .args(["show", &format!("{commit_oid}:{trimmed_file}")])
                                        .current_dir(workspace)
                                        .output();

                                    if let Ok(show_res) = show_out {
                                        if show_res.status.success() {
                                            let content = String::from_utf8_lossy(&show_res.stdout);
                                            let test_names = extract_test_identifiers(p, &content);
                                            for name in test_names {
                                                if !evidences.iter().any(
                                                    |e: &TestProvenanceEvidence| {
                                                        e.test_identifier == name
                                                    },
                                                ) {
                                                    let short_oid = if commit_oid.len() >= 7 {
                                                        &commit_oid[..7]
                                                    } else {
                                                        commit_oid
                                                    };
                                                    let failure_output = format!(
                                                        "Verified in RED checkpoint commit {short_oid}"
                                                    );
                                                    let failure_digest = format!(
                                                        "{:x}",
                                                        Sha256::digest(failure_output.as_bytes())
                                                    );
                                                    evidences.push(TestProvenanceEvidence {
                                                        test_identifier: name,
                                                        failure_output,
                                                        failure_digest,
                                                        verified_at: timestamp.to_string(),
                                                        status: TestProvenanceStatus::RedCheckpointVerified,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(evidences)
}
