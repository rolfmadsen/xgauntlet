//! AST Codebase Topology & Token-Optimized Discovery Engine.
//!
//! Provides deterministic sub-millisecond module, symbol, and dependency graph
//! extraction, graph traversal (BFS/DFS/shortest path), blast radius calculation,
//! and Cockpit HUD Scope telemetry injection.

pub mod markdown;
pub mod mermaid;
pub mod models;
pub mod parser;
pub mod rust_ast;
pub mod storage;
pub mod traversal;
pub mod viewer;

pub use markdown::{render_architecture_markdown, render_feature_architecture_markdown};
pub use mermaid::{
    discover_topology_features, export_feature_mermaid_class_diagram, export_mermaid_class_diagram,
    MermaidExportOptions, MermaidLevel,
};
pub use models::{
    BlastRadiusReport, Direction, EdgeType, NodeType, TopologyEdge, TopologyError, TopologyGraph,
    TopologyNode, TopologyOptions,
};
pub use parser::{scan_workspace, scan_workspace_topology};
pub use storage::{load_topology, save_topology, TOPOLOGY_FILE_PATH};
pub use traversal::{
    calculate_blast_radius, find_neighbors, find_shortest_path, render_ascii_topology,
};
pub use viewer::{open_in_browser, render_pele_viewer_html, write_pele_viewer_html};
