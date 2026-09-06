//! Diagnostic models for actionable compiler, linter, and specification findings.

use serde::{Deserialize, Serialize};

/// Categorized diagnostic finding types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingType {
    LintError,
    TypeMismatch,
    TestFailure,
    PropertyViolation,
    MutantSurvived,
    DriftDetected,
    InvariantViolation,
    GeneralError,
}

/// A single structured diagnostic finding with actionable remediation guidance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticFinding {
    pub finding_type: FindingType,
    pub tool_name: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub column_number: Option<usize>,
    pub message: String,
    pub remediation_hint: String,
    pub raw_context: String,
}

impl DiagnosticFinding {
    /// Creates a new basic diagnostic finding without line or raw context.
    pub fn new(
        finding_type: FindingType,
        tool_name: impl Into<String>,
        file_path: impl Into<String>,
        message: impl Into<String>,
        remediation_hint: impl Into<String>,
    ) -> Self {
        Self {
            finding_type,
            tool_name: tool_name.into(),
            file_path: file_path.into(),
            line_number: None,
            column_number: None,
            message: message.into(),
            remediation_hint: remediation_hint.into(),
            raw_context: String::new(),
        }
    }

    /// Sets the line number for this finding.
    pub fn with_line(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }
}

/// Structured report of diagnostic findings extracted from a verification layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub layer_name: String,
    pub passed: bool,
    pub exit_code: i32,
    pub findings: Vec<DiagnosticFinding>,
    pub summary: String,
}
