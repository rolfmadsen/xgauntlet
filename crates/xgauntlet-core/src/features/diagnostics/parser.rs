//! Unified diagnostic parser extracting structured findings from tool and compiler outputs.

use super::extractors::{
    extract_cargo_check_findings, extract_cargo_mutants_findings, extract_cargo_test_findings,
    extract_clippy_findings, extract_eslint_findings, extract_hypothesis_findings,
    extract_mutants_py_findings, extract_proptest_findings, extract_pytest_findings,
    extract_ruff_findings, extract_tsc_findings, extract_vitest_findings,
};
use super::models::{DiagnosticFinding, DiagnosticReport, FindingType};

/// Unified orchestrator for parsing compiler, linter, and test outputs into structured findings.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticParser;

impl DiagnosticParser {
    pub fn new() -> Self {
        Self
    }

    /// Extracts structured findings from a layer command's output.
    pub fn extract_findings(
        &self,
        layer_name: &str,
        command: &[String],
        exit_code: i32,
        output: &str,
    ) -> Vec<DiagnosticFinding> {
        let mut findings = Vec::new();
        let cmd_str = command.join(" ").to_lowercase();

        // 1. Linters
        if cmd_str.contains("clippy") || output.contains("warning:") || output.contains("error[E") {
            findings.extend(extract_clippy_findings(output));
        }
        if cmd_str.contains("ruff") {
            findings.extend(extract_ruff_findings(output));
        }
        if cmd_str.contains("eslint") || cmd_str.contains("biome") {
            findings.extend(extract_eslint_findings(output));
        }

        // 2. Type Checkers
        if (cmd_str.contains("cargo") && cmd_str.contains("check"))
            || (findings.is_empty() && output.contains("error[E"))
        {
            findings.extend(extract_cargo_check_findings(output));
        }
        if cmd_str.contains("tsc") || output.contains("error TS") {
            findings.extend(extract_tsc_findings(output));
        }

        // 3. Test Runners & Invariants
        if cmd_str.contains("test")
            || output.contains("panicked at")
            || output.contains("... FAILED")
        {
            findings.extend(extract_cargo_test_findings(output));
            findings.extend(extract_proptest_findings(output));
        }
        if cmd_str.contains("pytest") || output.contains("FAILED ") {
            findings.extend(extract_pytest_findings(output));
            findings.extend(extract_hypothesis_findings(output));
        }
        if cmd_str.contains("vitest") || cmd_str.contains("jest") {
            findings.extend(extract_vitest_findings(output));
        }

        // 4. Mutation Testing
        if cmd_str.contains("mutants") || output.contains("MISSED ") || output.contains("SURVIVED ")
        {
            findings.extend(extract_cargo_mutants_findings(output));
            findings.extend(extract_mutants_py_findings(output));
        }

        // 5. General Fallback if layer failed but no specific findings were captured
        if findings.is_empty() && exit_code != 0 {
            let first_lines: Vec<&str> = output
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .take(3)
                .collect();
            let msg = if !first_lines.is_empty() {
                first_lines.join(" | ")
            } else {
                format!("Layer '{layer_name}' failed with exit code {exit_code}")
            };
            findings.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                layer_name,
                "workspace",
                msg,
                "Check full command output logs for root cause remediation.".to_string(),
            ));
        }

        findings
    }

    /// Parses a layer output into a full `DiagnosticReport`.
    pub fn parse_layer_output(
        &self,
        layer_name: &str,
        command: &[String],
        exit_code: i32,
        output: &str,
    ) -> DiagnosticReport {
        let passed = exit_code == 0;
        let findings = self.extract_findings(layer_name, command, exit_code, output);
        let summary = if passed {
            format!("Layer '{layer_name}' passed successfully.")
        } else {
            format!(
                "Layer '{layer_name}' failed with exit code {exit_code} ({} findings).",
                findings.len()
            )
        };

        DiagnosticReport {
            layer_name: layer_name.to_string(),
            passed,
            exit_code,
            findings,
            summary,
        }
    }
}
