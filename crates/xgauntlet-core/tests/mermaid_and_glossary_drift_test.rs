//! Acceptance and invariant tests for Task 062:
//! Deterministic Mermaid Class Diagram Generation & Two-Way Glossary Drift Gate.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::tasks::{validate_glossary_drift, GlossaryDriftOptions};
use xgauntlet_core::features::topology::{
    export_mermaid_class_diagram, EdgeType, MermaidExportOptions, MermaidLevel, NodeType,
    TopologyEdge, TopologyGraph, TopologyNode,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
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
        let path = std::env::temp_dir().join(format!("xgauntlet_drift_test_{}", unique));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// ============================================================================
// 1. Mermaid Class Diagram Generation Tests
// ============================================================================

#[test]
fn test_mermaid_class_diagram_generation_conceptual() {
    let mut graph = TopologyGraph::new();

    let mut node_task = TopologyNode::new(
        "crate::core::tasks::TaskPackageInfo",
        "TaskPackageInfo",
        NodeType::Struct,
        "crates/xgauntlet-core/src/features/tasks/models.rs",
    );
    node_task
        .metadata
        .insert("is_domain_model".to_string(), "true".to_string());
    node_task.metadata.insert(
        "fields".to_string(),
        "task_id:String,title:String,status:TaskStatus,acceptance_criteria:Vec<String>".to_string(),
    );

    let mut node_status = TopologyNode::new(
        "crate::core::tasks::TaskStatus",
        "TaskStatus",
        NodeType::Struct,
        "crates/xgauntlet-core/src/features/tasks/models.rs",
    );
    node_status
        .metadata
        .insert("is_domain_model".to_string(), "true".to_string());
    node_status
        .metadata
        .insert("is_enum".to_string(), "true".to_string());
    node_status
        .metadata
        .insert("variants".to_string(), "Draft,Active,Done".to_string());

    let mut node_adapter = TopologyNode::new(
        "crate::core::adapters::HarnessAdapter",
        "HarnessAdapter",
        NodeType::Interface,
        "crates/xgauntlet-core/src/features/adapters/models.rs",
    );
    node_adapter
        .metadata
        .insert("is_domain_model".to_string(), "true".to_string());

    let mut node_tech = TopologyNode::new(
        "crate::core::checkpoint::GitRunner",
        "GitRunner",
        NodeType::Struct,
        "crates/xgauntlet-core/src/features/checkpoint/git.rs",
    );
    node_tech
        .metadata
        .insert("is_domain_model".to_string(), "false".to_string());

    graph.add_node(node_task);
    graph.add_node(node_status);
    graph.add_node(node_adapter);
    graph.add_node(node_tech);

    graph.add_edge(TopologyEdge::new(
        "crate::core::tasks::TaskPackageInfo",
        "crate::core::tasks::TaskStatus",
        EdgeType::DependsOn,
    ));

    let options = MermaidExportOptions {
        direction: "TB".to_string(),
        level: MermaidLevel::Conceptual,
        domain_only: true,
    };

    let mermaid = export_mermaid_class_diagram(&graph, &options);

    assert!(
        mermaid.starts_with("classDiagram\n    direction TB"),
        "Mermaid output must start with classDiagram and direction TB, got:\n{}",
        mermaid
    );
    assert!(
        mermaid.contains("class TaskPackageInfo {"),
        "Mermaid output must declare TaskPackageInfo"
    );
    assert!(
        mermaid.contains("<<enumeration>>") && mermaid.contains("class TaskStatus {"),
        "TaskStatus must be rendered as an enumeration"
    );
    assert!(
        mermaid.contains("<<interface>>") && mermaid.contains("class HarnessAdapter {"),
        "HarnessAdapter must be rendered as an interface"
    );
    assert!(
        mermaid.contains("TaskPackageInfo *-- TaskStatus : status")
            || mermaid.contains("TaskPackageInfo *-- TaskStatus")
            || mermaid.contains("TaskPackageInfo --> TaskStatus"),
        "Must contain dependency/composition relationship between TaskPackageInfo and TaskStatus"
    );
    assert!(
        !mermaid.contains("GitRunner"),
        "Conceptual view with domain_only=true must not contain technical GitRunner"
    );
}

#[test]
fn test_mermaid_class_diagram_detailed_includes_technical_structs() {
    let mut graph = TopologyGraph::new();

    let node_tech = TopologyNode::new(
        "crate::core::checkpoint::GitRunner",
        "GitRunner",
        NodeType::Struct,
        "crates/xgauntlet-core/src/features/checkpoint/git.rs",
    );
    graph.add_node(node_tech);

    let options = MermaidExportOptions {
        direction: "TB".to_string(),
        level: MermaidLevel::Detailed,
        domain_only: false,
    };

    let mermaid = export_mermaid_class_diagram(&graph, &options);

    assert!(
        mermaid.contains("class GitRunner {"),
        "Detailed view must include technical GitRunner struct"
    );
}

// ============================================================================
// 2. Glossary Drift Gate Tests
// ============================================================================

#[test]
fn test_glossary_drift_detects_orphan_models_missing_in_context() {
    let ws = TempWorkspace::new("orphan_models");
    let ws_path = ws.path();

    // Create minimal valid CONTEXT.md
    let context_content = r#"---
type: Knowledge Bundle Index
title: "Test Context"
status: stable
---

# Test Context

**Task**:
An executable unit of engineering work.
_Avoid_: ticket, issue.

**Layer**:
A verification step with timeout.
_Avoid_: phase, step.
"#;
    fs::write(ws_path.join("CONTEXT.md"), context_content).unwrap();

    // Create feature model with an orphan struct not defined in CONTEXT.md
    let models_dir = ws_path.join("crates/xgauntlet-core/src/features/evidence");
    fs::create_dir_all(&models_dir).unwrap();
    let models_content = r#"
pub struct TaskContract {
    pub task_id: String,
}

pub struct UnregisteredBountyEvidence {
    pub payout_id: String,
}
"#;
    fs::write(models_dir.join("models.rs"), models_content).unwrap();

    let options = GlossaryDriftOptions::default();
    let report = validate_glossary_drift(ws_path, &options);

    assert!(
        !report.is_valid,
        "Report must be invalid when orphan model exists"
    );
    assert!(
        report
            .orphan_models
            .iter()
            .any(|m| m.contains("UnregisteredBountyEvidence")),
        "Expected UnregisteredBountyEvidence in orphan_models, got: {:?}",
        report.orphan_models
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("UnregisteredBountyEvidence")),
        "Expected diagnostic finding mentioning UnregisteredBountyEvidence"
    );
}

#[test]
fn test_glossary_drift_passes_when_all_models_defined_in_context() {
    let ws = TempWorkspace::new("matching_models");
    let ws_path = ws.path();

    let context_content = r#"---
type: Knowledge Bundle Index
title: "Test Context"
status: stable
---

# Test Context

**Task**:
An executable unit of work.
_Avoid_: ticket.

**Verification Report**:
A sealed verification document.
_Avoid_: proof report.
"#;
    fs::write(ws_path.join("CONTEXT.md"), context_content).unwrap();

    let models_dir = ws_path.join("crates/xgauntlet-core/src/features/evidence");
    fs::create_dir_all(&models_dir).unwrap();
    let models_content = r#"
pub struct VerificationReport {
    pub schema: String,
}
"#;
    fs::write(models_dir.join("models.rs"), models_content).unwrap();

    let options = GlossaryDriftOptions::default();
    let report = validate_glossary_drift(ws_path, &options);

    assert!(
        report.is_valid,
        "Report should be valid when all models are defined in CONTEXT.md, errors: {:?}",
        report.diagnostics
    );
    assert!(report.orphan_models.is_empty());
}

#[test]
fn test_glossary_drift_detects_prohibited_avoid_synonyms_in_fields() {
    let ws = TempWorkspace::new("avoid_synonyms");
    let ws_path = ws.path();

    let context_content = r#"---
type: Knowledge Bundle Index
title: "Test Context"
status: stable
---

# Test Context

**Task**:
An executable unit of work.
_Avoid_: ticket, issue, story.
"#;
    fs::write(ws_path.join("CONTEXT.md"), context_content).unwrap();

    let models_dir = ws_path.join("crates/xgauntlet-core/src/features/tasks");
    fs::create_dir_all(&models_dir).unwrap();
    let models_content = r#"
pub struct TaskInfo {
    pub ticket_id: String,
}
"#;
    fs::write(models_dir.join("models.rs"), models_content).unwrap();

    let options = GlossaryDriftOptions {
        strict: true,
        check_avoid: true,
    };
    let report = validate_glossary_drift(ws_path, &options);

    assert!(
        !report.is_valid,
        "Report must fail when prohibited synonym 'ticket' is used as a field"
    );
    assert!(
        report
            .avoid_violations
            .iter()
            .any(|v| v.message.contains("ticket")),
        "Expected violation mentioning prohibited synonym 'ticket', got: {:?}",
        report.avoid_violations
    );
}

#[test]
fn test_check_spec_supports_1_level_slice_without_subdomain() {
    use xgauntlet_core::features::tasks::{
        check_task_specification, ScaffoldTaskOptions, TaskScaffolder,
    };

    let ws = TempWorkspace::new("slice_1_level_spec");
    let ws_path = ws.path();
    fs::create_dir_all(ws_path.join("specs")).unwrap();
    fs::write(ws_path.join("specs/architecture.md"), "# Architecture\n").unwrap();
    fs::write(
        ws_path.join("CONTEXT.md"),
        "---\ntype: Knowledge Bundle Index\nstatus: stable\n---\n# Context\n**Architecture**:\nBoundary gate.\n_Avoid_: monolith.\n",
    )
    .unwrap();

    let opts = ScaffoldTaskOptions {
        name: "1-level-slice-task".into(),
        title: Some("1-Level Slice Task".into()),
        intent: Some("enhancement".into()),
        purpose: None,
        workspace: ws_path.to_path_buf(),
        force: true,
    };
    let scaffolded = TaskScaffolder::scaffold_slice(&opts, None, Some("architecture"))
        .expect("create 1-level slice task");
    let task_path = scaffolded.path;

    let content = fs::read_to_string(&task_path).unwrap();
    assert!(
        content.contains("slice: architecture")
            && content.contains("spec_ref: \"specs/architecture.md\"")
            && !content.contains("domain:"),
        "Expected 1-level slice frontmatter without domain, got:\n{}",
        content
    );

    let report = check_task_specification(&task_path, ws_path);
    assert!(
        report.is_valid,
        "1-level slice task with specs/<slice>.md must pass check-spec, diagnostics: {:?}",
        report.diagnostics
    );
}

#[test]
fn test_glossary_drift_supports_parenthetical_aliases_and_dogfoods_workspace() {
    let ws = TempWorkspace::new("parenthetical_aliases");
    let ws_path = ws.path();

    let context_content = r#"---
type: Knowledge Bundle Index
title: "Test Context"
status: stable
---

# Test Context

**Architecture Gate (ArchitectureConfig / ArchitectureReport)**:
Deterministic structural verifier.
_Avoid_: ad-hoc lint.
"#;
    fs::write(ws_path.join("CONTEXT.md"), context_content).unwrap();

    let models_dir = ws_path.join("crates/xgauntlet-core/src/features/architecture");
    fs::create_dir_all(&models_dir).unwrap();
    fs::write(
        models_dir.join("models.rs"),
        "pub struct ArchitectureConfig {\n    pub isolate_slices: bool,\n}\npub struct ArchitectureReport {\n    pub is_valid: bool,\n}\n",
    )
    .unwrap();

    let report = validate_glossary_drift(ws_path, &GlossaryDriftOptions::default());
    assert!(
        report.is_valid,
        "Parenthetical slash-separated model aliases must be recognized, orphans: {:?}",
        report.orphan_models
    );

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let repo_report = validate_glossary_drift(repo_root, &GlossaryDriftOptions::default());
    assert!(
        repo_report.is_valid,
        "xgauntlet's own workspace must pass glossary drift check, orphans: {:?}, avoid: {:?}",
        repo_report.orphan_models, repo_report.avoid_violations
    );
}
