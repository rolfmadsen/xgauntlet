//! Deterministic AST to Mermaid class diagram generator.

use super::models::{NodeType, TopologyGraph, TopologyNode};
use std::collections::{BTreeMap, BTreeSet};

/// Level of detail for Mermaid class diagram generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MermaidLevel {
    /// Conceptual domain view: only core entities and business relations.
    #[default]
    Conceptual,
    /// Detailed view: includes all fields, methods and technical structs.
    Detailed,
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
    graph: &TopologyGraph,
    options: &MermaidExportOptions,
) -> String {
    let mut out = String::new();
    if options.level == MermaidLevel::Conceptual {
        out.push_str("%% ====================================================================\n");
        out.push_str("%% 🏛️ xGauntlet Core Domain Model (Conceptual Architecture - CONTEXT.md)\n");
        out.push_str(
            "%% Reflects domain entities and aggregates. Operational errors and CLI flags\n",
        );
        out.push_str("%% are excluded in conceptual view (inspect via --level detailed).\n");
        out.push_str("%% ====================================================================\n");
    }
    out.push_str("classDiagram\n");
    let dir = if options.direction.trim().is_empty() {
        "TB"
    } else {
        options.direction.trim()
    };
    out.push_str(&format!("    direction {dir}\n\n"));

    // Collect and filter nodes deterministically
    let mut visible_nodes: BTreeMap<String, &TopologyNode> = BTreeMap::new();

    for (id, node) in &graph.nodes {
        let is_domain = if let Some(val) = node.metadata.get("is_domain_model") {
            val == "true"
        } else {
            // Infer from path: models.rs or domain paths (e.g. models.rs or /domain/)
            node.path.ends_with("models.rs")
                || node.path.contains("/models/")
                || node.path.contains("\\models\\")
                || node.path.contains("/domain/")
                || node.path.contains("\\domain\\")
        };

        if options.domain_only && options.level == MermaidLevel::Conceptual && !is_domain {
            continue;
        }

        // Conceptual domain noise filtering: exclude operational errors and CLI options
        let is_technical_scaffolding = options.level == MermaidLevel::Conceptual
            && (node.name.ends_with("Error")
                || node.name.ends_with("Exception")
                || node.name.ends_with("Options")
                || node.name.ends_with("Args")
                || node.name == "Direction"
                || node.name == "SemverBump"
                || node.name == "PlatformTarget");

        if is_technical_scaffolding {
            continue;
        }

        // Only include struct, interface, class, enum
        if matches!(
            node.node_type,
            NodeType::Struct | NodeType::Interface | NodeType::Class
        ) || node.metadata.get("is_enum").map(|v| v.as_str()) == Some("true")
        {
            visible_nodes.insert(id.clone(), node);
        }
    }

    // Render classes/enums/interfaces in alphabetical order of node.name
    let mut sorted_nodes: Vec<&TopologyNode> = visible_nodes.values().copied().collect();
    sorted_nodes.sort_by(|a, b| a.name.cmp(&b.name));

    let mut rendered_names: BTreeSet<String> = BTreeSet::new();

    for node in &sorted_nodes {
        if rendered_names.contains(&node.name) {
            continue;
        }
        rendered_names.insert(node.name.clone());

        let is_enum = node.metadata.get("is_enum").map(|v| v.as_str()) == Some("true");
        let is_interface = node.node_type == NodeType::Interface;

        out.push_str(&format!("    class {} {{\n", node.name));

        if is_enum {
            out.push_str("        <<enumeration>>\n");
            if let Some(variants) = node.metadata.get("variants") {
                for var in variants.split(',') {
                    let v = var.trim();
                    if !v.is_empty() {
                        out.push_str(&format!("        {v}\n"));
                    }
                }
            }
        } else if is_interface {
            out.push_str("        <<interface>>\n");
        }

        if let Some(fields_str) = node.metadata.get("fields") {
            for field in fields_str.split(',') {
                let f = field.trim();
                if !f.is_empty() {
                    // Replace < and > with ~ for Mermaid generics
                    let sanitized = f.replace(['<', '>'], "~");
                    out.push_str(&format!("        +{sanitized}\n"));
                }
            }
        }

        out.push_str("    }\n\n");
    }

    // Render relationships
    let mut rendered_edges: BTreeSet<String> = BTreeSet::new();

    for edge in &graph.edges {
        if let (Some(from_node), Some(to_node)) =
            (visible_nodes.get(&edge.from), visible_nodes.get(&edge.to))
        {
            let label = edge
                .metadata
                .get("field_name")
                .or_else(|| edge.metadata.get("label"))
                .map(|s| s.as_str())
                .unwrap_or("");

            let rel_str = if label.is_empty() {
                format!("    {} *-- {}\n", from_node.name, to_node.name)
            } else {
                format!("    {} *-- {} : {}\n", from_node.name, to_node.name, label)
            };

            if !rendered_edges.contains(&rel_str) {
                out.push_str(&rel_str);
                rendered_edges.insert(rel_str);
            }
        }
    }

    out
}
