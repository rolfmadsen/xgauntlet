//! AST Codebase Topology & Token-Optimized Discovery Engine.
//!
//! Provides deterministic sub-millisecond module, symbol, and dependency graph
//! extraction, graph traversal (BFS/DFS/shortest path), blast radius calculation,
//! and Cockpit HUD Scope telemetry injection.

pub mod models;
pub mod parser;
pub mod storage;
pub mod traversal;

pub use models::{
    BlastRadiusReport, Direction, EdgeType, NodeType, TopologyEdge, TopologyError, TopologyGraph,
    TopologyNode, TopologyOptions,
};
pub use parser::{scan_workspace, scan_workspace_topology};
pub use storage::{load_topology, save_topology, TOPOLOGY_FILE_PATH};
pub use traversal::{
    calculate_blast_radius, find_neighbors, find_shortest_path, render_ascii_topology,
};
