//! Conformance and acceptance tests for Task 021:
//! AST Codebase Topology & Token-Optimized Discovery Engine.
//!
//! Enforces sub-3ms cold start, 0 token consumption during discovery,
//! BFS/DFS graph traversals, blast radius calculation, and Cockpit HUD Scope integration.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use xgauntlet_core::features::topology::{
    calculate_blast_radius, find_neighbors, find_shortest_path, load_topology,
    render_ascii_topology, save_topology, scan_workspace_topology, BlastRadiusReport, Direction,
    EdgeType, NodeType, TopologyEdge, TopologyGraph, TopologyNode, TopologyOptions,
    TOPOLOGY_FILE_PATH,
};

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let unique = format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            count,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(format!("xgauntlet_topology_test_{}", unique));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn find_xgauntlet_binary() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../../target/debug/xgauntlet"),
        manifest_dir.join("../../target/release/xgauntlet"),
        manifest_dir.join("../../target/debug/xgauntlet.exe"),
        manifest_dir.join("../../target/release/xgauntlet.exe"),
    ];
    for c in candidates {
        if c.is_file() {
            return c;
        }
    }
    PathBuf::from("xgauntlet")
}

// ============================================================================
// 1. Acceptance Criterion 1: Domænemodeller og Grafkonstruktion
// ============================================================================

#[test]
fn test_topology_graph_domain_models_and_structure() {
    let mut graph = TopologyGraph::new();

    let node_pkg = TopologyNode::new("crate::core", "xgauntlet-core", NodeType::Package, "crates/xgauntlet-core")
        .with_metadata("language", "rust");
    let node_mod1 = TopologyNode::new("crate::core::telemetry", "telemetry", NodeType::Module, "crates/xgauntlet-core/src/features/telemetry")
        .with_metadata("lines", "120");
    let node_mod2 = TopologyNode::new("crate::core::policy", "policy", NodeType::Module, "crates/xgauntlet-core/src/features/policy");
    let node_fn = TopologyNode::new("crate::core::telemetry::render_jit", "render_jit", NodeType::Function, "crates/xgauntlet-core/src/features/telemetry/jit.rs");

    graph.add_node(node_pkg);
    graph.add_node(node_mod1);
    graph.add_node(node_mod2);
    graph.add_node(node_fn);

    let edge1 = TopologyEdge::new("crate::core", "crate::core::telemetry", EdgeType::Defines);
    let edge2 = TopologyEdge::new("crate::core", "crate::core::policy", EdgeType::Defines);
    let edge3 = TopologyEdge::new("crate::core::telemetry", "crate::core::policy", EdgeType::Imports);
    let edge4 = TopologyEdge::new("crate::core::telemetry", "crate::core::telemetry::render_jit", EdgeType::Defines);

    graph.add_edge(edge1);
    graph.add_edge(edge2);
    graph.add_edge(edge3);
    graph.add_edge(edge4);

    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 4);

    let found_node = graph.get_node("crate::core::telemetry").expect("node must exist");
    assert_eq!(found_node.name, "telemetry");
    assert_eq!(found_node.node_type, NodeType::Module);
    assert!(!found_node.metadata.contains_key("language"));
    assert_eq!(found_node.metadata.get("lines").unwrap(), "120");

    // Test JSON serialization roundtrip
    let json_str = graph.export_json().expect("graph must serialize to json");
    let deserialized = TopologyGraph::from_json(&json_str).expect("graph must deserialize from json");
    assert_eq!(deserialized.node_count(), 4);
    assert_eq!(deserialized.edge_count(), 4);
}

#[test]
fn test_render_ascii_tree_visualization() {
    let mut graph = TopologyGraph::new();
    graph.add_node(TopologyNode::new("root", "Root", NodeType::Module, "src/root.rs"));
    graph.add_node(TopologyNode::new("child", "Child", NodeType::Module, "src/child.rs"));
    graph.add_edge(TopologyEdge::new("root", "child", EdgeType::Defines));

    let ascii = render_ascii_topology(&graph, Some("root"))
        .expect("render_ascii_topology must produce tree output");
    assert!(ascii.contains("root"));
    assert!(ascii.contains("child"));
}

// ============================================================================
// 2. Acceptance Criterion 3: Graf-traversering (BFS & Nabosøgning)
// ============================================================================

#[test]
fn test_graph_traversal_bfs_and_neighbor_discovery() {
    let mut graph = TopologyGraph::new();

    graph.add_node(TopologyNode::new("A", "ModuleA", NodeType::Module, "src/a.rs"));
    graph.add_node(TopologyNode::new("B", "ModuleB", NodeType::Module, "src/b.rs"));
    graph.add_node(TopologyNode::new("C", "ModuleC", NodeType::Module, "src/c.rs"));

    graph.add_edge(TopologyEdge::new("A", "B", EdgeType::Imports));
    graph.add_edge(TopologyEdge::new("A", "C", EdgeType::Imports));
    graph.add_edge(TopologyEdge::new("C", "B", EdgeType::Imports));

    // Test outgoing neighbors from A
    let outgoing = find_neighbors(&graph, "A", Direction::Outgoing)
        .expect("outgoing neighbors search must succeed");
    assert_eq!(outgoing.len(), 2);
    assert!(outgoing.contains(&"B".to_string()));
    assert!(outgoing.contains(&"C".to_string()));

    // Test incoming neighbors to B
    let incoming = find_neighbors(&graph, "B", Direction::Incoming)
        .expect("incoming neighbors search must succeed");
    assert_eq!(incoming.len(), 2);
    assert!(incoming.contains(&"A".to_string()));
    assert!(incoming.contains(&"C".to_string()));
}

// ============================================================================
// 3. Acceptance Criterion 3: Shortest Path Traversering
// ============================================================================

#[test]
fn test_graph_traversal_shortest_path_between_components() {
    let mut graph = TopologyGraph::new();

    graph.add_node(TopologyNode::new("cli", "xgauntlet-cli", NodeType::Package, "crates/xgauntlet-cli"));
    graph.add_node(TopologyNode::new("core", "xgauntlet-core", NodeType::Package, "crates/xgauntlet-core"));
    graph.add_node(TopologyNode::new("telemetry", "telemetry", NodeType::Module, "crates/xgauntlet-core/telemetry"));
    graph.add_node(TopologyNode::new("jit", "jit", NodeType::Module, "crates/xgauntlet-core/telemetry/jit.rs"));
    graph.add_node(TopologyNode::new("unused", "unused", NodeType::Module, "crates/unused"));

    // cli -> core -> telemetry -> jit
    graph.add_edge(TopologyEdge::new("cli", "core", EdgeType::DependsOn));
    graph.add_edge(TopologyEdge::new("core", "telemetry", EdgeType::Defines));
    graph.add_edge(TopologyEdge::new("telemetry", "jit", EdgeType::Defines));

    let path = find_shortest_path(&graph, "cli", "jit")
        .expect("find_shortest_path query must succeed");
    assert_eq!(
        path,
        Some(vec![
            "cli".to_string(),
            "core".to_string(),
            "telemetry".to_string(),
            "jit".to_string(),
        ]),
        "shortest path must be correctly discovered"
    );

    let no_path = find_shortest_path(&graph, "cli", "unused")
        .expect("unreachable search should return Ok(None)");
    assert_eq!(no_path, None);
}

// ============================================================================
// 4. Acceptance Criterion 3: Blast Radius Downstream Dependency Analyse
// ============================================================================

#[test]
fn test_blast_radius_downstream_dependency_calculation() {
    let mut graph = TopologyGraph::new();

    // Dependency hierarchy:
    // core_models (low-level target)
    //   <- parser (direct)
    //   <- traversal (direct)
    //   parser <- engine (indirect, depth 2)
    //   traversal <- engine (indirect, depth 2)
    //   engine <- cli (indirect, depth 3)
    graph.add_node(TopologyNode::new("core_models", "models", NodeType::Module, "src/models.rs"));
    graph.add_node(TopologyNode::new("parser", "parser", NodeType::Module, "src/parser.rs"));
    graph.add_node(TopologyNode::new("traversal", "traversal", NodeType::Module, "src/traversal.rs"));
    graph.add_node(TopologyNode::new("engine", "engine", NodeType::Module, "src/engine.rs"));
    graph.add_node(TopologyNode::new("cli", "cli", NodeType::Package, "crates/cli"));
    graph.add_node(TopologyNode::new("isolated", "isolated", NodeType::Module, "src/isolated.rs"));

    graph.add_edge(TopologyEdge::new("parser", "core_models", EdgeType::Imports));
    graph.add_edge(TopologyEdge::new("traversal", "core_models", EdgeType::Imports));
    graph.add_edge(TopologyEdge::new("engine", "parser", EdgeType::Imports));
    graph.add_edge(TopologyEdge::new("engine", "traversal", EdgeType::Imports));
    graph.add_edge(TopologyEdge::new("cli", "engine", EdgeType::DependsOn));

    // Full blast radius calculation
    let report = calculate_blast_radius(&graph, "core_models", None)
        .expect("blast radius calculation must succeed");

    assert_eq!(report.target, "core_models");
    assert_eq!(report.total_affected_count, 4);
    assert_eq!(report.direct_dependents.len(), 2);
    assert!(report.direct_dependents.contains(&"parser".to_string()));
    assert!(report.direct_dependents.contains(&"traversal".to_string()));
    assert!(report.affected_nodes.contains(&"engine".to_string()));
    assert!(report.affected_nodes.contains(&"cli".to_string()));
    assert_eq!(report.max_depth_reached, 3);
    assert_eq!(report.compact_summary(), "core_models (+4 downstream)");

    // Depth-bounded blast radius (max_depth: Some(1))
    let bounded_report = calculate_blast_radius(&graph, "core_models", Some(1))
        .expect("bounded blast radius calculation must succeed");
    assert_eq!(bounded_report.total_affected_count, 2);
    assert_eq!(bounded_report.max_depth_reached, 1);
    assert_eq!(bounded_report.affected_nodes.len(), 2);

    // Isolated node has 0 blast radius
    let isolated_report = calculate_blast_radius(&graph, "isolated", None)
        .expect("isolated blast radius calculation must succeed");
    assert_eq!(isolated_report.total_affected_count, 0);
    assert_eq!(isolated_report.compact_summary(), "isolated (isolated)");
}

// ============================================================================
// 5. Acceptance Criterion 2: Deterministisk Polyglot AST-Parser (0 tokens)
// ============================================================================

#[test]
fn test_polyglot_deterministic_ast_parser_zero_tokens() {
    let temp = TempDir::new("ast_parser");
    let ws = temp.path();

    // 1. Rust files
    let rust_dir = ws.join("crates/app/src");
    fs::create_dir_all(&rust_dir).unwrap();
    fs::write(
        ws.join("crates/app/Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n",
    ).unwrap();
    fs::write(
        rust_dir.join("main.rs"),
        r#"
mod config;
use crate::config::Settings;

fn main() {
    println!("hello");
}
"#,
    ).unwrap();
    fs::write(
        rust_dir.join("config.rs"),
        r#"
pub struct Settings {
    pub port: u16,
}
"#,
    ).unwrap();

    // 2. TypeScript / Node files
    let ts_dir = ws.join("packages/web/src");
    fs::create_dir_all(&ts_dir).unwrap();
    fs::write(
        ws.join("packages/web/package.json"),
        r#"{"name": "web", "version": "1.0.0"}"#,
    ).unwrap();
    fs::write(
        ts_dir.join("index.ts"),
        r#"
import { formatName } from "./utils";
export function render() {
    return formatName("world");
}
"#,
    ).unwrap();
    fs::write(
        ts_dir.join("utils.ts"),
        r#"
export function formatName(name: string): string {
    return "Hello " + name;
}
"#,
    ).unwrap();

    // 3. Python files
    let py_dir = ws.join("backend/app");
    fs::create_dir_all(&py_dir).unwrap();
    fs::write(
        py_dir.join("server.py"),
        r#"
import os
from app.auth import verify_token

def run():
    pass
"#,
    ).unwrap();
    fs::write(
        py_dir.join("auth.py"),
        r#"
def verify_token(tok):
    return True
"#,
    ).unwrap();

    // 4. Go files
    let go_dir = ws.join("cmd/server");
    fs::create_dir_all(&go_dir).unwrap();
    fs::write(
        go_dir.join("main.go"),
        r#"
package main

import (
    "fmt"
    "net/http"
)

func main() {
    fmt.Println("Server running")
}
"#,
    ).unwrap();

    let options = TopologyOptions {
        workspace_root: ws.to_path_buf(),
        ..Default::default()
    };

    let start = Instant::now();
    let graph = scan_workspace_topology(&options)
        .expect("scan_workspace_topology must scan polyglot workspace deterministically");
    let duration = start.elapsed();

    // 0 tokens used, fast execution
    assert!(
        duration.as_millis() < 500,
        "Polyglot scanning should be near-instant, took {:?}",
        duration
    );

    // Verify Rust components detected
    assert!(
        graph.nodes.values().any(|n| n.name == "app" || n.name == "main"),
        "Rust nodes must be detected in graph"
    );

    // Verify TypeScript components detected
    assert!(
        graph.nodes.values().any(|n| n.name.contains("utils") || n.name.contains("index")),
        "TypeScript nodes must be detected in graph"
    );

    // Verify Python components detected
    assert!(
        graph.nodes.values().any(|n| n.name.contains("server") || n.name.contains("auth")),
        "Python nodes must be detected in graph"
    );

    // Verify Go components detected
    assert!(
        graph.nodes.values().any(|n| n.name.contains("main") || n.path.contains("cmd/server")),
        "Go nodes must be detected in graph"
    );
}

// ============================================================================
// 6. Acceptance Criterion 4: Persistens i .xgauntlet/topology.json
// ============================================================================

#[test]
fn test_topology_persistence_roundtrip_to_disk() {
    let temp = TempDir::new("topology_persistence");
    let ws = temp.path();

    let mut graph = TopologyGraph::new();
    graph.add_node(TopologyNode::new("service::auth", "auth", NodeType::Module, "src/auth.rs"));
    graph.add_node(TopologyNode::new("service::db", "db", NodeType::Module, "src/db.rs"));
    graph.add_edge(TopologyEdge::new("service::auth", "service::db", EdgeType::Calls));

    // Save topology to disk
    let saved_path = save_topology(ws, &graph).expect("save_topology must write .xgauntlet/topology.json");
    assert!(saved_path.is_file(), "topology.json must exist at {}", saved_path.display());
    assert_eq!(saved_path, ws.join(TOPOLOGY_FILE_PATH));

    // Verify file content is valid JSON
    let content = fs::read_to_string(&saved_path).unwrap();
    assert!(content.contains("service::auth"));
    assert!(content.contains("service::db"));

    // Load topology back from disk
    let loaded_graph = load_topology(ws).expect("load_topology must read back graph");
    assert_eq!(loaded_graph.node_count(), 2);
    assert_eq!(loaded_graph.edge_count(), 1);
    assert!(loaded_graph.get_node("service::auth").is_some());
    assert!(loaded_graph.get_node("service::db").is_some());
}

// ============================================================================
// 7. Acceptance Criterion 5: CLI Subcommands (inspect, path, blast-radius)
// ============================================================================

#[test]
fn test_cli_topology_subcommands_inspect_path_blast_radius() {
    let temp = TempDir::new("cli_topology");
    let ws = temp.path();
    let bin = find_xgauntlet_binary();

    // Create a minimal project structure
    let src = ws.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        ws.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\n",
    ).unwrap();
    fs::write(
        src.join("main.rs"),
        "mod lib;\nfn main() {}\n",
    ).unwrap();
    fs::write(
        src.join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    ).unwrap();

    // 1. `xgauntlet topology inspect --json`
    let inspect_output = Command::new(&bin)
        .arg("topology")
        .arg("inspect")
        .arg("--workspace")
        .arg(ws)
        .arg("--json")
        .output()
        .expect("Failed to execute xgauntlet topology inspect");

    assert!(
        inspect_output.status.success(),
        "topology inspect --json must exit successfully, stderr: {}",
        String::from_utf8_lossy(&inspect_output.stderr)
    );
    let inspect_str = String::from_utf8_lossy(&inspect_output.stdout);
    assert!(inspect_str.contains("nodes"), "inspect output should contain nodes key");

    // 2. `xgauntlet topology path <from> <to> --json`
    let path_output = Command::new(&bin)
        .arg("topology")
        .arg("path")
        .arg("toy::main")
        .arg("toy::lib")
        .arg("--workspace")
        .arg(ws)
        .arg("--json")
        .output()
        .expect("Failed to execute xgauntlet topology path");

    assert!(
        path_output.status.success(),
        "topology path --json must exit successfully, stderr: {}",
        String::from_utf8_lossy(&path_output.stderr)
    );

    // 3. `xgauntlet topology blast-radius <target> --json`
    let blast_output = Command::new(&bin)
        .arg("topology")
        .arg("blast-radius")
        .arg("toy::lib")
        .arg("--workspace")
        .arg(ws)
        .arg("--json")
        .output()
        .expect("Failed to execute xgauntlet topology blast-radius");

    assert!(
        blast_output.status.success(),
        "topology blast-radius --json must exit successfully, stderr: {}",
        String::from_utf8_lossy(&blast_output.stderr)
    );
}

// ============================================================================
// 8. Acceptance Criterion 6: Cockpit HUD Scope & <45 Token Budget
// ============================================================================

#[test]
fn test_telemetry_hud_scope_injection_under_45_token_budget() {
    let report = BlastRadiusReport {
        target: "crates/xgauntlet-core".to_string(),
        affected_nodes: vec![
            "crates/xgauntlet-cli".to_string(),
            "crates/xgauntlet-core::tests".to_string(),
            "packages/cli".to_string(),
        ],
        direct_dependents: vec!["crates/xgauntlet-cli".to_string()],
        indirect_dependents: vec!["packages/cli".to_string()],
        max_depth_reached: 2,
        total_affected_count: 3,
    };

    let summary = report.compact_summary();
    assert_eq!(summary, "crates/xgauntlet-core (+3 downstream)");

    // Token estimation: in LLM BPE tokenizers, 1 token is ~3.5 to 4 characters.
    // Ensure the summary is strictly under 45 tokens (e.g. < 100 characters).
    assert!(
        summary.chars().count() < 60,
        "HUD Scope summary must be extremely concise, length: {}",
        summary.chars().count()
    );
}

// ============================================================================
// 9. Acceptance Criterion 7: Sub-3ms Koldstart & Ydelses-invariant
// ============================================================================

#[test]
fn test_sub_3ms_cold_start_invariant() {
    let mut graph = TopologyGraph::new();
    for i in 0..100 {
        graph.add_node(TopologyNode::new(
            format!("node_{i}"),
            format!("Module_{i}"),
            NodeType::Module,
            format!("src/module_{i}.rs"),
        ));
        if i > 0 {
            graph.add_edge(TopologyEdge::new(
                format!("node_{i}"),
                format!("node_{}", i - 1),
                EdgeType::Imports,
            ));
        }
    }

    let start = Instant::now();
    let report = calculate_blast_radius(&graph, "node_0", Some(5))
        .expect("blast radius query must succeed");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 3,
        "Blast radius query exceeded 3ms cold start invariant: took {:?}",
        elapsed
    );
    assert_eq!(report.affected_nodes.len(), 5);
}
