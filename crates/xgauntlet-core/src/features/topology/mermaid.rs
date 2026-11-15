//! Deterministic AST to Mermaid class diagram generator.

use super::models::{EdgeType, NodeType, TopologyError, TopologyGraph, TopologyNode};
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

fn is_renderable_node(node: &TopologyNode, options: &MermaidExportOptions) -> bool {
    let is_domain = if let Some(val) = node.metadata.get("is_domain_model") {
        val == "true"
    } else {
        node.path.ends_with("models.rs")
            || node.path.contains("/models/")
            || node.path.contains("\\models\\")
            || node.path.contains("/domain/")
            || node.path.contains("\\domain\\")
    };

    if options.domain_only && options.level == MermaidLevel::Conceptual && !is_domain {
        return false;
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
        return false;
    }

    matches!(
        node.node_type,
        NodeType::Struct | NodeType::Interface | NodeType::Class
    ) || node.metadata.get("is_enum").map(|v| v.as_str()) == Some("true")
}

fn collect_renderable_nodes<'a>(
    graph: &'a TopologyGraph,
    options: &MermaidExportOptions,
) -> BTreeMap<String, &'a TopologyNode> {
    let mut visible_nodes: BTreeMap<String, &'a TopologyNode> = BTreeMap::new();
    for (id, node) in &graph.nodes {
        if is_renderable_node(node, options) {
            visible_nodes.insert(id.clone(), node);
        }
    }
    visible_nodes
}

fn extract_feature_name_from_path(raw_path: &str) -> Option<String> {
    let norm = raw_path.replace('\\', "/");
    let after = if let Some(idx) = norm.find("/features/") {
        &norm[idx + "/features/".len()..]
    } else {
        norm.strip_prefix("features/")?
    };
    let feat = after.split('/').next()?.trim();
    if feat.is_empty() || feat.ends_with(".rs") || feat.ends_with(".ts") || feat.ends_with(".js") {
        None
    } else {
        Some(feat.to_string())
    }
}

/// Discovers sorted feature slice names that own at least one renderable domain model.
pub fn discover_topology_features(
    graph: &TopologyGraph,
    options: &MermaidExportOptions,
) -> Vec<String> {
    let mut features = BTreeSet::new();
    for node in collect_renderable_nodes(graph, options).into_values() {
        if let Some(feat) = extract_feature_name_from_path(&node.path) {
            features.insert(feat);
        }
    }
    features.into_iter().collect()
}

/// Exports a TopologyGraph to a deterministic Mermaid classDiagram.
pub fn export_mermaid_class_diagram(
    graph: &TopologyGraph,
    options: &MermaidExportOptions,
) -> String {
    let visible_nodes = collect_renderable_nodes(graph, options);
    render_nodes_to_mermaid(graph, options, &visible_nodes, None)
}

/// Exports a deterministic feature-scoped Mermaid classDiagram including 1-hop neighbors.
pub fn export_feature_mermaid_class_diagram(
    graph: &TopologyGraph,
    options: &MermaidExportOptions,
    feature: &str,
) -> Result<String, TopologyError> {
    let target = feature.trim();
    if target.is_empty() {
        return Err(TopologyError::ExportError {
            details: "Feature name cannot be empty.".to_string(),
        });
    }

    let all_renderable = collect_renderable_nodes(graph, options);
    let mut scoped_ids: BTreeSet<String> = BTreeSet::new();

    for (id, node) in &all_renderable {
        if extract_feature_name_from_path(&node.path).as_deref() == Some(target) {
            scoped_ids.insert(id.clone());
        }
    }

    if scoped_ids.is_empty() {
        let available = discover_topology_features(graph, options);
        let hint = if available.is_empty() {
            "none discovered".to_string()
        } else {
            available.join(", ")
        };
        return Err(TopologyError::ExportError {
            details: format!(
                "Unknown or empty feature slice '{target}'. Available features: {hint}."
            ),
        });
    }

    // Include 1-hop directly connected neighbor models to preserve complete relationships
    let seed_ids = scoped_ids.clone();
    for edge in &graph.edges {
        if seed_ids.contains(&edge.from) && all_renderable.contains_key(&edge.to) {
            scoped_ids.insert(edge.to.clone());
        } else if seed_ids.contains(&edge.to) && all_renderable.contains_key(&edge.from) {
            scoped_ids.insert(edge.from.clone());
        }
    }

    let visible_nodes: BTreeMap<String, &TopologyNode> = scoped_ids
        .into_iter()
        .filter_map(|id| all_renderable.get(&id).map(|n| (id, *n)))
        .collect();

    Ok(render_nodes_to_mermaid(
        graph,
        options,
        &visible_nodes,
        Some(target),
    ))
}

fn render_nodes_to_mermaid(
    graph: &TopologyGraph,
    options: &MermaidExportOptions,
    visible_nodes: &BTreeMap<String, &TopologyNode>,
    feature_scope: Option<&str>,
) -> String {
    let mut out = String::new();
    out.push_str("classDiagram\n");
    let dir = if options.direction.trim().is_empty() {
        "TB"
    } else {
        options.direction.trim()
    };
    out.push_str(&format!("    direction {dir}\n\n"));

    if let Some(feat) = feature_scope {
        out.push_str(
            "    %% ====================================================================\n",
        );
        out.push_str(&format!(
            "    %% 🧩 Feature Slice: {feat} (with 1-hop domain neighbors)\n"
        ));
        out.push_str(
            "    %% ====================================================================\n\n",
        );
    } else if options.level == MermaidLevel::Conceptual {
        out.push_str(
            "    %% ====================================================================\n",
        );
        out.push_str(
            "    %% 🏛️ xGauntlet Core Domain Model (Conceptual Architecture - CONTEXT.md)\n",
        );
        out.push_str(
            "    %% Reflects domain entities and aggregates. Operational errors and CLI flags\n",
        );
        out.push_str("    %% are excluded in conceptual view (inspect via --level detailed).\n");
        out.push_str(
            "    %% ====================================================================\n\n",
        );
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
            if edge.edge_type == EdgeType::Implements {
                let rel_str = format!("    {} ..|> {}\n", from_node.name, to_node.name);
                if !rendered_edges.contains(&rel_str) {
                    out.push_str(&rel_str);
                    rendered_edges.insert(rel_str);
                }
                continue;
            }

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
