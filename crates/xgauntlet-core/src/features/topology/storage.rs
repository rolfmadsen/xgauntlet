//! Deterministic topology persistence to `.xgauntlet/topology.json`.

use std::path::{Path, PathBuf};
use super::models::{TopologyError, TopologyGraph};

pub const TOPOLOGY_FILE_PATH: &str = ".xgauntlet/topology.json";

/// Persists the topology graph deterministically to `.xgauntlet/topology.json`.
pub fn save_topology(
    _workspace_root: &Path,
    _graph: &TopologyGraph,
) -> Result<PathBuf, TopologyError> {
    Err(TopologyError::RedPhaseUnmet {
        details: "TDD RED phase: save_topology not yet implemented".to_string(),
    })
}

/// Loads the persisted topology graph from `.xgauntlet/topology.json`.
pub fn load_topology(_workspace_root: &Path) -> Result<TopologyGraph, TopologyError> {
    Err(TopologyError::RedPhaseUnmet {
        details: "TDD RED phase: load_topology not yet implemented".to_string(),
    })
}
