//! Graph traversal algorithms: BFS, DFS blast radius, and shortest path.
//!
//! Sub-3ms cold start, zero tokens, zero background daemons.

use std::collections::{HashSet, VecDeque};
use super::models::{BlastRadiusReport, Direction, TopologyError, TopologyGraph};

/// Finds all neighbors for a specified node along incoming, outgoing, or both edge directions.
pub fn find_neighbors(
    graph: &TopologyGraph,
    id: &str,
    direction: Direction,
) -> Result<Vec<String>, TopologyError> {
    if !graph.nodes.contains_key(id) {
        return Err(TopologyError::NodeNotFound(id.to_string()));
    }

    let mut neighbors = Vec::new();
    for edge in &graph.edges {
        match direction {
            Direction::Outgoing => {
                if edge.from == id && !neighbors.contains(&edge.to) {
                    neighbors.push(edge.to.clone());
                }
            }
            Direction::Incoming => {
                if edge.to == id && !neighbors.contains(&edge.from) {
                    neighbors.push(edge.from.clone());
                }
            }
            Direction::Both => {
                if edge.from == id && !neighbors.contains(&edge.to) {
                    neighbors.push(edge.to.clone());
                }
                if edge.to == id && !neighbors.contains(&edge.from) {
                    neighbors.push(edge.from.clone());
                }
            }
        }
    }

    Ok(neighbors)
}

/// Finds the shortest directed path between two nodes in the topology graph via BFS.
pub fn find_shortest_path(
    graph: &TopologyGraph,
    from: &str,
    to: &str,
) -> Result<Option<Vec<String>>, TopologyError> {
    if !graph.nodes.contains_key(from) || !graph.nodes.contains_key(to) {
        return Ok(None);
    }

    if from == to {
        return Ok(Some(vec![from.to_string()]));
    }

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back((from.to_string(), vec![from.to_string()]));
    visited.insert(from.to_string());

    while let Some((curr, path)) = queue.pop_front() {
        for edge in &graph.edges {
            if edge.from == curr {
                let next = &edge.to;
                if next == to {
                    let mut final_path = path.clone();
                    final_path.push(next.clone());
                    return Ok(Some(final_path));
                }

                if visited.insert(next.clone()) {
                    let mut next_path = path.clone();
                    next_path.push(next.clone());
                    queue.push_back((next.clone(), next_path));
                }
            }
        }
    }

    Ok(None)
}

/// Calculates the downstream blast radius and affected dependents for a target node.
pub fn calculate_blast_radius(
    graph: &TopologyGraph,
    target: &str,
    max_depth: Option<usize>,
) -> Result<BlastRadiusReport, TopologyError> {
    if !graph.nodes.contains_key(target) {
        return Err(TopologyError::NodeNotFound(target.to_string()));
    }

    let mut visited = HashSet::new();
    visited.insert(target.to_string());

    let mut current_frontier = vec![target.to_string()];
    let mut direct_dependents = Vec::new();
    let mut indirect_dependents = Vec::new();
    let mut affected_nodes = Vec::new();
    let mut depth = 0;
    let mut max_depth_reached = 0;

    while !current_frontier.is_empty() {
        if let Some(limit) = max_depth {
            if depth >= limit {
                break;
            }
        }

        depth += 1;
        let mut next_frontier = Vec::new();

        for curr in &current_frontier {
            // Find all nodes that depend on curr (i.e. edge.to == curr)
            for edge in &graph.edges {
                if edge.to == *curr {
                    let dependent = &edge.from;
                    if visited.insert(dependent.clone()) {
                        next_frontier.push(dependent.clone());
                        affected_nodes.push(dependent.clone());
                        if depth == 1 {
                            direct_dependents.push(dependent.clone());
                        } else {
                            indirect_dependents.push(dependent.clone());
                        }
                    }
                }
            }
        }

        if !next_frontier.is_empty() {
            max_depth_reached = depth;
        }
        current_frontier = next_frontier;
    }

    let total_affected_count = affected_nodes.len();

    Ok(BlastRadiusReport {
        target: target.to_string(),
        affected_nodes,
        direct_dependents,
        indirect_dependents,
        max_depth_reached,
        total_affected_count,
    })
}

/// Renders a human-friendly ASCII tree representation of the codebase topology.
pub fn render_ascii_topology(
    graph: &TopologyGraph,
    root: Option<&str>,
) -> Result<String, TopologyError> {
    let mut output = String::new();
    let roots: Vec<&str> = match root {
        Some(r) => {
            if !graph.nodes.contains_key(r) {
                return Err(TopologyError::NodeNotFound(r.to_string()));
            }
            vec![r]
        }
        None => {
            let mut candidates: Vec<&str> = graph.nodes.keys().map(|s| s.as_str()).collect();
            candidates.sort();
            candidates
        }
    };

    if roots.is_empty() {
        return Ok("Empty topology graph".to_string());
    }

    for (idx, r) in roots.iter().enumerate() {
        if let Some(node) = graph.get_node(r) {
            output.push_str(&format!("[{}] {} ({})\n", node.node_type.as_str(), node.name, node.id));
            let mut outgoing_edges: Vec<&super::models::TopologyEdge> =
                graph.edges.iter().filter(|e| e.from == *r).collect();
            outgoing_edges.sort_by(|a, b| a.to.cmp(&b.to));

            for (e_idx, edge) in outgoing_edges.iter().enumerate() {
                let is_last = e_idx == outgoing_edges.len() - 1;
                let branch = if is_last { "└──" } else { "├──" };
                output.push_str(&format!("  {} [{}] {}\n", branch, edge.edge_type.as_str(), edge.to));
            }
            if idx < roots.len() - 1 {
                output.push('\n');
            }
        }
    }

    Ok(output)
}
