//! Git repository and version control diagnostic checks.

use crate::features::doctor::models::{DoctorCategory, DoctorCheckItem, DoctorCheckStatus};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Inspects Git binary availability, repository integrity, user identity, working tree state, and push.followTags.
pub fn check_git(workspace: &Path) -> Vec<DoctorCheckItem> {
    let mut checks = Vec::new();

    // 1. Git binary in PATH
    let start = Instant::now();
    let git_cmd = Command::new("git").arg("--version").output();
    let duration = start.elapsed().as_millis() as u64;

    let has_git = match git_cmd {
        Ok(out) if out.status.success() => {
            let ver_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            checks.push(DoctorCheckItem {
                name: "git_binary".to_string(),
                category: DoctorCategory::Git,
                status: DoctorCheckStatus::Pass,
                message: ver_str.clone(),
                detail: Some(format!("Available: {ver_str}")),
                remediation: None,
                duration_ms: duration,
            });
            true
        }
        _ => {
            checks.push(DoctorCheckItem {
                name: "git_binary".to_string(),
                category: DoctorCategory::Git,
                status: DoctorCheckStatus::Fail,
                message: "Git binary not found in PATH".to_string(),
                detail: None,
                remediation: Some(
                    "Install Git (e.g. 'apt install git', 'brew install git') and ensure it is in PATH."
                        .to_string(),
                ),
                duration_ms: duration,
            });
            false
        }
    };

    if !has_git {
        return checks;
    }

    // 2. Git repository verification
    let start = Instant::now();
    let is_repo_out = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output();
    let duration = start.elapsed().as_millis() as u64;

    let is_repo = match is_repo_out {
        Ok(out) if out.status.success() => {
            // Retrieve current branch or commit
            let branch_out = Command::new("git")
                .arg("-C")
                .arg(workspace)
                .arg("rev-parse")
                .arg("--abbrev-ref")
                .arg("HEAD")
                .output();
            let branch = branch_out
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_else(|| "HEAD".to_string());
            let branch_clean = branch.trim();

            checks.push(DoctorCheckItem {
                name: "git_repository".to_string(),
                category: DoctorCategory::Git,
                status: DoctorCheckStatus::Pass,
                message: format!("Git worktree detected (branch: {branch_clean})"),
                detail: Some(format!("Workspace root: {}", workspace.display())),
                remediation: None,
                duration_ms: duration,
            });
            true
        }
        _ => {
            checks.push(DoctorCheckItem {
                name: "git_repository".to_string(),
                category: DoctorCategory::Git,
                status: DoctorCheckStatus::Warn,
                message: "Workspace is not inside a Git repository worktree".to_string(),
                detail: Some(format!("Path: {}", workspace.display())),
                remediation: Some("Initialize a Git repository using 'git init'.".to_string()),
                duration_ms: duration,
            });
            false
        }
    };

    if !is_repo {
        return checks;
    }

    // 3. Git user configuration (user.name, user.email)
    let start = Instant::now();
    let name_out = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("config")
        .arg("user.name")
        .output();
    let email_out = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("config")
        .arg("user.email")
        .output();
    let duration = start.elapsed().as_millis() as u64;

    let user_name = name_out
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let user_email = email_out
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    match (user_name, user_email) {
        (Some(name), Some(email)) => {
            checks.push(DoctorCheckItem {
                name: "git_user".to_string(),
                category: DoctorCategory::Git,
                status: DoctorCheckStatus::Pass,
                message: format!("{name} <{email}>"),
                detail: Some(format!("user.name={name}, user.email={email}")),
                remediation: None,
                duration_ms: duration,
            });
        }
        (Some(name), None) => {
            checks.push(DoctorCheckItem {
                name: "git_user".to_string(),
                category: DoctorCategory::Git,
                status: DoctorCheckStatus::Warn,
                message: format!("Configured name '{name}', but user.email is missing"),
                detail: None,
                remediation: Some(
                    "Configure git email via 'git config user.email <email>'.".to_string(),
                ),
                duration_ms: duration,
            });
        }
        (None, Some(email)) => {
            checks.push(DoctorCheckItem {
                name: "git_user".to_string(),
                category: DoctorCategory::Git,
                status: DoctorCheckStatus::Warn,
                message: format!("Configured email '{email}', but user.name is missing"),
                detail: None,
                remediation: Some(
                    "Configure git user name via 'git config user.name <name>'.".to_string(),
                ),
                duration_ms: duration,
            });
        }
        (None, None) => {
            checks.push(DoctorCheckItem {
                name: "git_user".to_string(),
                category: DoctorCategory::Git,
                status: DoctorCheckStatus::Warn,
                message: "Git identity not configured (user.name and user.email are unset)".to_string(),
                detail: None,
                remediation: Some(
                    "Configure git identity: 'git config user.name <name>' and 'git config user.email <email>'."
                        .to_string(),
                ),
                duration_ms: duration,
            });
        }
    }

    // 4. push.followTags configuration (ADR 0003 compliance)
    let start = Instant::now();
    let follow_tags_out = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("config")
        .arg("push.followTags")
        .output();
    let duration = start.elapsed().as_millis() as u64;

    let is_follow_tags = follow_tags_out
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if is_follow_tags {
        checks.push(DoctorCheckItem {
            name: "git_push_follow_tags".to_string(),
            category: DoctorCategory::Git,
            status: DoctorCheckStatus::Pass,
            message: "push.followTags is enabled (ADR 0003 compliant)".to_string(),
            detail: Some("git config push.followTags = true".to_string()),
            remediation: None,
            duration_ms: duration,
        });
    } else {
        checks.push(DoctorCheckItem {
            name: "git_push_follow_tags".to_string(),
            category: DoctorCategory::Git,
            status: DoctorCheckStatus::Warn,
            message: "push.followTags is not enabled".to_string(),
            detail: Some(
                "ADR 0003 recommends push.followTags true so annotated release tags follow standard push."
                    .to_string(),
            ),
            remediation: Some("Run 'git config push.followTags true' to enable tag following.".to_string()),
            duration_ms: duration,
        });
    }

    // 5. Working tree status
    let start = Instant::now();
    let status_out = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("status")
        .arg("--porcelain")
        .output();
    let duration = start.elapsed().as_millis() as u64;

    match status_out {
        Ok(out) if out.status.success() => {
            let changes = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = changes.lines().filter(|l| !l.trim().is_empty()).collect();
            if lines.is_empty() {
                checks.push(DoctorCheckItem {
                    name: "git_cleanliness".to_string(),
                    category: DoctorCategory::Git,
                    status: DoctorCheckStatus::Pass,
                    message: "Working tree is clean".to_string(),
                    detail: None,
                    remediation: None,
                    duration_ms: duration,
                });
            } else {
                let modified = lines.iter().filter(|l| !l.starts_with("??")).count();
                let untracked = lines.iter().filter(|l| l.starts_with("??")).count();
                checks.push(DoctorCheckItem {
                    name: "git_cleanliness".to_string(),
                    category: DoctorCategory::Git,
                    status: DoctorCheckStatus::Info,
                    message: format!(
                        "Working tree has {modified} tracked change(s) and {untracked} untracked file(s)"
                    ),
                    detail: Some(format!("Total entries in git status: {}", lines.len())),
                    remediation: None,
                    duration_ms: duration,
                });
            }
        }
        _ => {}
    }

    checks
}
