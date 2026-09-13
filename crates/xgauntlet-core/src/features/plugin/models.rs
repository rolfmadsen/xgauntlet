//! Domain models and types for xGauntlet Global Plugin Distribution.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Supported target platforms for cross-platform harness discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlatformTarget {
    Linux,
    MacOS,
    Windows,
}

impl PlatformTarget {
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::MacOS
        }
        #[cfg(target_os = "windows")]
        {
            Self::Windows
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Self::Linux
        }
    }
}

/// Information about a discovered AI agent harness and its xGauntlet plugin status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredHarness {
    pub name: String,
    pub detected: bool,
    pub config_dir: PathBuf,
    pub plugin_installed: bool,
    pub plugin_dir: PathBuf,
}

/// Options controlling `xgauntlet plugin install`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInstallOptions {
    pub global: bool,
    pub harness: Option<String>,
    pub dry_run: bool,
    pub force: bool,
    pub target: Option<PathBuf>,
    pub json: bool,
}

/// Installation status of a single plugin target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginInstallStatus {
    Installed,
    UpToDate,
    SkippedExisting,
    DryRunSimulated,
}

/// Result of installing the plugin into a specific harness or directory target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginTargetInstallReport {
    pub harness: String,
    pub target_dir: PathBuf,
    pub status: PluginInstallStatus,
    pub files_written: Vec<PathBuf>,
    pub files_skipped: Vec<PathBuf>,
}

/// Overall report of plugin installation operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInstallReport {
    pub targets: Vec<PluginTargetInstallReport>,
    pub success: bool,
    pub is_dry_run: bool,
}

/// Plugin management and distribution errors.
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("TDD RED phase unmet: {details}")]
    RedPhaseUnmet { details: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid target harness '{0}'")]
    UnknownHarness(String),

    #[error("Plugin manifest error: {details}")]
    ManifestError { details: String },
}
