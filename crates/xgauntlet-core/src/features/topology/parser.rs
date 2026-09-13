//! Polyglot AST and dependency parser for Rust, TypeScript/Node, Python, and Go.
//!
//! Sub-3ms cold start, zero tokens, zero background daemons.

use std::path::Path;
use super::models::{TopologyError, TopologyGraph, TopologyOptions};

/// Scans the specified workspace root and builds a deterministic codebase topology graph.
pub fn scan_workspace_topology(_options: &TopologyOptions) -> Result<TopologyGraph, TopologyError> {
    Err(TopologyError::RedPhaseUnmet {
        details: "TDD RED phase: scan_workspace_topology not yet implemented".to_string(),
    })
}

/// Convenience helper to scan a workspace directory with default options.
pub fn scan_workspace(root: &Path) -> Result<TopologyGraph, TopologyError> {
    let options = TopologyOptions {
        workspace_root: root.to_path_buf(),
        ..Default::default()
    };
    scan_workspace_topology(&options)
}
