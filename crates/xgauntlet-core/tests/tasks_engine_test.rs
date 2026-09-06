//! Black-box integration tests for Task & Spec Engine (features/tasks)
//! and Diagnostics Engine (features/diagnostics) in xgauntlet-core.

use std::fs;
use std::path::Path;
use xgauntlet_core::features::tasks::{
    check_task_specification, has_active_task, is_task_active, parse_frontmatter,
    parse_task_content, parse_task_status, resolve_active_task_id, resolve_task_contract,
    validate_context_content, validate_context_glossary, Actor, TaskStatus,
};

#[test]
fn test_actor_parsing_valid_formats() {
    let human = Actor::parse("human:alice").expect("valid human actor");
    assert_eq!(human.kind, "human");
    assert_eq!(human.identifier, "alice");

    let process = Actor::parse("process:xgauntlet-init").expect("valid process actor");
    assert_eq!(process.kind, "process");
    assert_eq!(process.identifier, "xgauntlet-init");

    let agent = Actor::parse("antigravity/v2.1").expect("valid agent actor");
    assert_eq!(agent.kind, "agent");
    assert_eq!(agent.namespace.as_deref(), Some("antigravity"));
    assert_eq!(agent.identifier, "v2.1");
}

#[test]
fn test_actor_parsing_invalid_formats() {
    assert!(Actor::parse("").is_err());
    assert!(Actor::parse("human:").is_err());
    assert!(Actor::parse("human:alice/bob").is_err());
    assert!(Actor::parse("process:").is_err());
    assert!(Actor::parse("invalid_no_colon_or_slash").is_err());
}

#[test]
fn test_okf_frontmatter_parsing_and_validation() {
    let markdown = r#"---
type: Task Package
title: "Sample Task"
description: "A sample task description"
status: stable
generated: { by: process:test-gen, at: "2026-09-06T12:00:00Z" }
tags: [test, spec, rust]
---

# Sample Task

Content goes here.
"#;

    let (metadata_opt, body) = parse_frontmatter(markdown).expect("should parse frontmatter");
    let meta = metadata_opt.expect("metadata must be present");

    assert_eq!(meta.doc_type, "Task Package");
    assert_eq!(meta.title.as_deref(), Some("Sample Task"));
    assert_eq!(
        meta.description.as_deref(),
        Some("A sample task description")
    );
    assert_eq!(meta.status.as_deref(), Some("stable"));
    assert_eq!(meta.tags, vec!["test", "spec", "rust"]);

    let gen = meta.generated.expect("generated block required");
    assert_eq!(gen.by, "process:test-gen");
    assert_eq!(gen.at.as_deref(), Some("2026-09-06T12:00:00Z"));

    assert!(body.contains("# Sample Task"));
}

#[test]
fn test_task_status_parsing_and_active_detection() {
    let content_active = r#"---
type: Task Package
status: active
---
# Task 001: Active Task
**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`

## 🎯 Formål
Testing active detection.

## 📋 Acceptance Criteria
- [ ] Criterion 1
- [x] Criterion 2

## 🚫 Must NOT
- Must not fail
"#;

    assert_eq!(parse_task_status(content_active), TaskStatus::Active);
    assert!(is_task_active(content_active));

    let content_done = r#"---
type: Task Package
status: stable
---
# Task 002: Done Task
**Status**: `DONE`

## 📋 Acceptance Criteria
- [x] All done
"#;

    assert_eq!(parse_task_status(content_done), TaskStatus::Done);
    assert!(!is_task_active(content_done));

    let content_draft_no_criteria = r#"
# Task 003: Draft
**Status**: `DRAFT`

## 🎯 Formål
Planning only.
"#;

    assert_eq!(
        parse_task_status(content_draft_no_criteria),
        TaskStatus::Draft
    );
    assert!(!is_task_active(content_draft_no_criteria));
}

#[test]
fn test_parse_task_package_info() {
    let content = r#"---
type: Task Package
title: "Task 042: Deep Feature"
status: active
generated: { by: human:developer, at: "2026-09-06T15:00:00Z" }
tags: [feature, deep]
---

# Task 042: Deep Feature

**Status**: `IN_PROGRESS`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Build deep feature module with zero ambient authority.

## 📋 Acceptance Criteria
- [x] First criterion completed
- [ ] Second criterion pending
- [ ] Third criterion pending

## 🚫 Must NOT
- Must NOT introduce background daemons
- Must NOT leak ambient authority

## 📝 Revisions
- Initial commit

## 🧪 Verifikation
- cargo test
"#;

    let info = parse_task_content(content, "042-deep-feature").expect("parse should succeed");
    assert_eq!(info.task_id, "042-deep-feature");
    assert_eq!(info.title, "Task 042: Deep Feature");
    assert_eq!(info.status, TaskStatus::InProgress);
    assert_eq!(info.intent.as_deref(), Some("🚀 NEW FEATURE"));
    assert!(info.purpose.contains("Build deep feature module"));
    assert_eq!(info.acceptance_criteria.len(), 3);
    assert_eq!(info.unresolved_criteria.len(), 2);
    assert_eq!(info.must_not.len(), 2);
    assert!(info.must_not[0].contains("Must NOT introduce background daemons"));
}

#[test]
fn test_aristotle_context_validation_valid() {
    let valid_context = r#"# Domain Glossary

This document defines terms using Aristotle's formula.

## Core Concepts

**Task**:
An executable unit of engineering work, that has bounded acceptance criteria and verifiable completion evidence.
_Avoid_: Ticket, issue, story, workitem.

**Layer**:
A verification step, that executes a specific analysis or testing command within a bounded timeout.
_Avoid_: Stage, phase, check-item.
"#;

    let diagnostics = validate_context_content(valid_context);
    assert!(
        diagnostics.is_empty(),
        "Valid Aristotle glossary must yield 0 diagnostics, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_aristotle_context_validation_violations() {
    // Missing _Avoid_:
    let missing_avoid = r#"# Domain Glossary
**Task**:
An executable unit of engineering work, that has bounded acceptance criteria.
"#;

    let diags = validate_context_content(missing_avoid);
    assert!(!diags.is_empty());
    assert_eq!(diags[0].tool_name, "ARISTOTLE_FORMAT_VIOLATION");
    assert!(diags[0].message.contains("missing an '_Avoid_:' line"));

    // Empty content / no terms:
    let empty_context = "# Empty Glossary\n\nNo terms defined here.";
    let diags_empty = validate_context_content(empty_context);
    assert!(!diags_empty.is_empty());
    assert_eq!(diags_empty[0].tool_name, "ARISTOTLE_FORMAT_VIOLATION");
    assert!(diags_empty[0]
        .message
        .contains("No valid Aristotle glossary terms"));
}

#[test]
fn test_workspace_real_context_and_tasks_pass_validation() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // 1. CONTEXT.md in real workspace root must pass Aristotle check
    let context_diags = validate_context_glossary(workspace);
    assert!(
        context_diags.is_empty(),
        "Workspace CONTEXT.md must pass Aristotle validation, errors: {:?}",
        context_diags
    );

    // 2. Existing task files in tasks/ must validate
    let task_001 = workspace.join("tasks/001-bootstrap.md");
    if task_001.exists() {
        let rep_001 = check_task_specification(&task_001, workspace);
        assert!(
            rep_001.is_valid,
            "Task 001 must be valid: {:?}",
            rep_001.diagnostics
        );
        assert!(!rep_001.must_not_rules.is_empty());
        assert!(!rep_001.acceptance_criteria.is_empty());
    }

    let task_002 = workspace.join("tasks/002-wasmtime-policy-engine.md");
    if task_002.exists() {
        let rep_002 = check_task_specification(&task_002, workspace);
        assert!(
            rep_002.is_valid,
            "Task 002 must be valid: {:?}",
            rep_002.diagnostics
        );
    }

    let task_003 = workspace.join("tasks/003-task-and-spec-engine.md");
    if task_003.exists() {
        let rep_003 = check_task_specification(&task_003, workspace);
        assert!(
            rep_003.is_valid,
            "Task 003 must be valid: {:?}",
            rep_003.diagnostics
        );
    }

    // 3. Active task resolution consistency invariant
    let active_id = resolve_active_task_id(workspace);
    assert_eq!(has_active_task(workspace), active_id.is_some());

    // 4. Contract resolution
    let contract = resolve_task_contract(workspace, Some("003")).expect("contract must resolve");
    assert!(contract.task_id.contains("003"));
    assert!(!contract.acceptance_criteria.is_empty());
}

#[test]
fn test_check_task_specification_catches_all_spec_defects() {
    let temp_dir = std::env::temp_dir().join(format!(
        "xgauntlet_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    // Create a valid CONTEXT.md
    fs::write(
        temp_dir.join("CONTEXT.md"),
        "**Feature**:\nA functional capability that provides value.\n_Avoid_: Thing, stuff.\n",
    )
    .unwrap();

    // 1. Missing file
    let missing_path = temp_dir.join("non_existent.md");
    let rep = check_task_specification(&missing_path, &temp_dir);
    assert!(!rep.is_valid);
    assert_eq!(rep.diagnostics[0].tool_name, "TASK_FILE_NOT_FOUND");

    // 2. Missing OKF frontmatter
    let no_okf = temp_dir.join("no_okf.md");
    fs::write(&no_okf, "# Task Without OKF\n## 🎯 Formål\nTest\n## 📋 Acceptance Criteria\n- [ ] Crit\n## 🚫 Must NOT\n- Rule\n").unwrap();
    let rep = check_task_specification(&no_okf, &temp_dir);
    assert!(!rep.is_valid);
    assert!(rep
        .diagnostics
        .iter()
        .any(|d| d.tool_name == "INVALID_OKF_METADATA"));

    // 3. Missing Purpose
    let no_purpose = temp_dir.join("no_purpose.md");
    fs::write(&no_purpose, "---\ntype: Task Package\n---\n# Task\n## 📋 Acceptance Criteria\n- [ ] Crit\n## 🚫 Must NOT\n- Rule\n").unwrap();
    let rep = check_task_specification(&no_purpose, &temp_dir);
    assert!(!rep.is_valid);
    assert!(rep
        .diagnostics
        .iter()
        .any(|d| d.tool_name == "MISSING_PURPOSE"));

    // 4. Missing Acceptance Criteria
    let no_criteria = temp_dir.join("no_criteria.md");
    fs::write(
        &no_criteria,
        "---\ntype: Task Package\n---\n# Task\n## 🎯 Formål\nTest\n## 🚫 Must NOT\n- Rule\n",
    )
    .unwrap();
    let rep = check_task_specification(&no_criteria, &temp_dir);
    assert!(!rep.is_valid);
    assert!(rep
        .diagnostics
        .iter()
        .any(|d| d.tool_name == "MISSING_ACCEPTANCE_CRITERIA"));

    // 5. Missing Must NOT
    let no_must_not = temp_dir.join("no_must_not.md");
    fs::write(&no_must_not, "---\ntype: Task Package\n---\n# Task\n## 🎯 Formål\nTest\n## 📋 Acceptance Criteria\n- [ ] Crit\n").unwrap();
    let rep = check_task_specification(&no_must_not, &temp_dir);
    assert!(!rep.is_valid);
    assert!(rep
        .diagnostics
        .iter()
        .any(|d| d.tool_name == "MISSING_MUST_NOT"));

    let _ = fs::remove_dir_all(temp_dir);
}
