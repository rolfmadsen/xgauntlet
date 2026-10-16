//! Architecture check implementation and workspace traversal.

use std::fs;
use std::path::Path;

use super::models::{
    ArchitectureConfig, ArchitectureFinding, ArchitectureFindingCategory, ArchitectureReport,
};

/// Runs architectural analysis over a workspace directory using the provided configuration.
pub fn run_architecture_check(
    workspace: &Path,
    config: &ArchitectureConfig,
) -> Result<ArchitectureReport, std::io::Error> {
    let mut report = ArchitectureReport::new();
    scan_dir(workspace, workspace, config, &mut report)?;
    Ok(report)
}

fn scan_dir(
    root: &Path,
    current: &Path,
    config: &ArchitectureConfig,
    report: &mut ArchitectureReport,
) -> Result<(), std::io::Error> {
    if !current.is_dir() {
        return Ok(());
    }

    let entries = match fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if path.is_dir() {
            // Skip common build, cache, and vendor directories
            if matches!(
                name.as_ref(),
                "target"
                    | "node_modules"
                    | ".git"
                    | ".system_generated"
                    | ".gemini"
                    | "dist"
                    | "build"
                    | "wasm"
                    | ".cargo"
            ) {
                continue;
            }
            scan_dir(root, &path, config, report)?;
        } else if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(
                ext,
                "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "toml" | "json"
            ) {
                // Only inspect code/config files
                if matches!(ext, "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go") {
                    inspect_file(root, &path, config, report)?;
                }
            }
        }
    }

    Ok(())
}

fn inspect_file(
    root: &Path,
    path: &Path,
    config: &ArchitectureConfig,
    report: &mut ArchitectureReport,
) -> Result<(), std::io::Error> {
    let rel_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // Skip binary or unreadable files
    };

    report.files_scanned += 1;
    let line_count = content.lines().count();

    let is_test_file = rel_path.starts_with("tests/")
        || rel_path.contains("/tests/")
        || rel_path.contains("__tests__")
        || rel_path.ends_with("_test.rs")
        || rel_path.ends_with(".test.ts")
        || rel_path.ends_with(".spec.ts")
        || rel_path.ends_with("_test.py")
        || rel_path.ends_with("_test.go");

    // 1. Line count evaluation with exemptions
    let exemption = config
        .exemptions
        .iter()
        .find(|ex| ex.path == rel_path || rel_path.ends_with(&ex.path));

    match exemption {
        Some(ex) => {
            if let Some(ceiling) = ex.max_lines {
                if line_count > ceiling {
                    report.add_finding(ArchitectureFinding {
                        file_path: rel_path.clone(),
                        line_number: None,
                        category: ArchitectureFindingCategory::ExemptionCeilingExceeded,
                        message: format!(
                            "File '{}' has {} lines, which exceeds exemption ceiling of {} lines (Reason: '{}')",
                            rel_path, line_count, ceiling, ex.reason
                        ),
                        remediation_hint: "Decompose this file into smaller slices or refactor logic into peer modules.".into(),
                    });
                }
            }
        }
        None => {
            if is_test_file {
                if let Some(max) = config.max_test_file_lines {
                    if line_count > max {
                        report.add_finding(ArchitectureFinding {
                            file_path: rel_path.clone(),
                            line_number: None,
                            category: ArchitectureFindingCategory::TestFileExceedsMaxLines,
                            message: format!(
                                "Test file '{}' has {} lines, which exceeds max_test_file_lines limit of {}",
                                rel_path, line_count, max
                            ),
                            remediation_hint: "Decompose monolithic tests into feature-bound test suites (e.g. tests/<feature>_test.rs).".into(),
                        });
                    }
                }
            } else if let Some(max) = config.max_file_lines {
                if line_count > max {
                    report.add_finding(ArchitectureFinding {
                        file_path: rel_path.clone(),
                        line_number: None,
                        category: ArchitectureFindingCategory::FileExceedsMaxLines,
                        message: format!(
                            "Production file '{}' has {} lines, which exceeds max_file_lines limit of {}",
                            rel_path, line_count, max
                        ),
                        remediation_hint: "Decompose monolithic file into vertical feature slices or dedicated domain components.".into(),
                    });
                }
            }
        }
    }

    // 2. Token and Import evaluation (strictly for production source code)
    if !is_test_file {
        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;

            // Check forbidden tokens
            for token in &config.forbidden_tokens {
                if line.contains(token) {
                    report.add_finding(ArchitectureFinding {
                    file_path: rel_path.clone(),
                    line_number: Some(line_num),
                    category: ArchitectureFindingCategory::ForbiddenTokenDetected,
                    message: format!(
                        "Forbidden token '{}' detected at {}:{}",
                        token, rel_path, line_num
                    ),
                    remediation_hint: "Use standardized Design Tokens or centralized abstractions instead of raw tokens.".into(),
                });
                }
            }

            // Check forbidden imports
            for import_pattern in &config.forbidden_imports {
                if line.contains(import_pattern) {
                    let trimmed = line.trim();
                    if trimmed.starts_with("use ")
                        || trimmed.starts_with("import ")
                        || trimmed.starts_with("from ")
                    {
                        report.add_finding(ArchitectureFinding {
                        file_path: rel_path.clone(),
                        line_number: Some(line_num),
                        category: ArchitectureFindingCategory::ForbiddenImportDetected,
                        message: format!(
                            "Forbidden import '{}' detected at {}:{}",
                            import_pattern, rel_path, line_num
                        ),
                        remediation_hint: "Decouple dependency and respect architectural package-by-feature module boundaries.".into(),
                    });
                    }
                }
            }
        }
    }

    Ok(())
}
