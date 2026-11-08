//! Integration and acceptance tests for Task 066:
//! Automatic Architecture Mermaid Diagram Synchronization on verify --save.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use xgauntlet_core::features::config::GauntletConfig;
use xgauntlet_core::features::gauntlet::{execute_gauntlet_pipeline, GauntletOptions, LayerDefinition};

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
        let path = std::env::temp_dir().join(format!("xgauntlet_mermaid_sync_test_{}", unique));
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

fn scaffold_test_workspace(ws: &Path, diagram_file: Option<&str>) {
    let tasks_dir = ws.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();

    let task_content = r#"---
type: Task Package
title: "Task 001: Bootstrap"
description: "Bootstrap task"
status: active
generated: { by: test, at: "2026-11-08T12:00:00Z" }
tags: [test]
scope: ["src/**"]
test_refs: ["tests/test.rs"]
---
# Task 001: Bootstrap
**Status**: ACTIVE
## 🎯 Formål
Test
## 📋 Acceptance Criteria
- [x] Test criterion
## 🚫 Must NOT
- None
"#;
    fs::write(tasks_dir.join("001-bootstrap.md"), task_content).unwrap();

    let src_dir = ws.join("src").join("features").join("billing");
    fs::create_dir_all(&src_dir).unwrap();
    let models_code = r#"
/// @domain
pub struct CustomerInvoice {
    pub invoice_id: String,
    pub amount_cents: u64,
}

pub enum PaymentStatus {
    Pending,
    Settled,
}
"#;
    fs::write(src_dir.join("models.rs"), models_code).unwrap();

    let diagram_cfg = match diagram_file {
        Some(df) => format!("diagram_file = \"{df}\"\n"),
        None => String::new(),
    };

    let config_content = format!(
        r#"stack = "rust"
save_evidence = true

[paths]
tasks_dir = "tasks"
spec_file = "spec.md"
context_file = "CONTEXT.md"
coding_standards_file = "CODING_STANDARDS.md"
{diagram_cfg}
[[layers]]
name = "unit"
command = ["echo", "test-ok"]
optional = false
timeout_seconds = 10.0
"#
    );
    fs::write(ws.join("gauntlet.toml"), config_content).unwrap();
    fs::write(ws.join("spec.md"), "# Spec\n").unwrap();
    fs::write(ws.join("CONTEXT.md"), "# Context\n").unwrap();
    fs::write(ws.join("CODING_STANDARDS.md"), "# Standards\n").unwrap();
}

#[test]
fn test_paths_config_default_diagram_file() {
    let cfg = GauntletConfig::default();
    assert_eq!(cfg.paths.diagram_file, "architecture.mermaid");
}

#[tokio::test]
async fn test_verify_save_generates_architecture_mermaid() {
    let ws = TempWorkspace::new("auto_mermaid");
    scaffold_test_workspace(ws.path(), None);

    let options = GauntletOptions {
        task_id: Some("001".to_string()),
        save_evidence: true,
        layers: Some(vec![LayerDefinition::new(
            "echo",
            vec!["echo".to_string(), "ok".to_string()],
        )]),
    };

    let outcome = execute_gauntlet_pipeline(ws.path(), &options)
        .await
        .expect("pipeline execution should succeed");

    assert!(outcome.success, "pipeline should pass");
    assert!(!outcome.is_self_mutated, "should not be self-mutated");

    // The architecture.mermaid file MUST be created automatically
    let mermaid_path = ws.path().join("architecture.mermaid");
    assert!(
        mermaid_path.is_file(),
        "architecture.mermaid should be automatically generated on verify --save"
    );

    let content = fs::read_to_string(&mermaid_path).expect("read mermaid file");
    assert!(content.contains("classDiagram"), "must contain classDiagram");
    assert!(
        content.contains("CustomerInvoice"),
        "must contain CustomerInvoice from models.rs"
    );
    assert!(
        content.contains("PaymentStatus"),
        "must contain PaymentStatus enum"
    );
}

#[tokio::test]
async fn test_verify_save_skips_when_diagram_file_empty() {
    let ws = TempWorkspace::new("disabled_mermaid");
    scaffold_test_workspace(ws.path(), Some(""));

    let options = GauntletOptions {
        task_id: Some("001".to_string()),
        save_evidence: true,
        layers: Some(vec![LayerDefinition::new(
            "echo",
            vec!["echo".to_string(), "ok".to_string()],
        )]),
    };

    let outcome = execute_gauntlet_pipeline(ws.path(), &options)
        .await
        .expect("pipeline execution should succeed");

    assert!(outcome.success);
    assert!(
        !ws.path().join("architecture.mermaid").exists(),
        "when diagram_file is empty, no diagram file should be created"
    );
}
