//! Domain models and types for AST Codebase Topology & Discovery Engine.
//!
//! Enforces ADR 0001 (Package-by-Feature) and ADR 0004 (Harness Adapter Slices).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Supported node classification in code topology graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Package,
    Module,
    File,
    Function,
    Struct,
    Interface,
    Class,
    Unknown,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Package => "package",
            Self::Module => "module",
            Self::File => "file",
            Self::Function => "function",
            Self::Struct => "struct",
            Self::Interface => "interface",
            Self::Class => "class",
            Self::Unknown => "unknown",
        }
    }
}

/// Relationship edge type between topology nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Imports,
    Calls,
    DependsOn,
    Defines,
    Exports,
    Implements,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Imports => "imports",
            Self::Calls => "calls",
            Self::DependsOn => "depends_on",
            Self::Defines => "defines",
            Self::Exports => "exports",
            Self::Implements => "implements",
        }
    }
}

/// Traversal direction for neighbor searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Incoming,
    Outgoing,
    Both,
}

/// A node in the codebase topology graph representing a package, module, file, or symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub path: String,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl TopologyNode {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        node_type: NodeType,
        path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type,
            path: path.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// A directed edge connecting two topology nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,
}

impl TopologyEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>, edge_type: EdgeType) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type,
            weight: None,
        }
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = Some(weight);
        self
    }
}

/// Detailed blast radius report for a targeted symbol, module, or package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BlastRadiusReport {
    pub target: String,
    pub affected_nodes: Vec<String>,
    pub direct_dependents: Vec<String>,
    pub indirect_dependents: Vec<String>,
    pub max_depth_reached: usize,
    pub total_affected_count: usize,
}

impl BlastRadiusReport {
    /// Formats a concise token-optimized summary suitable for Cockpit HUD `Scope: ...`.
    pub fn compact_summary(&self) -> String {
        if self.affected_nodes.is_empty() {
            format!("{} (isolated)", self.target)
        } else {
            format!("{} (+{} downstream)", self.target, self.affected_nodes.len())
        }
    }
}

/// In-memory graph structure representing the codebase topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TopologyGraph {
    pub nodes: HashMap<String, TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

impl TopologyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: TopologyNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: TopologyEdge) {
        self.edges.push(edge);
    }

    pub fn get_node(&self, id: &str) -> Option<&TopologyNode> {
        self.nodes.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn export_json(&self) -> Result<String, TopologyError> {
        serde_json::to_string_pretty(self).map_err(TopologyError::from)
    }

    pub fn from_json(json_str: &str) -> Result<Self, TopologyError> {
        serde_json::from_str(json_str).map_err(TopologyError::from)
    }
}

/// Options controlling workspace scanning and AST parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyOptions {
    pub workspace_root: PathBuf,
    pub max_depth: Option<usize>,
    pub exclude_dirs: Vec<String>,
    pub detect_symbols: bool,
}

impl Default for TopologyOptions {
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::from("."),
            max_depth: None,
            exclude_dirs: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                ".venv".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
            detect_symbols: true,
        }
    }
}

/// Domain errors for topology extraction, traversal, and persistence.
#[derive(Debug, Error)]
pub enum TopologyError {
    #[error("TDD RED phase unmet: {details}")]
    RedPhaseUnmet { details: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Node not found: '{0}'")]
    NodeNotFound(String),

    #[error("Parse error in '{path}': {message}")]
    ParseError { path: String, message: String },

    #[error("Export error: {details}")]
    ExportError { details: String },
}
