//! Toolchain, compiler, linter, and runtime diagnostic checks.

use crate::features::config::detect_stack;
use crate::features::doctor::models::{DoctorCategory, DoctorCheckItem, DoctorCheckStatus};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn probe_command(
    cmd: &str,
    args: &[&str],
    name: &str,
    remediation_on_fail: Option<&str>,
    is_mandatory: bool,
) -> DoctorCheckItem {
    let start = Instant::now();
    #[allow(unused_mut)]
    let mut res = Command::new(cmd).args(args).output();

    #[cfg(windows)]
    if res.is_err() || matches!(&res, Ok(out) if !out.status.success()) {
        if let Ok(out) = Command::new(format!("{cmd}.cmd")).args(args).output() {
            if out.status.success() {
                res = Ok(out);
            }
        } else if let Ok(out) = Command::new(format!("{cmd}.bat")).args(args).output() {
            if out.status.success() {
                res = Ok(out);
            }
        } else if let Ok(out) = Command::new(format!("{cmd}.exe")).args(args).output() {
            if out.status.success() {
                res = Ok(out);
            }
        } else if let Ok(out) = Command::new("cmd").args(["/C", cmd]).args(args).output() {
            if out.status.success() {
                res = Ok(out);
            }
        }
    }

    let duration = start.elapsed().as_millis() as u64;

    match res {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let ver = if !stdout.is_empty() {
                stdout.lines().next().unwrap_or(&stdout).to_string()
            } else if !stderr.is_empty() {
                stderr.lines().next().unwrap_or(&stderr).to_string()
            } else {
                format!("{cmd} available")
            };

            DoctorCheckItem {
                name: name.to_string(),
                category: DoctorCategory::Toolchains,
                status: DoctorCheckStatus::Pass,
                message: ver.clone(),
                detail: Some(format!("Command: {} {}", cmd, args.join(" "))),
                remediation: None,
                duration_ms: duration,
            }
        }
        _ => {
            let status = if is_mandatory {
                DoctorCheckStatus::Fail
            } else {
                DoctorCheckStatus::Warn
            };

            DoctorCheckItem {
                name: name.to_string(),
                category: DoctorCategory::Toolchains,
                status,
                message: format!("Command '{cmd}' not found or failed"),
                detail: None,
                remediation: remediation_on_fail.map(|r| r.to_string()),
                duration_ms: duration,
            }
        }
    }
}

/// Inspects workspace toolchains based on auto-detected stack.
pub fn check_toolchains(workspace: &Path) -> Vec<DoctorCheckItem> {
    let mut checks = Vec::new();

    // 1. Detect Stack
    let start = Instant::now();
    let stack = detect_stack(workspace);
    let duration = start.elapsed().as_millis() as u64;

    checks.push(DoctorCheckItem {
        name: "detected_stack".to_string(),
        category: DoctorCategory::Toolchains,
        status: DoctorCheckStatus::Pass,
        message: format!("Detected stack profile: {stack}"),
        detail: Some(format!("Workspace root: {}", workspace.display())),
        remediation: None,
        duration_ms: duration,
    });

    match stack {
        "rust" => {
            checks.push(probe_command(
                "cargo",
                &["--version"],
                "tool_cargo",
                Some("Install Rust and Cargo via https://rustup.rs"),
                true,
            ));
            checks.push(probe_command(
                "rustc",
                &["--version"],
                "tool_rustc",
                Some("Install rustc via 'rustup toolchain install stable'"),
                true,
            ));
            checks.push(probe_command(
                "cargo",
                &["clippy", "--version"],
                "tool_clippy",
                Some("Install clippy via 'rustup component add clippy'"),
                false,
            ));
            checks.push(probe_command(
                "cargo",
                &["fmt", "--version"],
                "tool_rustfmt",
                Some("Install rustfmt via 'rustup component add rustfmt'"),
                false,
            ));
        }
        "node" | "typescript" => {
            checks.push(probe_command(
                "node",
                &["--version"],
                "tool_node",
                Some("Install Node.js (https://nodejs.org)"),
                true,
            ));
            checks.push(probe_command(
                "npm",
                &["--version"],
                "tool_npm",
                Some("Install npm, pnpm, or yarn"),
                true,
            ));
        }
        "python" => {
            let py_check = probe_command(
                "python3",
                &["--version"],
                "tool_python3",
                Some("Install Python 3 (https://www.python.org)"),
                false,
            );
            if py_check.status == DoctorCheckStatus::Pass {
                checks.push(py_check);
            } else {
                checks.push(probe_command(
                    "python",
                    &["--version"],
                    "tool_python",
                    Some("Install Python 3 and ensure it is in PATH"),
                    true,
                ));
            }
            checks.push(probe_command(
                "pytest",
                &["--version"],
                "tool_pytest",
                Some("Install pytest via 'pip install pytest'"),
                false,
            ));
            checks.push(probe_command(
                "ruff",
                &["--version"],
                "tool_ruff",
                Some("Install ruff via 'pip install ruff'"),
                false,
            ));
        }
        "go" => {
            checks.push(probe_command(
                "go",
                &["version"],
                "tool_go",
                Some("Install Go (https://go.dev)"),
                true,
            ));
            checks.push(probe_command(
                "golangci-lint",
                &["--version"],
                "tool_golangci_lint",
                Some("Install golangci-lint (https://golangci-lint.run)"),
                false,
            ));
        }
        _ => {
            // General multi-stack fallback: test if cargo or node is available
            checks.push(probe_command(
                "cargo",
                &["--version"],
                "tool_cargo",
                Some("Install Cargo if working with Rust"),
                false,
            ));
            checks.push(probe_command(
                "node",
                &["--version"],
                "tool_node",
                Some("Install Node.js if working with JavaScript/TypeScript"),
                false,
            ));
        }
    }

    checks
}
