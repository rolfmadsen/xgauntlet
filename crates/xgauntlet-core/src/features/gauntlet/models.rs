//! Domain models for gauntlet execution layers, requirements, and reports.

use serde::{Deserialize, Serialize};

/// Execution status of a verification layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayerExecutionStatus {
    Passed,
    Failed,
    Skipped,
    Unavailable,
    TimedOut,
    Error,
}

impl LayerExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "PASSED",
            Self::Failed => "FAILED",
            Self::Skipped => "SKIPPED",
            Self::Unavailable => "UNAVAILABLE",
            Self::TimedOut => "TIMED_OUT",
            Self::Error => "ERROR",
        }
    }
}

/// Enforcement requirement for a verification layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayerRequirement {
    Required,
    Optional,
}

/// Definition of a single verification layer in the gauntlet pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerDefinition {
    pub name: String,
    pub command: Vec<String>,
    pub optional: bool,
    pub timeout_seconds: f64,
    pub requirement: LayerRequirement,
}

impl LayerDefinition {
    /// Creates a new required verification layer with default 60s timeout.
    pub fn new(name: impl Into<String>, command: Vec<String>) -> Self {
        Self {
            name: name.into(),
            command,
            optional: false,
            timeout_seconds: 60.0,
            requirement: LayerRequirement::Required,
        }
    }

    /// Sets the timeout in seconds for this layer.
    pub fn with_timeout(mut self, timeout_seconds: f64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Sets the requirement level for this layer.
    pub fn with_requirement(mut self, requirement: LayerRequirement) -> Self {
        self.requirement = requirement;
        self.optional = requirement == LayerRequirement::Optional;
        self
    }

    /// Marks this layer as optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self.requirement = LayerRequirement::Optional;
        self
    }
}

/// Execution result of a single verification layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerResult {
    pub name: String,
    pub exit_code: i32,
    pub passed: bool,
    pub output: String,
    pub status: LayerExecutionStatus,
    pub duration_seconds: f64,
    pub requirement: LayerRequirement,
}

/// Consolidated execution report from running verification layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GauntletReport {
    pub success: bool,
    pub layers: Vec<LayerResult>,
    pub total_duration_seconds: f64,
}

impl GauntletReport {
    pub fn new(success: bool, layers: Vec<LayerResult>, total_duration_seconds: f64) -> Self {
        Self {
            success,
            layers,
            total_duration_seconds,
        }
    }
}
