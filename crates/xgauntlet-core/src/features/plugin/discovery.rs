//! Cross-platform agent harness discovery engine.

use std::path::Path;
use super::models::{DiscoveredHarness, PlatformTarget};

/// Discovers installed AI agent harnesses in a specific home directory for target platform.
pub fn discover_harnesses_in(_home_dir: &Path, _platform: PlatformTarget) -> Vec<DiscoveredHarness> {
    // RED phase: stub returns empty list
    Vec::new()
}

/// Discovers installed AI agent harnesses on current host platform.
pub fn discover_installed_harnesses() -> Vec<DiscoveredHarness> {
    // RED phase: stub returns empty list
    Vec::new()
}
