//! Acceptance tests for Task 057:
//! Deterministic Slice Architecture Gates (Gate A/B/C), Living Slice Specs (`spec_ref`),
//! 4-State ADR Lifecycle (`proposed | accepted | superseded | deprecated`), and Polyglot Skill Docs.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use xgauntlet_core::features::architecture::{run_architecture_check, ArchitectureFindingCategory};
use xgauntlet_core::features::checkpoint::engine::validate_staged_files_scope;
use xgauntlet_core::features::config::{parse_toml, render_toml, GauntletConfig};
use xgauntlet_core::features::plugin::bundle::get_embedded_companion_files;
use xgauntlet_core::features::plugin::get_embedded_skill;
use xgauntlet_core::features::release::adr::check_adr_coverage;
use xgauntlet_core::features::tasks::{
    check_adr_frontmatter, check_task_specification, parse_task_file, ScaffoldTaskOptions,
    TaskScaffolder,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(prefix: &str) -> Self {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("xgauntlet_t057_{prefix}_{pid}_{id}_{nanos}"));
        fs::create_dir_all(&path).expect("create temp workspace");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let full = self.path.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(&full, content).expect("write file");
        full
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

const VALID_CONTEXT_MD: &str = r#"# Domain Glossary

**Bounded Context**:
An explicit linguistic and architectural system boundary in the solution space, within which every domain term carries a single unambiguous meaning.
_Avoid_: Global namespace, monolith module.
"#;

// ============================================================================
// AC1: Konfiguration af Arkitektur-Gates i gauntlet.toml ([architecture])
// ============================================================================

#[test]
fn test_ac1_architecture_config_toml_roundtrip_for_layers_slices_and_exemptions() {
    let ws = TempWorkspace::new("ac1_config");
    let toml_content = r#"
[project]
name = "kant-test"
stack = "rust"

[architecture]
max_file_lines = 800
max_test_file_lines = 1000
max_public_exports = 20
layers = ["app", "features", "core", "shared"]
slice_root = "src/features"
isolate_slices = true
forbid_core_imports = ["iced", "react", "javafx"]

[[architecture.exemptions]]
path = "src/features/legacy/panel.rs"
max_lines = 900
allow_imports = ["crate::app::Message", "crate::features::other::State"]
reason = "Gradual migration to Slice Action pattern (ADR 0012)"
adr = "docs/adr/0012-slice.md"
"#;
    ws.write("gauntlet.toml", toml_content);

    let loaded = GauntletConfig::load(ws.path()).expect("load gauntlet.toml");
    let arch_json = serde_json::to_value(&loaded.architecture).expect("serialize architecture");

    assert_eq!(
        arch_json["layers"],
        json!(["app", "features", "core", "shared"]),
        "ArchitectureConfig must parse 'layers' array from gauntlet.toml"
    );
    assert_eq!(
        arch_json["slice_root"],
        json!("src/features"),
        "ArchitectureConfig must parse 'slice_root' from gauntlet.toml"
    );
    assert_eq!(
        arch_json["isolate_slices"],
        json!(true),
        "ArchitectureConfig must parse 'isolate_slices = true' from gauntlet.toml"
    );
    assert_eq!(
        arch_json["forbid_core_imports"],
        json!(["iced", "react", "javafx"]),
        "ArchitectureConfig must parse 'forbid_core_imports' from gauntlet.toml"
    );
    assert_eq!(
        arch_json["exemptions"][0]["allow_imports"],
        json!(["crate::app::Message", "crate::features::other::State"]),
        "ArchitectureExemption must parse 'allow_imports' from gauntlet.toml"
    );

    // Roundtrip render_toml & parse_toml
    let rendered = render_toml(&loaded);
    let reloaded = parse_toml(&rendered).expect("reparse rendered gauntlet.toml");
    let reloaded_json =
        serde_json::to_value(&reloaded.architecture).expect("serialize reloaded architecture");

    assert_eq!(
        reloaded_json["layers"],
        json!(["app", "features", "core", "shared"])
    );
    assert_eq!(reloaded_json["isolate_slices"], json!(true));
    assert_eq!(
        reloaded_json["forbid_core_imports"],
        json!(["iced", "react", "javafx"])
    );
    assert_eq!(
        reloaded_json["exemptions"][0]["allow_imports"],
        json!(["crate::app::Message", "crate::features::other::State"])
    );
}

// ============================================================================
// AC2: Deterministiske Grænse-Gates (Gate A, Gate B, Gate C)
// ============================================================================

#[test]
fn test_ac2_gate_a_layer_direction_violations_across_rust_ts_and_java() {
    let ws = TempWorkspace::new("ac2_gate_a");
    ws.write(
        "gauntlet.toml",
        r#"
[project]
name = "polyglot-layers"
stack = "rust"

[architecture]
max_file_lines = 800
max_test_file_lines = 1000
max_public_exports = 20
layers = ["app", "features", "core", "shared"]
slice_root = "src/features"
isolate_slices = false
"#,
    );

    // Valid downward imports: app -> features -> core -> shared
    ws.write(
        "src/app/main.rs",
        "use crate::features::diagram::canvas::CanvasAction;\nuse crate::core::model::Graph;\npub fn run() {}\n",
    );
    ws.write(
        "src/shared/math.rs",
        "pub struct Vec2 { pub x: f32, pub y: f32 }\n",
    );

    // 1. Rust violation: features -> app
    ws.write(
        "src/features/diagram/canvas/mod.rs",
        "use crate::app::Message;\nuse crate::shared::math::Vec2;\npub struct CanvasSlice;\n",
    );

    // 2. Rust violation: core -> features
    ws.write(
        "src/core/model.rs",
        "use crate::features::diagram::canvas::CanvasSlice;\npub struct Graph;\n",
    );

    // 3. TypeScript/React violation: features -> app
    ws.write(
        "src/features/diagram/canvas/CanvasView.tsx",
        "import { dispatchApp } from '../../../app/store';\nexport const CanvasView = () => null;\n",
    );

    // 4. Java violation: features -> app
    ws.write(
        "src/features/billing/invoice/InvoiceSlice.java",
        "package com.acme.features.billing.invoice;\nimport com.acme.app.MainController;\npublic class InvoiceSlice {}\n",
    );

    let config = GauntletConfig::load(ws.path()).expect("load config");
    let report =
        run_architecture_check(ws.path(), &config.architecture).expect("run architecture check");

    assert!(
        !report.is_valid,
        "Gate A must fail when lower layers import from higher layers"
    );

    let layer_findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            f.category == ArchitectureFindingCategory::ForbiddenImportDetected
                && f.message.contains("LayerDirectionViolation")
        })
        .collect();

    assert!(
        layer_findings
            .iter()
            .any(|f| f.file_path == "src/features/diagram/canvas/mod.rs"),
        "Must flag Rust features -> app import in src/features/diagram/canvas/mod.rs, got: {:?}",
        report.findings
    );
    assert!(
        layer_findings
            .iter()
            .any(|f| f.file_path == "src/core/model.rs"),
        "Must flag Rust core -> features import in src/core/model.rs"
    );
    assert!(
        layer_findings
            .iter()
            .any(|f| f.file_path == "src/features/diagram/canvas/CanvasView.tsx"),
        "Must flag TS/React features -> app import in CanvasView.tsx"
    );
    assert!(
        layer_findings
            .iter()
            .any(|f| f.file_path == "src/features/billing/invoice/InvoiceSlice.java"),
        "Must flag Java features -> app import in InvoiceSlice.java"
    );
    assert!(
        !layer_findings
            .iter()
            .any(|f| f.file_path == "src/app/main.rs"),
        "Valid top-down imports from app -> features/core must NOT be flagged"
    );
}

#[test]
fn test_ac2_gate_b_cross_slice_isolation_and_internal_slice_freedom() {
    let ws = TempWorkspace::new("ac2_gate_b");
    ws.write(
        "docs/adr/0012-slice.md",
        "---\ntype: Architectural Decision Record\ntitle: \"ADR 0012\"\nstatus: accepted\ngenerated: { by: human:dev, at: \"2026-11-07T10:00:00Z\" }\ntags: [arch]\n---\n# ADR 0012\n",
    );
    ws.write(
        "gauntlet.toml",
        r#"
[project]
name = "slice-isolation"
stack = "rust"

[architecture]
max_file_lines = 800
max_test_file_lines = 1000
max_public_exports = 20
layers = ["app", "features", "core", "shared"]
slice_root = "src/features"
isolate_slices = true

[[architecture.exemptions]]
path = "src/features/legacy/bridge/mod.rs"
max_lines = 800
allow_imports = ["crate::features::diagram::canvas"]
reason = "Legacy bridge awaiting extraction (ADR 0012)"
adr = "docs/adr/0012-slice.md"
"#,
    );

    // Internal imports within the SAME slice (features/diagram/canvas) -> MUST BE 100% ALLOWED
    ws.write(
        "src/features/diagram/canvas/mod.rs",
        "pub mod state;\npub mod view;\nuse crate::features::diagram::canvas::state::CanvasState;\npub struct CanvasAction;\n",
    );
    ws.write(
        "src/features/diagram/canvas/state.rs",
        "pub struct CanvasState { pub zoom: f32 }\n",
    );
    ws.write(
        "src/features/diagram/canvas/view.rs",
        "use super::state::CanvasState;\npub fn render(_s: &CanvasState) {}\n",
    );

    // Violation 1: Cross-slice import within same domain (diagram/inspector -> diagram/canvas)
    ws.write(
        "src/features/diagram/inspector/mod.rs",
        "use crate::features::diagram::canvas::state::CanvasState;\npub struct InspectorSlice;\n",
    );

    // Violation 2: Cross-slice import across domains in TypeScript (export/pdf -> diagram/canvas)
    ws.write(
        "src/features/export/pdf/ExportSlice.ts",
        "import { CanvasState } from '../../diagram/canvas/state';\nexport const exportPdf = () => {};\n",
    );

    // Exempted cross-slice import -> MUST PASS
    ws.write(
        "src/features/legacy/bridge/mod.rs",
        "use crate::features::diagram::canvas::state::CanvasState;\npub fn bridged() {}\n",
    );

    let config = GauntletConfig::load(ws.path()).expect("load config");
    let report =
        run_architecture_check(ws.path(), &config.architecture).expect("run architecture check");

    assert!(
        !report.is_valid,
        "Gate B must fail when slices import directly across slice boundaries"
    );

    let slice_findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            f.category == ArchitectureFindingCategory::ForbiddenImportDetected
                && f.message.contains("CrossSliceImportViolation")
        })
        .collect();

    assert!(
        slice_findings
            .iter()
            .any(|f| f.file_path == "src/features/diagram/inspector/mod.rs"),
        "Must flag cross-slice import from diagram/inspector to diagram/canvas"
    );
    assert!(
        slice_findings
            .iter()
            .any(|f| f.file_path == "src/features/export/pdf/ExportSlice.ts"),
        "Must flag cross-slice relative import in TypeScript ExportSlice.ts"
    );
    assert!(
        !slice_findings
            .iter()
            .any(|f| f.file_path.starts_with("src/features/diagram/canvas/")),
        "Internal imports within the same slice (diagram/canvas) must NEVER be flagged"
    );
    assert!(
        !slice_findings
            .iter()
            .any(|f| f.file_path == "src/features/legacy/bridge/mod.rs"),
        "Exempted allow_imports entry must NOT be flagged"
    );
}

#[test]
fn test_ac2_gate_c_clean_core_forbids_ui_framework_imports() {
    let ws = TempWorkspace::new("ac2_gate_c");
    ws.write(
        "gauntlet.toml",
        r#"
[project]
name = "clean-core"
stack = "rust"

[architecture]
max_file_lines = 800
max_test_file_lines = 1000
max_public_exports = 20
layers = ["app", "features", "core", "shared"]
slice_root = "src/features"
isolate_slices = true
forbid_core_imports = ["iced", "react", "javafx"]
"#,
    );

    // Violations in core/
    ws.write(
        "src/core/graph.rs",
        "use iced::widget::canvas;\npub struct GraphCore;\n",
    );
    ws.write(
        "src/core/hooks.ts",
        "import { useState } from 'react';\nexport const useCore = () => {};\n",
    );
    ws.write(
        "src/core/DomainNode.java",
        "package com.acme.core;\nimport javafx.scene.Node;\npublic class DomainNode {}\n",
    );

    // Allowed UI import in features/
    ws.write(
        "src/features/diagram/canvas/view.rs",
        "use iced::widget::canvas;\npub fn view() {}\n",
    );

    let config = GauntletConfig::load(ws.path()).expect("load config");
    let report =
        run_architecture_check(ws.path(), &config.architecture).expect("run architecture check");

    assert!(
        !report.is_valid,
        "Gate C must fail when core imports forbidden UI/framework packages"
    );

    let core_findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            f.category == ArchitectureFindingCategory::ForbiddenImportDetected
                && f.message.contains("ForbiddenCoreImportViolation")
        })
        .collect();

    assert!(
        core_findings
            .iter()
            .any(|f| f.file_path == "src/core/graph.rs"),
        "Must flag iced import inside src/core/graph.rs"
    );
    assert!(
        core_findings
            .iter()
            .any(|f| f.file_path == "src/core/hooks.ts"),
        "Must flag react import inside src/core/hooks.ts"
    );
    assert!(
        core_findings
            .iter()
            .any(|f| f.file_path == "src/core/DomainNode.java"),
        "Must flag javafx import inside src/core/DomainNode.java"
    );
    assert!(
        !core_findings
            .iter()
            .any(|f| f.file_path.starts_with("src/features/")),
        "UI framework imports inside src/features/ must be permitted"
    );
}

// ============================================================================
// AC3: Deterministisk Domain -> Slice -> Task Mapning i check-spec & checkpoint
// ============================================================================

#[test]
fn test_ac3_domain_slice_task_spec_ref_validation_and_done_exemption() {
    let ws = TempWorkspace::new("ac3_slice_spec");
    ws.write("CONTEXT.md", VALID_CONTEXT_MD);
    ws.write(
        "specs/diagram/canvas.md",
        "# Slice Specification: diagram / canvas\n\n## Capabilities\n- Zoom and pan canvas.\n",
    );
    ws.write("crates/app/tests/canvas_test.rs", "#[test]\nfn t() {}\n");

    // 1. Valid active task with domain, slice, and matching existing spec_ref
    let valid_task = ws.write(
        "tasks/010-canvas-zoom.md",
        r#"---
type: Task Package
title: "Task 010: Canvas Zoom"
description: "Add zoom to canvas slice"
status: active
intent: feature
domain: diagram
slice: canvas
spec_ref: "specs/diagram/canvas.md"
generated: { by: process:xgauntlet, at: "2026-11-07T10:00:00Z" }
tags: [diagram, canvas]
scope: ["src/features/diagram/canvas/**", "specs/diagram/canvas.md"]
test_refs: ["crates/app/tests/canvas_test.rs"]
---

# Task 010: Canvas Zoom

## 🎯 Formål
Zoom support.

## 📋 Acceptance Criteria
- [ ] AC1: Zoom works.

## 🚫 Must NOT
- Must not import app.
"#,
    );

    let rep_valid = check_task_specification(&valid_task, ws.path());
    assert!(
        rep_valid.is_valid,
        "Valid task with domain, slice, and existing spec_ref must pass, diagnostics: {:?}",
        rep_valid.diagnostics
    );

    let parsed = parse_task_file(&valid_task).expect("parse valid task");
    let parsed_json = serde_json::to_value(&parsed).expect("serialize TaskPackageInfo");
    assert_eq!(parsed_json["domain"], json!("diagram"));
    assert_eq!(parsed_json["slice"], json!("canvas"));
    assert_eq!(parsed_json["spec_ref"], json!("specs/diagram/canvas.md"));

    // 2. Active feature task in a workspace with specs/ directory but missing domain/slice/spec_ref
    let missing_mapping_task = ws.write(
        "tasks/011-unmapped-feature.md",
        r#"---
type: Task Package
title: "Task 011: Unmapped Feature"
description: "Missing slice mapping"
status: active
intent: feature
generated: { by: process:xgauntlet, at: "2026-11-07T10:00:00Z" }
tags: [diagram]
scope: ["src/features/diagram/canvas/**"]
test_refs: ["crates/app/tests/canvas_test.rs"]
---

# Task 011: Unmapped Feature

## 📋 Acceptance Criteria
- [ ] AC1: Works.

## 🚫 Must NOT
- Must not break.
"#,
    );
    let rep_missing = check_task_specification(&missing_mapping_task, ws.path());
    assert!(
        !rep_missing.is_valid,
        "Active feature task in workspace with specs/ must fail when domain/slice/spec_ref are missing"
    );
    assert!(
        rep_missing
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "MISSING_TASK_SLICE_MAPPING"),
        "Expected MISSING_TASK_SLICE_MAPPING diagnostic, got: {:?}",
        rep_missing.diagnostics
    );

    // 3. Active task where spec_ref does not match specs/<domain>/<slice>.md
    let mismatched_ref_task = ws.write(
        "tasks/012-mismatched-spec-ref.md",
        r#"---
type: Task Package
title: "Task 012: Mismatched Spec Ref"
description: "Wrong spec_ref path"
status: active
intent: enhancement
domain: diagram
slice: canvas
spec_ref: "specs/other/wrong.md"
generated: { by: process:xgauntlet, at: "2026-11-07T10:00:00Z" }
tags: [diagram]
scope: ["src/features/diagram/canvas/**"]
test_refs: ["crates/app/tests/canvas_test.rs"]
---

# Task 012: Mismatched Spec Ref

## 📋 Acceptance Criteria
- [ ] AC1: Works.

## 🚫 Must NOT
- Must not break.
"#,
    );
    let rep_mismatch = check_task_specification(&mismatched_ref_task, ws.path());
    assert!(!rep_mismatch.is_valid);
    assert!(
        rep_mismatch
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "INVALID_TASK_SPEC_REF"),
        "Expected INVALID_TASK_SPEC_REF diagnostic, got: {:?}",
        rep_mismatch.diagnostics
    );

    // 4. Active task where specs/<domain>/<slice>.md file does not exist on disk
    let missing_file_task = ws.write(
        "tasks/013-missing-spec-file.md",
        r#"---
type: Task Package
title: "Task 013: Missing Spec File"
description: "Spec file not created yet"
status: active
intent: bug
domain: diagram
slice: nonexistent
spec_ref: "specs/diagram/nonexistent.md"
generated: { by: process:xgauntlet, at: "2026-11-07T10:00:00Z" }
tags: [diagram]
scope: ["src/features/diagram/nonexistent/**"]
test_refs: ["crates/app/tests/canvas_test.rs"]
---

# Task 013: Missing Spec File

## 📋 Acceptance Criteria
- [ ] AC1: Works.

## 🚫 Must NOT
- Must not break.
"#,
    );
    let rep_missing_file = check_task_specification(&missing_file_task, ws.path());
    assert!(!rep_missing_file.is_valid);
    assert!(
        rep_missing_file
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "MISSING_SLICE_SPEC_FILE"),
        "Expected MISSING_SLICE_SPEC_FILE diagnostic, got: {:?}",
        rep_missing_file.diagnostics
    );

    // 5. Completed task (status: done) WITHOUT domain/slice/spec_ref -> MUST PASS (100% backward compatible)
    let done_task = ws.write(
        "tasks/001-legacy-done.md",
        r#"---
type: Task Package
title: "Task 001: Legacy Done Task"
description: "Historical completed task without slice fields"
status: done
intent: feature
generated: { by: process:xgauntlet, at: "2026-09-01T10:00:00Z" }
tags: [legacy]
---

# Task 001: Legacy Done Task

## 🎯 Formål
Historical purpose.

## 📋 Acceptance Criteria
- [x] AC1: Historical criterion.

## 🚫 Must NOT
- Must not break.
"#,
    );
    let rep_done = check_task_specification(&done_task, ws.path());
    assert!(
        rep_done.is_valid,
        "status: done tasks must remain 100% exempt from new slice mapping requirements, got: {:?}",
        rep_done.diagnostics
    );

    // 6. Checkpoint scope validation allows specs/ and docs/
    let staged = vec![
        "specs/diagram/canvas.md".to_string(),
        "docs/adr/0012-test.md".to_string(),
    ];
    let scope_res = validate_staged_files_scope(ws.path(), "011-unmapped-feature", &staged);
    assert!(
        scope_res.is_ok(),
        "validate_staged_files_scope must permit staging files under specs/ and docs/, got: {:?}",
        scope_res
    );

    // 7. TaskScaffolder with domain/slice ("<domain>/<slice>:<slug>") scaffolds task + specs/<domain>/<slice>.md
    let scaffold_opts = ScaffoldTaskOptions {
        name: "diagram/minimap:viewport-sync".to_string(),
        title: Some("Viewport Sync".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Sync minimap viewport".to_string()),
        workspace: ws.path().to_path_buf(),
        force: true,
    };
    let scaffolded = TaskScaffolder::scaffold(&scaffold_opts).expect("scaffold slice task");
    let scaffolded_info = parse_task_file(&scaffolded.path).expect("parse scaffolded task");
    let scaffolded_json = serde_json::to_value(&scaffolded_info).expect("serialize scaffolded");
    assert_eq!(scaffolded_json["domain"], json!("diagram"));
    assert_eq!(scaffolded_json["slice"], json!("minimap"));
    assert_eq!(
        scaffolded_json["spec_ref"],
        json!("specs/diagram/minimap.md")
    );
    assert!(
        ws.path().join("specs/diagram/minimap.md").is_file(),
        "TaskScaffolder must create specs/<domain>/<slice>.md when scaffolding a slice-bound task"
    );
}

// ============================================================================
// AC4: ADR-livscyklus i check-adr og check-release
// ============================================================================

#[test]
fn test_ac4_adr_lifecycle_statuses_and_release_coverage_filtering() {
    let ws = TempWorkspace::new("ac4_adr_lifecycle");

    // 1. Accepted ADR
    let adr_accepted = ws.write(
        "docs/adr/0001-core-architecture.md",
        r#"---
type: Architectural Decision Record
title: "ADR 0001: Core Architecture"
status: accepted
generated: { by: human:architect, at: "2026-11-07T10:00:00Z" }
tags: [architecture]
---
# ADR 0001
"#,
    );
    assert!(check_adr_frontmatter(&adr_accepted).is_valid);

    // 2. Proposed ADR
    let adr_proposed = ws.write(
        "docs/adr/0002-proposed-idea.md",
        r#"---
type: Architectural Decision Record
title: "ADR 0002: Proposed Idea"
status: proposed
generated: { by: human:architect, at: "2026-11-07T10:00:00Z" }
tags: [architecture]
---
# ADR 0002
"#,
    );
    assert!(check_adr_frontmatter(&adr_proposed).is_valid);

    // 3. Deprecated ADR
    let adr_deprecated = ws.write(
        "docs/adr/0003-deprecated-legacy.md",
        r#"---
type: Architectural Decision Record
title: "ADR 0003: Deprecated Legacy"
status: deprecated
generated: { by: human:architect, at: "2026-11-07T10:00:00Z" }
tags: [architecture]
---
# ADR 0003
"#,
    );
    assert!(check_adr_frontmatter(&adr_deprecated).is_valid);

    // 4. Superseded ADR WITH superseded_by -> VALID
    let adr_superseded_ok = ws.write(
        "docs/adr/0004-superseded-old.md",
        r#"---
type: Architectural Decision Record
title: "ADR 0004: Superseded Old"
status: superseded
superseded_by: "0001-core-architecture.md"
generated: { by: human:architect, at: "2026-11-07T10:00:00Z" }
tags: [architecture]
---
# ADR 0004
"#,
    );
    let rep_sup_ok = check_adr_frontmatter(&adr_superseded_ok);
    assert!(
        rep_sup_ok.is_valid,
        "Superseded ADR with superseded_by must pass, got: {:?}",
        rep_sup_ok.diagnostics
    );

    // 5. Superseded ADR WITHOUT superseded_by -> MUST FAIL with MISSING_ADR_SUPERSEDED_BY
    let adr_superseded_missing = ws.write(
        "docs/adr/0005-superseded-missing-target.md",
        r#"---
type: Architectural Decision Record
title: "ADR 0005: Superseded Without Target"
status: superseded
generated: { by: human:architect, at: "2026-11-07T10:00:00Z" }
tags: [architecture]
---
# ADR 0005
"#,
    );
    let rep_sup_missing = check_adr_frontmatter(&adr_superseded_missing);
    assert!(
        !rep_sup_missing.is_valid,
        "Superseded ADR without superseded_by must fail validation"
    );
    assert!(
        rep_sup_missing
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "MISSING_ADR_SUPERSEDED_BY"),
        "Expected MISSING_ADR_SUPERSEDED_BY diagnostic, got: {:?}",
        rep_sup_missing.diagnostics
    );

    // 6. Invalid ADR status -> MUST FAIL with INVALID_ADR_STATUS
    let adr_invalid_status = ws.write(
        "docs/adr/0006-invalid-status.md",
        r#"---
type: Architectural Decision Record
title: "ADR 0006: Invalid Status"
status: zombie_forever
generated: { by: human:architect, at: "2026-11-07T10:00:00Z" }
tags: [architecture]
---
# ADR 0006
"#,
    );
    let rep_invalid = check_adr_frontmatter(&adr_invalid_status);
    assert!(
        !rep_invalid.is_valid,
        "ADR with unrecognized status must fail validation"
    );
    assert!(
        rep_invalid
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "INVALID_ADR_STATUS"),
        "Expected INVALID_ADR_STATUS diagnostic, got: {:?}",
        rep_invalid.diagnostics
    );

    // Clean up invalid test files before testing release coverage
    let _ = fs::remove_file(&adr_superseded_missing);
    let _ = fs::remove_file(&adr_invalid_status);

    // 7. Release ADR coverage: ONLY 'accepted' ADRs are required in README.md / spec.md
    ws.write(
        "README.md",
        "# Project\nFollows [ADR 0001](docs/adr/0001-core-architecture.md).\n",
    );
    ws.write("spec.md", "# Spec\n");

    let cov = check_adr_coverage(ws.path()).expect("check_adr_coverage");
    assert!(
        cov.unreferenced_adrs.is_empty(),
        "proposed, deprecated, and superseded ADRs must NOT be required in README.md/spec.md during check-release, but found unreferenced: {:?}",
        cov.unreferenced_adrs
    );

    // Adding an unreferenced 'accepted' ADR must be caught
    ws.write(
        "docs/adr/0007-unlinked-accepted.md",
        r#"---
type: Architectural Decision Record
title: "ADR 0007: Unlinked Accepted"
status: accepted
generated: { by: human:architect, at: "2026-11-07T10:00:00Z" }
tags: [architecture]
---
# ADR 0007
"#,
    );
    let cov_after = check_adr_coverage(ws.path()).expect("check_adr_coverage after");
    assert_eq!(
        cov_after.unreferenced_adrs,
        vec!["docs/adr/0007-unlinked-accepted.md".to_string()],
        "Unreferenced accepted ADR must still be flagged by check_adr_coverage"
    );
}

// ============================================================================
// AC5: Pædagogisk Polyglot Dokumentation & Sovereign Skills
// ============================================================================

#[test]
fn test_ac5_sovereign_skills_and_coding_standards_document_slice_taxonomy_and_polyglot_examples() {
    let discover_skill = get_embedded_skill("discover").expect("discover skill");
    let spec_skill = get_embedded_skill("spec").expect("spec skill");
    let tdd_skill = get_embedded_skill("tdd").expect("tdd skill");
    let review_skill = get_embedded_skill("review").expect("review skill");
    let release_skill = get_embedded_skill("release").expect("release skill");

    // 1. discover skill: Bounded Context -> Domain -> Slice & ADR Gatekeeper
    assert!(
        discover_skill.contains("Bounded Context")
            && discover_skill.contains("Domain")
            && discover_skill.contains("Slice"),
        "discover/SKILL.md must explain the Bounded Context -> Domain -> Slice taxonomy"
    );
    assert!(
        discover_skill.contains("specs/<domain>/<slice>.md"),
        "discover/SKILL.md must explain that functional slice behavior belongs in specs/<domain>/<slice>.md, not ADRs"
    );

    // 2. ADR-FORMAT.md: 4 statuses and superseded_by
    let companions = get_embedded_companion_files();
    let (_, _, adr_format) = companions
        .iter()
        .find(|(skill, file, _)| *skill == "discover" && *file == "ADR-FORMAT.md")
        .expect("ADR-FORMAT.md companion");
    assert!(
        adr_format.contains("proposed")
            && adr_format.contains("accepted")
            && adr_format.contains("superseded")
            && adr_format.contains("deprecated")
            && adr_format.contains("superseded_by"),
        "ADR-FORMAT.md must document the 4 ADR statuses and superseded_by field"
    );

    // 3. spec skill: 4 layers (app -> features -> core -> shared) and spec_ref
    assert!(
        spec_skill.contains("app -> features -> core -> shared") && spec_skill.contains("spec_ref"),
        "spec/SKILL.md must document the 4 universal layers and spec_ref task mapping"
    );

    // 4. tdd skill: Stateful UI-Slice & Stateless Pipeline-Slice across Rust/Iced, TS/React, Java
    assert!(
        tdd_skill.contains("Stateful UI-Slice")
            && tdd_skill.contains("Stateless Pipeline-Slice")
            && tdd_skill.contains("React")
            && tdd_skill.contains("Java"),
        "tdd/SKILL.md must document Stateful UI-Slice and Stateless Pipeline-Slice with polyglot examples (Rust/Iced, TS/React, Java)"
    );

    // 5. review & release skills: Boundary gates and ADR lifecycle
    assert!(
        review_skill.contains("LayerDirectionViolation")
            || review_skill.contains("CrossSliceImportViolation")
            || review_skill.contains("app -> features -> core -> shared"),
        "review/SKILL.md must include slice and layer boundary checks on Axis 2"
    );
    assert!(
        release_skill.contains("superseded") && release_skill.contains("deprecated"),
        "release/SKILL.md must document ADR lifecycle hygiene for superseded and deprecated ADRs"
    );
}
