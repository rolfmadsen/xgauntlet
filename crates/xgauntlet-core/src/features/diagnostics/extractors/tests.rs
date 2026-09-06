//! Extractors for test runners (Cargo test, Pytest, Vitest).

use crate::features::diagnostics::models::{DiagnosticFinding, FindingType};

/// Extracts Cargo test failure findings.
pub fn extract_cargo_test_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Detect: thread '...' panicked at src/lib.rs:42:5: assertion `left == right` failed
        if trimmed.contains("panicked at") {
            let mut file_path = "tests".to_string();
            let mut line_number = None;
            let mut column_number = None;

            if let Some((_prefix, rest)) = trimmed.split_once("panicked at ") {
                if let Some((loc, msg)) = rest.split_once(':') {
                    let loc_parts: Vec<&str> = loc.split(':').collect();
                    if !loc_parts.is_empty() {
                        file_path = loc_parts[0].trim_matches('\'').to_string();
                    }
                    if loc_parts.len() > 1 {
                        line_number = loc_parts[1].parse::<usize>().ok();
                    }
                    if loc_parts.len() > 2 {
                        column_number = loc_parts[2].parse::<usize>().ok();
                    }

                    let message = format!("Test assertion failure: {}", msg.trim());
                    let mut finding = DiagnosticFinding::new(
                        FindingType::TestFailure,
                        "cargo-test",
                        file_path,
                        message,
                        "Update code or assertions to fulfill task contract.".to_string(),
                    );
                    finding.line_number = line_number;
                    finding.column_number = column_number;
                    finding.raw_context = trimmed.to_string();
                    findings.push(finding);
                    continue;
                }
            }
        }

        // Detect failed test header: test tests::foo ... FAILED
        if trimmed.starts_with("test ") && trimmed.ends_with("... FAILED") {
            let test_name = trimmed
                .trim_start_matches("test ")
                .trim_end_matches("... FAILED")
                .trim();
            // Check next lines for detailed panic message
            let mut detail = format!("Test '{}' failed", test_name);
            let mut file_path = "tests".to_string();
            let mut line_num = None;

            for next_line in lines.iter().skip(i + 1).take(15) {
                let tn = next_line.trim();
                if tn.contains("panicked at ") {
                    if let Some((_, loc_msg)) = tn.split_once("panicked at ") {
                        let parts: Vec<&str> = loc_msg.split(':').collect();
                        if !parts.is_empty() {
                            file_path = parts[0].trim_matches('\'').to_string();
                        }
                        if parts.len() > 1 {
                            line_num = parts[1].parse::<usize>().ok();
                        }
                        if parts.len() > 3 {
                            detail = parts[3..].join(":").trim().to_string();
                        }
                    }
                    break;
                }
            }

            // Only add if not already captured
            if !findings
                .iter()
                .any(|f| f.file_path == file_path && f.line_number == line_num)
            {
                let mut finding = DiagnosticFinding::new(
                    FindingType::TestFailure,
                    "cargo-test",
                    file_path,
                    detail,
                    "Fix failing test assertion.".to_string(),
                );
                finding.line_number = line_num;
                finding.raw_context = trimmed.to_string();
                findings.push(finding);
            }
        }
    }

    findings
}

/// Extracts Pytest failure findings.
pub fn extract_pytest_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        // Format: FAILED tests/test_foo.py::test_bar - AssertionError: ...
        if trimmed.starts_with("FAILED ") {
            let rest = trimmed.trim_start_matches("FAILED ").trim();
            let parts: Vec<&str> = rest.split(" - ").collect();
            let test_id = parts[0];
            let msg = if parts.len() > 1 {
                parts[1]
            } else {
                "Test failed"
            };
            let file_path = test_id.split("::").next().unwrap_or(test_id).to_string();

            findings.push(DiagnosticFinding::new(
                FindingType::TestFailure,
                "pytest",
                file_path,
                msg.to_string(),
                "Fix failing test in pytest suite.".to_string(),
            ));
        }
    }
    findings
}

/// Extracts Vitest / Jest failure findings.
pub fn extract_vitest_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("FAIL ") {
            let file_path = trimmed.trim_start_matches("FAIL ").trim().to_string();
            findings.push(DiagnosticFinding::new(
                FindingType::TestFailure,
                "vitest",
                file_path,
                "Test suite failed execution.".to_string(),
                "Inspect Vitest report and satisfy failed assertion.".to_string(),
            ));
        }
    }
    findings
}
