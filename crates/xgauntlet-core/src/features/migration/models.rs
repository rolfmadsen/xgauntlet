//! Domain models and error types for the in-repo migration and auto-remediation engine.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Options configuring migration execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationOptions {
    /// Workspace root directory.
    pub workspace: PathBuf,
    /// When true, report planned actions without writing changes to disk.
    pub dry_run: bool,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            workspace: PathBuf::from("."),
            dry_run: false,
        }
    }
}

/// A planned or executed migration action on a specific file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationItem {
    /// Relative or absolute path to the target file.
    pub file: PathBuf,
    /// High-level description of the migration action.
    pub description: String,
    /// Detailed list of changes applied or proposed.
    pub changes: Vec<String>,
}

/// Comprehensive outcome report for a migration execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Overall success indicator.
    pub is_success: bool,
    /// List of migration items evaluated.
    pub items: Vec<MigrationItem>,
    /// List of files modified on disk (empty during dry run).
    pub files_modified: Vec<PathBuf>,
}

/// Errors occurring during migration operations.
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("IO error during migration: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse JSON file {path}: {source}")]
    JsonParse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("Migration failed: {0}")]
    Other(String),
}
