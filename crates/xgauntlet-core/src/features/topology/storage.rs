//! Deterministic topology persistence to `.xgauntlet/topology.json`.

use super::models::{TopologyError, TopologyGraph};
use std::fs;
use std::path::{Path, PathBuf};

pub const TOPOLOGY_FILE_PATH: &str = ".xgauntlet/topology.json";

/// Persists the topology graph deterministically to `.xgauntlet/topology.json`.
pub fn save_topology(
    workspace_root: &Path,
    graph: &TopologyGraph,
) -> Result<PathBuf, TopologyError> {
    let target_path = workspace_root.join(TOPOLOGY_FILE_PATH);
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = graph.export_json()?;
    fs::write(&target_path, json)?;
    Ok(target_path)
}

/// Loads the persisted topology graph from `.xgauntlet/topology.json`.
pub fn load_topology(workspace_root: &Path) -> Result<TopologyGraph, TopologyError> {
    let target_path = workspace_root.join(TOPOLOGY_FILE_PATH);
    let content = fs::read_to_string(&target_path)?;
    TopologyGraph::from_json(&content)
}
