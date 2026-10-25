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
    validate_exemptions(workspace, config, &mut report);
    scan_dir(workspace, workspace, config, &mut report)?;
    Ok(report)
}

fn validate_exemptions(
    workspace: &Path,
    config: &ArchitectureConfig,
    report: &mut ArchitectureReport,
) {
    for ex in &config.exemptions {
        let trimmed_adr = ex.adr.trim();
        if trimmed_adr.is_empty() {
            report.add_finding(ArchitectureFinding {
                file_path: ex.path.clone(),
                line_number: None,
                category: ArchitectureFindingCategory::ExemptionMissingAdr,
                message: format!(
                    "Exemption for '{}' is missing mandatory 'adr' field. All architectural exemptions must reference an accepted ADR.",
                    ex.path
                ),
                remediation_hint: "Add 'adr = \"docs/adr/<adr-file>.md\"' to [[architecture.exemptions]] in gauntlet.toml pointing to an accepted ADR.".into(),
            });
            continue;
        }

        let adr_path = if Path::new(trimmed_adr).is_absolute() {
            std::path::PathBuf::from(trimmed_adr)
        } else {
            workspace.join(trimmed_adr)
        };

        if !adr_path.is_file() {
            report.add_finding(ArchitectureFinding {
                file_path: ex.path.clone(),
                line_number: None,
                category: ArchitectureFindingCategory::ExemptionAdrNotFound,
                message: format!(
                    "Exemption for '{}' references non-existent ADR file '{}'.",
                    ex.path, trimmed_adr
                ),
                remediation_hint: format!(
                    "Ensure ADR file '{}' exists and is committed to docs/adr/.",
                    trimmed_adr
                ),
            });
            continue;
        }

        let adr_content = match fs::read_to_string(&adr_path) {
            Ok(c) => c,
            Err(e) => {
                report.add_finding(ArchitectureFinding {
                    file_path: ex.path.clone(),
                    line_number: None,
                    category: ArchitectureFindingCategory::ExemptionAdrNotFound,
                    message: format!(
                        "Exemption for '{}' failed to read ADR file '{}': {}",
                        ex.path, trimmed_adr, e
                    ),
                    remediation_hint: "Ensure ADR file is readable.".into(),
                });
                continue;
            }
        };

        let status = extract_adr_status(&adr_content);
        let status_lower = status.to_ascii_lowercase();
        if status_lower != "accepted" && status_lower != "stable" {
            report.add_finding(ArchitectureFinding {
                file_path: ex.path.clone(),
                line_number: None,
                category: ArchitectureFindingCategory::ExemptionAdrNotAccepted,
                message: format!(
                    "Exemption for '{}' references ADR '{}' with status '{}'. Only 'accepted' or 'stable' ADRs may grant architectural exemptions.",
                    ex.path, trimmed_adr, status
                ),
                remediation_hint: format!(
                    "Update frontmatter of ADR '{}' to 'status: accepted' once the architectural decision has been approved.",
                    trimmed_adr
                ),
            });
        }
    }
}

fn extract_adr_status(content: &str) -> String {
    if let Ok((Some(meta), _)) = crate::features::tasks::okf::parse_frontmatter(content) {
        if let Some(s) = meta.status {
            return s.trim().to_string();
        }
    }
    let mut in_frontmatter = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            } else {
                in_frontmatter = true;
                continue;
            }
        }
        if in_frontmatter && trimmed.starts_with("status:") {
            let val = trimmed.trim_start_matches("status:").trim();
            return val.trim_matches('\'').trim_matches('"').trim().to_string();
        }
    }
    "unknown".to_string()
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
