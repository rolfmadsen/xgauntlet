//! Acceptance tests for Task 095:
//! 1-Click Mermaid DX, Feature-Scoped Diagrams, Pele HTML Viewer & Cockpit HUD Links.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use xgauntlet_core::features::evidence::manifest::EXCLUDED_FILES;
use xgauntlet_core::features::gauntlet::{
    execute_gauntlet_pipeline, GauntletOptions, LayerDefinition,
};
use xgauntlet_core::features::tasks::inspect_task_telemetry_for_session;
use xgauntlet_core::features::topology::{
    export_feature_mermaid_class_diagram, pele_playground_url, render_architecture_markdown,
    render_pele_viewer_html, scan_workspace_topology, MermaidExportOptions, MermaidLevel,
    TopologyOptions,
};
use xgauntlet_core::AntigravityAdapter;

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
        let path = std::env::temp_dir().join(format!("xgauntlet_mermaid_dx_{unique}"));
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

fn scaffold_multi_feature_workspace(ws: &Path) {
    fs::create_dir_all(ws.join("tasks")).unwrap();
    fs::write(
        ws.join("tasks/001-billing-dx.md"),
        r#"---
type: Task Package
title: "Task 001: Billing DX"
description: "Billing feature task"
status: active
generated: { by: process:test, at: "2026-11-15T12:00:00Z" }
tags: [billing]
feature: billing
spec_ref: "specs/billing.md"
scope:
  - "src/features/billing/**"
---
# Task 001: Billing DX
**Status**: `ACTIVE`
## 🎯 Formål
Billing test
## 📋 Acceptance Criteria
- [x] Criterion 1
## 🚫 Must NOT
- None
"#,
    )
    .unwrap();

    fs::create_dir_all(ws.join("src/features/shared")).unwrap();
    fs::write(
        ws.join("src/features/shared/models.rs"),
        r#"
pub struct CurrencyCode {
    pub iso: String,
}
"#,
    )
    .unwrap();

    fs::create_dir_all(ws.join("src/features/billing")).unwrap();
    fs::write(
        ws.join("src/features/billing/models.rs"),
        r#"
/// @domain
pub struct CustomerInvoice {
    pub invoice_id: String,
    pub currency: CurrencyCode,
    pub status: PaymentStatus,
}

pub enum PaymentStatus {
    Pending,
    Settled,
}
"#,
    )
    .unwrap();

    fs::create_dir_all(ws.join("src/features/shipping")).unwrap();
    fs::write(
        ws.join("src/features/shipping/models.rs"),
        r#"
/// @domain
pub struct FreightParcel {
    pub tracking_id: String,
}
"#,
    )
    .unwrap();

    fs::write(
        ws.join("gauntlet.toml"),
        r#"stack = "rust"
save_evidence = true

[paths]
tasks_dir = "tasks"
spec_file = "spec.md"
context_file = "CONTEXT.md"
coding_standards_file = "CODING_STANDARDS.md"
diagram_file = "architecture.mermaid"

[[layers]]
name = "unit"
command = ["echo", "ok"]
optional = false
timeout_seconds = 10.0
"#,
    )
    .unwrap();
    fs::write(ws.join("spec.md"), "# Spec\n").unwrap();
    fs::write(ws.join("CONTEXT.md"), "# Context\n").unwrap();
    fs::write(ws.join("CODING_STANDARDS.md"), "# Standards\n").unwrap();
}

#[test]
fn test_ac1_feature_scoped_mermaid_includes_neighbors_and_excludes_unrelated_features() {
    let ws = TempWorkspace::new("ac1_scope");
    scaffold_multi_feature_workspace(ws.path());

    let graph = scan_workspace_topology(&TopologyOptions {
        workspace_root: ws.path().to_path_buf(),
        ..Default::default()
    })
    .expect("scan topology");

    let opts = MermaidExportOptions {
        direction: "TB".to_string(),
        level: MermaidLevel::Conceptual,
        domain_only: true,
    };

    let billing_mermaid = export_feature_mermaid_class_diagram(&graph, &opts, "billing")
        .expect("billing feature export should succeed");

    assert!(billing_mermaid.contains("classDiagram"));
    assert!(billing_mermaid.contains("CustomerInvoice"));
    assert!(billing_mermaid.contains("PaymentStatus"));
    // Direct 1-hop dependency from shared/models.rs must be preserved for relationship completeness
    assert!(
        billing_mermaid.contains("CurrencyCode"),
        "expected 1-hop neighbor CurrencyCode in:\n{billing_mermaid}"
    );
    // Unrelated feature model must be excluded
    assert!(
        !billing_mermaid.contains("FreightParcel"),
        "unrelated shipping model FreightParcel should not appear in billing diagram:\n{billing_mermaid}"
    );

    let err = export_feature_mermaid_class_diagram(&graph, &opts, "nonexistent_feature")
        .expect_err("unknown feature must return an actionable error");
    assert!(
        err.to_string().contains("nonexistent_feature"),
        "error should mention missing feature name: {err}"
    );
}

#[tokio::test]
async fn test_ac2_verify_save_syncs_architecture_md_and_feature_diagrams_without_tamper() {
    let ws = TempWorkspace::new("ac2_sync");
    scaffold_multi_feature_workspace(ws.path());

    assert!(
        EXCLUDED_FILES.contains(&"architecture.md"),
        "architecture.md must be excluded from anti-tamper manifest"
    );

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
        .expect("pipeline should succeed");
    assert!(outcome.success);
    assert!(!outcome.is_self_mutated);

    // 1. Canonical architecture.mermaid remains intact
    assert!(ws.path().join("architecture.mermaid").is_file());

    // 2. architecture.md wraps Mermaid cleanly for GitHub / IDE Markdown Preview
    let arch_md = fs::read_to_string(ws.path().join("architecture.md"))
        .expect("architecture.md must be generated on verify --save");
    assert!(arch_md.contains("```mermaid"));
    assert!(arch_md.contains("classDiagram"));
    assert!(arch_md.contains("https://pele.run/playground#source="));
    assert!(!arch_md.contains("mermaid.live"));
    assert!(arch_md.contains("docs/architecture/billing.md"));
    assert!(arch_md.contains("docs/architecture/shipping.md"));

    // 3. Per-feature markdown files exist under docs/architecture/
    let billing_md = fs::read_to_string(ws.path().join("docs/architecture/billing.md"))
        .expect("docs/architecture/billing.md must be generated");
    assert!(billing_md.contains("```mermaid"));
    assert!(billing_md.contains("https://pele.run/playground#source="));
    assert!(!billing_md.contains("mermaid.live"));
    assert!(billing_md.contains("CustomerInvoice"));
    assert!(!billing_md.contains("FreightParcel"));

    // Render helper sanity check
    let custom_md =
        render_architecture_markdown("classDiagram\n    class Foo\n", &["billing".into()]);
    assert!(custom_md.contains("```mermaid"));
    assert!(custom_md.contains("docs/architecture/billing.md"));
}

#[test]
fn test_ac3_pele_viewer_html_contains_pan_zoom_theme_tokens_and_offline_fallback() {
    let mermaid_src =
        "classDiagram\n    class CustomerInvoice {\n        +invoice_id: String\n    }\n";
    let html = render_pele_viewer_html(mermaid_src, Some("billing"), "Conceptual");

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(
        html.contains("pele") || html.contains("obsidianmd/pele"),
        "HTML viewer must integrate Pele renderer"
    );
    assert!(
        html.contains("https://cdn.jsdelivr.net/npm/pele@1/dist/pele.js"),
        "HTML viewer must import official Pele v1 ESM entrypoint"
    );
    assert!(html.contains("CustomerInvoice"));
    assert!(html.contains("prefers-color-scheme"));
    assert!(html.contains("--bg-surface"));
    assert!(html.contains("zoom"));
    assert!(html.contains("copy"));
    assert!(
        html.contains("mermaid-fallback") || html.contains("fallback"),
        "HTML viewer must provide offline/error raw Mermaid fallback"
    );
}

#[test]
fn test_ac4_cockpit_hud_links_include_scoped_feature_diagram_or_architecture_md() {
    let ws = TempWorkspace::new("ac4_hud");
    scaffold_multi_feature_workspace(ws.path());

    // Without docs/architecture/billing.md yet, but with architecture.md present
    fs::write(ws.path().join("architecture.md"), "# Arch\n").unwrap();
    let telemetry_global = inspect_task_telemetry_for_session(ws.path(), Some("001"), None)
        .expect("inspect telemetry");
    let hud_global = AntigravityAdapter::render_blockquote_hud(&telemetry_global, None);
    assert!(
        hud_global.contains("🗺️ [Arch](architecture.md)"),
        "expected global architecture.md link in HUD:\n{hud_global}"
    );

    // Once scoped docs/architecture/billing.md exists, HUD prefers the feature diagram
    fs::create_dir_all(ws.path().join("docs/architecture")).unwrap();
    fs::write(
        ws.path().join("docs/architecture/billing.md"),
        "# Billing\n",
    )
    .unwrap();
    let telemetry_scoped = inspect_task_telemetry_for_session(ws.path(), Some("001"), None)
        .expect("inspect scoped telemetry");
    let hud_scoped = AntigravityAdapter::render_blockquote_hud(&telemetry_scoped, None);
    assert!(
        hud_scoped.contains("🗺️ [Arch](docs/architecture/billing.md)"),
        "expected feature-scoped architecture link in HUD:\n{hud_scoped}"
    );
}

#[test]
fn test_task096_pele_playground_url_matches_empirical_share_links() {
    let flowchart_source = "flowchart TD\n  A([Mermaid text]) --> B{Supported?}\n  B -- Yes --> C[Parse]\n  B -- No --> H[Fall back]\n\n  subgraph Pele\n    C --> D[Layout]\n    D --> E[Render]\n  end\n\n  E --> F[(SVG)]\n  F -.-> G[Theme with CSS]\n";
    let expected_flowchart_url = "https://pele.run/playground#source=flowchart+TD%0A++A%28%5BMermaid+text%5D%29+--%3E+B%7BSupported%3F%7D%0A++B+--+Yes+--%3E+C%5BParse%5D%0A++B+--+No+--%3E+H%5BFall+back%5D%0A%0A++subgraph+Pele%0A++++C+--%3E+D%5BLayout%5D%0A++++D+--%3E+E%5BRender%5D%0A++end%0A%0A++E+--%3E+F%5B%28SVG%29%5D%0A++F+-.-%3E+G%5BTheme+with+CSS%5D%0A";
    assert_eq!(
        pele_playground_url(flowchart_source),
        expected_flowchart_url
    );

    let class_diagram_source = "---\nconfig:\n  class:\n    hierarchicalNamespaces: false\n---\nclassDiagram\n    namespace Earth.Volcano.Summit {\n        class Crater {\n            +erupt()\n        }\n    }\n    namespace Earth.Volcano.Slope {\n        class LavaFlow {\n            +buildLand()\n        }\n    }\n    namespace Earth {\n        class Hotspot {\n            +meltRock()\n        }\n    }\n    Hotspot --> Crater : feeds\n    Hotspot --> LavaFlow : feeds\n";
    let expected_class_url = "https://pele.run/playground#source=---%0Aconfig%3A%0A++class%3A%0A++++hierarchicalNamespaces%3A+false%0A---%0AclassDiagram%0A++++namespace+Earth.Volcano.Summit+%7B%0A++++++++class+Crater+%7B%0A++++++++++++%2Berupt%28%29%0A++++++++%7D%0A++++%7D%0A++++namespace+Earth.Volcano.Slope+%7B%0A++++++++class+LavaFlow+%7B%0A++++++++++++%2BbuildLand%28%29%0A++++++++%7D%0A++++%7D%0A++++namespace+Earth+%7B%0A++++++++class+Hotspot+%7B%0A++++++++++++%2BmeltRock%28%29%0A++++++++%7D%0A++++%7D%0A++++Hotspot+--%3E+Crater+%3A+feeds%0A++++Hotspot+--%3E+LavaFlow+%3A+feeds%0A";
    assert_eq!(
        pele_playground_url(class_diagram_source),
        expected_class_url
    );
}
