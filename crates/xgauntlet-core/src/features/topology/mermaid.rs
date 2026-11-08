//! Deterministic AST to Mermaid class diagram generator.

use super::models::{NodeType, TopologyEdge, TopologyGraph, TopologyNode};

/// Level of detail for Mermaid class diagram generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MermaidLevel {
    /// Conceptual domain view: only core entities and business relations.
    Conceptual,
    /// Detailed view: includes all fields, methods and technical structs.
    Detailed,
}

impl Default for MermaidLevel {
    fn default() -> Self {
        Self::Conceptual
    }
}

/// Options configuring Mermaid class diagram generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MermaidExportOptions {
    pub direction: String,
    pub level: MermaidLevel,
    pub domain_only: bool,
}

impl Default for MermaidExportOptions {
    fn default() -> Self {
        Self {
            direction: "TB".to_string(),
            level: MermaidLevel::Conceptual,
            domain_only: true,
        }
    }
}

/// Exports a TopologyGraph to a deterministic Mermaid classDiagram.
pub fn export_mermaid_class_diagram(
    _graph: &TopologyGraph,
    _options: &MermaidExportOptions,
) -> String {
    // RED PHASE: Stub returns empty string to fail acceptance tests
    String::new()
}
