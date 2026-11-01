//! Global plugin distribution, harness discovery, and skills installation engine.

pub mod bundle;
pub mod discovery;
pub mod installer;
pub mod models;

pub use bundle::{
    get_embedded_hooks_manifest, get_embedded_plugin_manifest, get_embedded_skill,
    list_embedded_skills, ALL_EMBEDDED_SKILLS,
};
pub use discovery::{discover_harnesses_in, discover_installed_harnesses};
pub use installer::{clean_duplicate_settings_hooks, run_plugin_install};
pub use models::{
    DiscoveredHarness, PlatformTarget, PluginError, PluginInstallOptions, PluginInstallReport,
    PluginInstallStatus, PluginTargetInstallReport,
};
