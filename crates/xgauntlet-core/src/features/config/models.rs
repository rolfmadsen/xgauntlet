//! Domain models and error types for declarative gauntlet configuration.

use std::path::Path;
use thiserror::Error;

use crate::features::gauntlet::models::{LayerDefinition, LayerRequirement};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error while reading configuration: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON syntax error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("TOML syntax error: {0}")]
    TomlError(String),

    #[error("Configuration validation failed with {0} issue(s)")]
    ValidationError(usize),

    #[error("Unknown stack profile: '{0}'")]
    UnknownStack(String),
}

/// Severity classification for configuration validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// An individual validation finding.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigValidationIssue {
    pub severity: ValidationSeverity,
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub remediation: Option<String>,
}

impl ConfigValidationIssue {
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        field: Option<String>,
        remediation: Option<String>,
    ) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            code: code.into(),
            message: message.into(),
            field,
            remediation,
        }
    }

    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        field: Option<String>,
        remediation: Option<String>,
    ) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            code: code.into(),
            message: message.into(),
            field,
            remediation,
        }
    }
}

/// Summary report of configuration schema and path validation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigValidationReport {
    pub is_valid: bool,
    pub issues: Vec<ConfigValidationIssue>,
}

impl ConfigValidationReport {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            issues: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: ConfigValidationIssue) {
        if issue.severity == ValidationSeverity::Error {
            self.is_valid = false;
        }
        self.issues.push(issue);
    }
}

impl Default for ConfigValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for workspace paths and document bindings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PathsConfig {
    #[serde(default = "default_tasks_dir")]
    pub tasks_dir: String,
    #[serde(default = "default_spec_file")]
    pub spec_file: String,
    #[serde(default = "default_context_file")]
    pub context_file: String,
    #[serde(default = "default_coding_standards")]
    pub coding_standards_file: String,
}

fn default_tasks_dir() -> String {
    "tasks".to_string()
}
fn default_spec_file() -> String {
    "spec.md".to_string()
}
fn default_context_file() -> String {
    "CONTEXT.md".to_string()
}
fn default_coding_standards() -> String {
    "CODING_STANDARDS.md".to_string()
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            tasks_dir: default_tasks_dir(),
            spec_file: default_spec_file(),
            context_file: default_context_file(),
            coding_standards_file: default_coding_standards(),
        }
    }
}

/// Configuration for a single verification layer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayerConfig {
    pub name: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: f64,
}

fn default_timeout() -> f64 {
    60.0
}

impl LayerConfig {
    pub fn to_layer_definition(&self) -> LayerDefinition {
        let req = if self.optional {
            LayerRequirement::Optional
        } else {
            LayerRequirement::Required
        };
        LayerDefinition {
            name: self.name.clone(),
            command: self.command.clone(),
            optional: self.optional,
            timeout_seconds: self.timeout_seconds,
            requirement: req,
        }
    }
}

/// Metadata and layer templates for a recognized programming stack profile.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StackProfile {
    pub name: String,
    pub description: String,
    pub detection_files: Vec<String>,
    pub default_layers: Vec<LayerConfig>,
}

/// Root gauntlet configuration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GauntletConfig {
    #[serde(default = "default_stack")]
    pub stack: String,
    #[serde(default = "default_true")]
    pub save_evidence: bool,
    #[serde(default = "default_evidence_file")]
    pub evidence_file: String,
    #[serde(default = "default_evidence_markdown")]
    pub evidence_markdown_file: String,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub layers: Vec<LayerConfig>,
}

fn default_stack() -> String {
    "rust".to_string()
}
fn default_true() -> bool {
    true
}
fn default_evidence_file() -> String {
    "evidence.json".to_string()
}
fn default_evidence_markdown() -> String {
    "evidence.md".to_string()
}

impl Default for GauntletConfig {
    fn default() -> Self {
        Self::default_for_stack("rust")
    }
}

impl GauntletConfig {
    /// Loads configuration looking for gauntlet.toml, gauntlet.json, or fallback to stack detection.
    pub fn load(workspace: &Path) -> Result<Self, ConfigError> {
        crate::features::config::loader::load_config(workspace)
    }

    /// Parses gauntlet.json into GauntletConfig.
    pub fn parse_json(content: &str) -> Result<Self, ConfigError> {
        crate::features::config::loader::parse_json(content)
    }

    /// Parses gauntlet.toml into GauntletConfig using the streaming zero-dependency parser.
    pub fn parse_toml(content: &str) -> Result<Self, ConfigError> {
        crate::features::config::loader::parse_toml(content)
    }

    /// Renders this configuration to canonical gauntlet.toml formatted string.
    pub fn render_toml(&self) -> String {
        crate::features::config::loader::render_toml(self)
    }

    /// Produces a default configuration for a given programming stack profile.
    pub fn default_for_stack(stack: &str) -> Self {
        crate::features::config::profiles::default_config_for_stack(stack)
    }

    /// Detects default configuration based on workspace files.
    pub fn detect_default(workspace: &Path) -> Self {
        let stack = crate::features::config::profiles::detect_stack(workspace);
        Self::default_for_stack(stack)
    }

    /// Converts configured layers to executable LayerDefinitions.
    pub fn to_layer_definitions(&self) -> Vec<LayerDefinition> {
        self.layers
            .iter()
            .map(|l| l.to_layer_definition())
            .collect()
    }
}
