//! Extractors for type checker outputs (Cargo check, TSC, Pyright, Mypy).

use crate::features::diagnostics::models::{DiagnosticFinding, FindingType};

/// Extracts Cargo check findings.
pub fn extract_cargo_check_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("error[E") || trimmed.starts_with("error:") {
            let message = trimmed.to_string();
            let mut file_path = "src/lib.rs".to_string();
            let mut line_number = None;
            let mut column_number = None;

            for next_line in lines.iter().skip(i + 1).take(5) {
                let trimmed_next = next_line.trim();
                if let Some(stripped) = trimmed_next.strip_prefix("-->") {
                    let loc_part = stripped.trim();
                    let parts: Vec<&str> = loc_part.split(':').collect();
                    if !parts.is_empty() {
                        file_path = parts[0].to_string();
                    }
                    if parts.len() > 1 {
                        line_number = parts[1].parse::<usize>().ok();
                    }
                    if parts.len() > 2 {
                        column_number = parts[2].parse::<usize>().ok();
                    }
                    break;
                }
            }

            let mut finding = DiagnosticFinding::new(
                FindingType::TypeMismatch,
                "cargo-check",
                file_path,
                message,
                "Resolve Rust compiler type mismatch or unresolved symbol.".to_string(),
            );
            finding.line_number = line_number;
            finding.column_number = column_number;
            finding.raw_context = trimmed.to_string();
            findings.push(finding);
        }
    }

    findings
}

/// Extracts TypeScript compiler (tsc) findings.
pub fn extract_tsc_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        // Format: file.ts(12,5): error TS2322: Type 'string' is not assignable to type 'number'.
        if trimmed.contains(": error TS") {
            if let Some((loc, msg)) = trimmed.split_once(": error ") {
                let mut file_path = loc.to_string();
                let mut line_number = None;
                let mut column_number = None;

                if let Some((f, pos)) = loc.split_once('(') {
                    file_path = f.trim().to_string();
                    let pos_clean = pos.trim_end_matches(')');
                    if let Some((l, c)) = pos_clean.split_once(',') {
                        line_number = l.trim().parse::<usize>().ok();
                        column_number = c.trim().parse::<usize>().ok();
                    }
                }

                let mut finding = DiagnosticFinding::new(
                    FindingType::TypeMismatch,
                    "tsc",
                    file_path,
                    msg.to_string(),
                    "Fix TypeScript type contract violation.".to_string(),
                );
                finding.line_number = line_number;
                finding.column_number = column_number;
                finding.raw_context = trimmed.to_string();
                findings.push(finding);
            }
        }
    }
    findings
}

/// Extracts Pyright / Mypy type checker findings.
pub fn extract_pyright_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains(" - error: ") || trimmed.contains(": error: ") {
            let parts: Vec<&str> = trimmed.split(':').collect();
            if parts.len() >= 3 {
                let file_path = parts[0].trim().to_string();
                let line_number = parts[1].trim().parse::<usize>().ok();
                let message = parts[2..].join(":").trim().to_string();

                let mut finding = DiagnosticFinding::new(
                    FindingType::TypeMismatch,
                    "pyright",
                    file_path,
                    message,
                    "Fix Python static typing annotation mismatch.".to_string(),
                );
                finding.line_number = line_number;
                finding.raw_context = trimmed.to_string();
                findings.push(finding);
            }
        }
    }
    findings
}
