//! Integration tests for Task Lifecycle Engine, Intent Scaffolding,
//! and Task Telemetry (Task 013).

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::tasks::{
    check_task_specification, collect_git_telemetry, inspect_task_telemetry, list_workspace_tasks,
    parse_criteria_progress, ScaffoldTaskOptions, TaskScaffolder, TaskStatus,
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
        let path = std::env::temp_dir().join(format!("xgauntlet_test_{}", unique));
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

fn setup_mock_glossary(workspace: &Path) {
    let glossary = r#"---
type: Knowledge Bundle Index
title: "xGauntlet Context & Domain Glossary"
description: "Test glossary"
status: stable
---

# xGauntlet Context & Domain Glossary

**Task**:
An executable unit of engineering work.
_Avoid_: Ticket, issue.

**Gauntlet**:
A sequential verification pipeline.
_Avoid_: Test runner.
"#;
    fs::write(workspace.join("CONTEXT.md"), glossary).unwrap();
}

#[test]
fn test_detect_next_task_number() {
    let temp = TempDir::new("task_num_detect");
    let tasks_dir = temp.path().join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();

    // 1. Empty tasks dir -> should detect 1
    assert_eq!(TaskScaffolder::detect_next_task_number(&tasks_dir), 1);

    // 2. Add some task files
    fs::write(tasks_dir.join("001-core-policy.md"), "# Task 1").unwrap();
    fs::write(tasks_dir.join("002-manifest.md"), "# Task 2").unwrap();
    fs::write(tasks_dir.join("README.md"), "# Readme").unwrap(); // ignore non-numeric

    assert_eq!(TaskScaffolder::detect_next_task_number(&tasks_dir), 3);

    // 3. Add higher task number
    fs::write(tasks_dir.join("014-dx-enhancements.md"), "# Task 14").unwrap();
    assert_eq!(TaskScaffolder::detect_next_task_number(&tasks_dir), 15);
}

#[test]
fn test_slugify_and_formatting() {
    assert_eq!(TaskScaffolder::format_task_number(1), "001");
    assert_eq!(TaskScaffolder::format_task_number(14), "014");
    assert_eq!(TaskScaffolder::format_task_number(125), "125");

    assert_eq!(
        TaskScaffolder::slugify("Dynamic HUD & Telemetry!"),
        "dynamic-hud-telemetry"
    );
    assert_eq!(
        TaskScaffolder::slugify("task_lifecycle_engine"),
        "task-lifecycle-engine"
    );

    assert_eq!(
        TaskScaffolder::format_intent(Some("feature")),
        "🚀 NEW FEATURE"
    );
    assert_eq!(TaskScaffolder::format_intent(Some("bug")), "🐛 BUG FIX");
    assert_eq!(
        TaskScaffolder::format_intent(Some("refactor")),
        "🔄 REFACTOR"
    );
    assert_eq!(TaskScaffolder::format_intent(None), "🚀 NEW FEATURE");
}

#[test]
fn test_scaffold_task_package_creates_valid_spec() {
    let temp = TempDir::new("task_scaffold_valid");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let options = ScaffoldTaskOptions {
        name: "intent-telemetry-engine".to_string(),
        title: Some("Intent Telemetry Engine".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Etablere telemetri- og intent-scaffolding motor jf. spec.md".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };

    let result = TaskScaffolder::scaffold(&options).expect("scaffold must succeed");
    assert_eq!(result.task_number, 1);
    assert_eq!(result.task_id, "001-intent-telemetry-engine");
    assert!(result.created);

    let created_file = ws.join("tasks/001-intent-telemetry-engine.md");
    assert!(created_file.is_file(), "Task file must exist on disk");

    let content = fs::read_to_string(&created_file).unwrap();
    assert!(content.contains("type: Task Package"));
    assert!(content.contains("Task 001: Intent Telemetry Engine"));
    assert!(content.contains("**Intent**: `🚀 NEW FEATURE`"));
    assert!(content.contains("## 🎯 Formål"));
    assert!(content.contains("## 📋 Acceptance Criteria"));
    assert!(content.contains("## 🚫 Must NOT"));
    assert!(content.contains("## 📝 Revisions"));
    assert!(content.contains("## 🧪 Verifikation"));

    // Check that check-spec validates this scaffolded task with 0 errors!
    let report = check_task_specification(&created_file, ws);
    assert!(
        report.is_valid,
        "Scaffolded task must pass check_task_specification: {:?}",
        report.diagnostics
    );
}

#[test]
fn test_scaffold_collision_fail_closed_without_force() {
    let temp = TempDir::new("task_scaffold_collision");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let options = ScaffoldTaskOptions {
        name: "duplicate-task".to_string(),
        title: None,
        intent: None,
        purpose: None,
        workspace: ws.to_path_buf(),
        force: false,
    };

    // First scaffold succeeds
    TaskScaffolder::scaffold(&options).expect("first scaffold must succeed");

    // Second scaffold for existing file with force: false must fail closed
    let err = TaskScaffolder::scaffold(&options);
    assert!(
        err.is_err(),
        "Must fail closed when target task already exists and force is false"
    );

    // With force: true, it overwrites
    let force_options = ScaffoldTaskOptions {
        force: true,
        ..options
    };
    let res = TaskScaffolder::scaffold(&force_options);
    assert!(res.is_ok(), "Must succeed with force: true");
}

#[test]
fn test_parse_criteria_progress() {
    let mixed_content = r#"
# Task 099: Test
## 📋 Acceptance Criteria
- [x] First completed criteria
- [X] Second completed criteria
- [ ] Third pending criteria
- [x] Fourth completed criteria
- [ ] Fifth pending criteria
"#;

    let progress = parse_criteria_progress(mixed_content);
    assert_eq!(progress.total, 5);
    assert_eq!(progress.completed, 3);
    assert_eq!(progress.pending, 2);
    assert_eq!(progress.percentage, 60);
    assert_eq!(progress.bar, "[■■■□□]");

    let empty_content = "# Task without criteria\n";
    let empty_progress = parse_criteria_progress(empty_content);
    assert_eq!(empty_progress.total, 0);
    assert_eq!(empty_progress.completed, 0);
    assert_eq!(empty_progress.pending, 0);
    assert_eq!(empty_progress.percentage, 0);
    assert_eq!(empty_progress.bar, "[□□□□□]");

    let full_content = r#"
- [x] A
- [x] B
- [x] C
- [x] D
"#;
    let full_progress = parse_criteria_progress(full_content);
    assert_eq!(full_progress.total, 4);
    assert_eq!(full_progress.completed, 4);
    assert_eq!(full_progress.pending, 0);
    assert_eq!(full_progress.percentage, 100);
    assert_eq!(full_progress.bar, "[■■■■■]");
}

#[test]
fn test_git_telemetry_graceful_fallback() {
    let temp = TempDir::new("non_git_ws");
    let ws = temp.path();

    // Ensure directory is definitely not a git repository
    let telemetry = collect_git_telemetry(ws);
    assert_eq!(telemetry.branch, "unknown");
    assert_eq!(telemetry.head_oid, "unknown");
    assert_eq!(telemetry.dirty_count, 0);
    assert!(telemetry.is_clean);
}

#[test]
fn test_inspect_and_list_workspace_tasks() {
    let temp = TempDir::new("task_inspect_list");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let opt1 = ScaffoldTaskOptions {
        name: "alpha-feature".to_string(),
        title: Some("Alpha Feature".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Alpha purpose".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    let opt2 = ScaffoldTaskOptions {
        name: "beta-fix".to_string(),
        title: Some("Beta Fix".to_string()),
        intent: Some("bug".to_string()),
        purpose: Some("Beta purpose".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };

    TaskScaffolder::scaffold(&opt1).unwrap();
    TaskScaffolder::scaffold(&opt2).unwrap();

    let tasks = list_workspace_tasks(ws).expect("list tasks must succeed");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].task_id, "001-alpha-feature");
    assert_eq!(tasks[0].task_number, Some(1));
    assert_eq!(tasks[0].title, "Task 001: Alpha Feature");
    assert_eq!(tasks[0].status, TaskStatus::Active);

    assert_eq!(tasks[1].task_id, "002-beta-fix");
    assert_eq!(tasks[1].task_number, Some(2));
    assert_eq!(tasks[1].status, TaskStatus::Active);

    // Test inspect_task_telemetry
    let telemetry = inspect_task_telemetry(ws, Some("001")).expect("inspect must succeed");
    assert_eq!(telemetry.task_id, "001-alpha-feature");
    assert_eq!(telemetry.status, TaskStatus::Active);
}

#[test]
fn test_render_box_card_format_and_borders() {
    let temp = TempDir::new("box_card_test");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let opt = ScaffoldTaskOptions {
        name: "box-card-feature".to_string(),
        title: Some("Box Card Feature".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Box card purpose".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    TaskScaffolder::scaffold(&opt).unwrap();

    let telemetry = inspect_task_telemetry(ws, Some("001"))
        .expect("inspect telemetry must succeed")
        .with_scope("crates/xgauntlet-core")
        .with_invariants("14/14 PASS")
        .with_evidence("pending")
        .with_phase("RED");

    let card = telemetry.render_box_card();
    let lines: Vec<&str> = card.lines().collect();

    assert_eq!(lines.len(), 6, "Box card must be exactly 6 lines");

    // All lines must be exactly 64 characters wide
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(
            line.chars().count(),
            64,
            "Line {} length should be 64, but got {}: '{}'",
            idx + 1,
            line.chars().count(),
            line
        );
    }

    // Header border
    assert!(lines[0].starts_with("┌─── xgauntlet: "));
    assert!(lines[0].ends_with("┐"));

    // Row 1: Status and Scope
    assert!(lines[1].starts_with("│ "));
    assert!(lines[1].ends_with(" │"));
    assert!(lines[1].contains("Status: RED"));
    assert!(lines[1].contains("Scope: crates/xgauntlet-core"));

    // Row 2: Progress and Invariants
    assert!(lines[2].starts_with("│ "));
    assert!(lines[2].ends_with(" │"));
    assert!(lines[2].contains("Progress:"));
    assert!(lines[2].contains("Invariants: 14/14 PASS"));

    // Row 3: Git and Evidence
    assert!(lines[3].starts_with("│ "));
    assert!(lines[3].ends_with(" │"));
    assert!(lines[3].contains("Git:"));
    assert!(lines[3].contains("Evidence: pending"));

    // Row 4: Ref row with clean paths (no markdown []())
    assert!(lines[4].starts_with("│ "));
    assert!(lines[4].ends_with(" │"));
    assert!(lines[4].contains("Ref: tasks/001.md"));
    assert!(lines[4].contains("spec.md"));
    assert!(lines[4].contains("docs/adr/README.md"));
    assert!(
        !lines[4].contains("]("),
        "Ref line must not contain markdown links"
    );

    // Bottom border
    assert!(lines[5].starts_with("└"));
    assert!(lines[5].ends_with("┘"));
}

#[test]
fn test_render_box_card_progress_states() {
    let temp = TempDir::new("box_card_states");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let opt = ScaffoldTaskOptions {
        name: "states-task".to_string(),
        title: Some("States Task".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Testing states".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    TaskScaffolder::scaffold(&opt).unwrap();

    let task_file = ws.join("tasks/001-states-task.md");

    // 1. Partial progress (60%)
    let mixed = r#"---
type: Task Package
title: Task 001: States Task
status: active
---
# Task 001: States Task
## 📋 Acceptance Criteria
- [x] A
- [x] B
- [x] C
- [ ] D
- [ ] E
"#;
    fs::write(&task_file, mixed).unwrap();

    let telemetry = inspect_task_telemetry(ws, Some("001")).unwrap();
    let card = telemetry.render_box_card();
    assert!(card.contains("[██████░░░░] 60%"));

    // 2. Full progress (100%)
    let full = r#"---
type: Task Package
title: Task 001: States Task
status: done
---
# Task 001: States Task
## 📋 Acceptance Criteria
- [x] A
- [x] B
"#;
    fs::write(&task_file, full).unwrap();

    let telemetry_full = inspect_task_telemetry(ws, Some("001")).unwrap();
    let card_full = telemetry_full.render_box_card();
    assert!(card_full.contains("[██████████] 100%"));
}

#[test]
fn test_render_box_compact() {
    let temp = TempDir::new("box_compact");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let opt = ScaffoldTaskOptions {
        name: "compact-task".to_string(),
        title: Some("Compact Task".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Compact test".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    TaskScaffolder::scaffold(&opt).unwrap();

    let telemetry = inspect_task_telemetry(ws, Some("001")).unwrap();
    let compact = telemetry.render_box_compact();

    assert!(
        !compact.contains('\n'),
        "Compact badge must be a single line"
    );
    assert!(compact.contains("xgauntlet"));
    assert!(compact.contains("001-compact-task"));
}

#[test]
fn test_render_box_card_long_strings_ellipsing_and_fixed_64_width() {
    let temp = TempDir::new("box_long_strings");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let opt = ScaffoldTaskOptions {
        name: "long-strings-feature".to_string(),
        title: Some("Long Strings Feature".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Testing long strings ellipsing".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    TaskScaffolder::scaffold(&opt).unwrap();

    let mut telemetry = inspect_task_telemetry(ws, Some("001"))
        .expect("inspect telemetry must succeed")
        .with_scope(
            "crates/xgauntlet-core,crates/xgauntlet-cli,crates/xgauntlet-policy,crates/extra",
        )
        .with_invariants("14/14 PASS (all verification layers green and verified)")
        .with_evidence("evidence-manifest-verification-digest-signature-sha256")
        .with_phase("RED (Tests failing with comprehensive assertions)");

    telemetry.task_id = "019-extremely-long-task-identifier-with-many-words".to_string();
    telemetry.git.branch =
        "feature/super-long-branch-name-that-definitely-exceeds-standard-limits".to_string();
    telemetry.git.dirty_count = 142;
    telemetry.git.is_clean = false;

    let card = telemetry.render_box_card();
    let lines: Vec<&str> = card.lines().collect();

    assert_eq!(lines.len(), 6, "Box card must always be exactly 6 lines");

    // Every single line MUST be exactly 64 characters wide even with long strings!
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(
            line.chars().count(),
            64,
            "Line {} length should be exactly 64, but got {}: '{}'",
            idx + 1,
            line.chars().count(),
            line
        );
        if idx == 0 {
            assert!(line.starts_with("┌─── xgauntlet: "));
            assert!(line.ends_with("┐"));
        } else if idx == 5 {
            assert!(line.starts_with("└"));
            assert!(line.ends_with("┘"));
        } else {
            assert!(
                line.starts_with("│ "),
                "Line {} must start with border '│ '",
                idx + 1
            );
            assert!(
                line.ends_with(" │"),
                "Line {} must end with border ' │'",
                idx + 1
            );
        }
    }
}

#[test]
fn test_render_box_card_windows_path_normalization() {
    let temp = TempDir::new("box_windows_path");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let opt = ScaffoldTaskOptions {
        name: "win-path-feature".to_string(),
        title: Some("Win Path Feature".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Testing windows path".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    TaskScaffolder::scaffold(&opt).unwrap();

    let mut telemetry =
        inspect_task_telemetry(ws, Some("001")).expect("inspect telemetry must succeed");

    // Simulate Windows backslash path
    telemetry.file_path = "tasks\\001-win-path-feature.md".to_string();

    let card = telemetry.render_box_card();
    let lines: Vec<&str> = card.lines().collect();

    assert_eq!(lines.len(), 6);
    let ref_line = lines[4];
    assert!(ref_line.contains("Ref: tasks/001.md"));
    assert!(
        !ref_line.contains('\\'),
        "Ref line must not contain Windows backslashes: '{}'",
        ref_line
    );
}

#[test]
fn test_recursive_task_discovery_and_global_id_increment() {
    let temp = TempDir::new("task_recursive_discovery");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let tasks_dir = ws.join("tasks");
    let archive_dir = tasks_dir.join("archive");
    let infra_dir = tasks_dir.join("infra");
    fs::create_dir_all(&archive_dir).unwrap();
    fs::create_dir_all(&infra_dir).unwrap();

    // Non-task markdown files that must be ignored
    fs::write(tasks_dir.join("README.md"), "# Tasks Overview").unwrap();
    fs::write(archive_dir.join("TEMPLATE.md"), "# Template").unwrap();

    let done_task = r#"---
type: Task Package
title: "Task 042: Archived Done Task"
description: "Done task in archive"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-11-14T12:00:00Z" }
---

# Task 042: Archived Done Task

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Archived task purpose.

## 📋 Acceptance Criteria
- [x] Done item

## 🚫 Must NOT
- Keep backward compatibility.
"#;
    fs::write(archive_dir.join("042-archived-done.md"), done_task).unwrap();

    let active_task = r#"---
type: Task Package
title: "Task 043: Subdir Active Task"
description: "Active task in nested subdir"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-11-14T12:05:00Z" }
scope:
  - "crates/xgauntlet-core/src/features/tasks/**"
test_refs:
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
---

# Task 043: Subdir Active Task

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Active task in infra subdir.

## 📋 Acceptance Criteria
- [ ] Recursive discovery works

## 🚫 Must NOT
- Reuse IDs.
"#;
    fs::write(infra_dir.join("043-subdir-active.md"), active_task).unwrap();

    // 1. Global next ID across all subdirectories
    assert_eq!(TaskScaffolder::detect_next_task_number(&tasks_dir), 44);

    // 2. Recursive listing ignores README.md / TEMPLATE.md
    let listed = list_workspace_tasks(ws).expect("list_workspace_tasks must succeed");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].task_id, "042-archived-done");
    assert_eq!(listed[0].file_path, "tasks/archive/042-archived-done.md");
    assert_eq!(listed[1].task_id, "043-subdir-active");
    assert_eq!(listed[1].file_path, "tasks/infra/043-subdir-active.md");

    // 3. Active/done resolution & contract resolution
    assert!(xgauntlet_core::has_active_task(ws));
    assert_eq!(
        xgauntlet_core::resolve_active_task_id(ws).as_deref(),
        Some("043-subdir-active")
    );
    assert_eq!(
        xgauntlet_core::resolve_latest_done_task_id(ws).as_deref(),
        Some("042-archived-done")
    );

    let contract = xgauntlet_core::resolve_task_contract(ws, None)
        .expect("resolve_task_contract must find active task in subdir");
    assert_eq!(contract.task_id, "043-subdir-active");

    let archived_contract = xgauntlet_core::resolve_task_contract(ws, Some("042"))
        .expect("resolve_task_contract must find archived task by short ID");
    assert_eq!(archived_contract.task_id, "042-archived-done");

    // 4. check_all_tasks inspects both subdir tasks (and skips README/TEMPLATE)
    let reports = xgauntlet_core::check_all_tasks(ws);
    assert_eq!(reports.len(), 2);
    assert!(reports.iter().all(|r| r.is_valid));

    // 5. EnforcementContext loads scope from active task in subdir
    let ctx = xgauntlet_core::EnforcementContext::from_workspace(ws, false);
    assert!(ctx.has_active_task);
    assert_eq!(ctx.active_task_id, "043-subdir-active");
    assert_eq!(
        ctx.allowed_scopes,
        vec!["crates/xgauntlet-core/src/features/tasks/**".to_string()]
    );

    // 6. Telemetry & HUD link directly to task file path
    let telemetry = inspect_task_telemetry(ws, None).expect("inspect telemetry in subdir");
    assert_eq!(telemetry.file_path, "tasks/infra/043-subdir-active.md");
    let hud = xgauntlet_core::AntigravityAdapter::render_blockquote_hud(&telemetry, None);
    assert!(
        hud.contains("[Task](tasks/infra/043-subdir-active.md)"),
        "HUD must link directly to task file path: {hud}"
    );
}

#[test]
fn test_numeric_task_sorting_beyond_999_and_exact_prefix_priority() {
    let temp = TempDir::new("task_numeric_1000");
    let ws = temp.path();
    setup_mock_glossary(ws);

    let tasks_dir = ws.join("tasks");
    let archive_dir = tasks_dir.join("archive");
    fs::create_dir_all(&archive_dir).unwrap();

    let make_task = |num: &str, slug: &str, status: &str| {
        let upper = status.to_ascii_uppercase();
        format!(
            "---\ntype: Task Package\ntitle: \"Task {num}: {slug}\"\nstatus: {status}\ngenerated: {{ by: process:xgauntlet-task-init, at: \"2026-11-14T12:00:00Z\" }}\nscope:\n  - \"src/**\"\ntest_refs:\n  - \"tests/**\"\n---\n\n# Task {num}: {slug}\n\n**Status**: `{upper}`\n\n## 🎯 Formål\nPurpose {num}.\n\n## 📋 Acceptance Criteria\n- [ ] Crit {num}\n\n## 🚫 Must NOT\n- Break.\n"
        )
    };

    fs::write(
        archive_dir.join("001-initial.md"),
        make_task("001", "initial", "done"),
    )
    .unwrap();
    fs::write(
        archive_dir.join("999-before-thousand.md"),
        make_task("999", "before-thousand", "done"),
    )
    .unwrap();
    fs::write(
        archive_dir.join("1000-beyond-thousand.md"),
        make_task("1000", "beyond-thousand", "done"),
    )
    .unwrap();
    // 1001 contains "001" as substring - must NOT shadow 001!
    fs::write(
        tasks_dir.join("1001-active-newest.md"),
        make_task("1001", "active-newest", "active"),
    )
    .unwrap();
    fs::write(
        tasks_dir.join("998-older-active.md"),
        make_task("998", "older-active", "active"),
    )
    .unwrap();

    assert_eq!(TaskScaffolder::detect_next_task_number(&tasks_dir), 1002);
    assert_eq!(
        xgauntlet_core::resolve_latest_done_task_id(ws).as_deref(),
        Some("1000-beyond-thousand")
    );
    assert_eq!(
        xgauntlet_core::resolve_active_task_id(ws).as_deref(),
        Some("1001-active-newest")
    );

    // Exact prefix "001" must match 001-initial, NOT 1001-active-newest
    let contract_001 = xgauntlet_core::resolve_task_contract(ws, Some("001")).unwrap();
    assert_eq!(contract_001.task_id, "001-initial");

    let tel_001 = inspect_task_telemetry(ws, Some("001")).unwrap();
    assert_eq!(tel_001.task_id, "001-initial");
}

#[test]
fn test_scaffold_in_subdir_and_archive_done_tasks_updates_spec_links() {
    let temp = TempDir::new("task_archive_flow");
    let ws = temp.path();
    setup_mock_glossary(ws);

    // 1. Scaffold a task in a custom subdir with slice spec link
    let opt1 = ScaffoldTaskOptions {
        name: "billing:invoice-export".to_string(),
        title: Some("Invoice Export".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Export invoices".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    let res1 = TaskScaffolder::scaffold_in_subdir(&opt1, None, None, Some("billing-ops"))
        .expect("scaffold_in_subdir must succeed");
    assert_eq!(
        res1.path,
        ws.join("tasks/billing-ops/001-invoice-export.md")
    );
    assert!(res1.path.is_file());

    // Verify specs/billing.md points to the actual subdir path
    let spec_content = fs::read_to_string(ws.join("specs/billing.md")).unwrap();
    assert!(
        spec_content.contains("(../tasks/billing-ops/001-invoice-export.md)"),
        "Living spec must link to actual subdir path: {spec_content}"
    );

    // 2. Reject path traversal in subdir
    let bad = TaskScaffolder::scaffold_in_subdir(&opt1, None, None, Some("../escape"));
    assert!(bad.is_err(), "Path traversal in subdir must be rejected");

    // 3. Active task cannot be archived
    let active_arch = TaskScaffolder::archive_tasks(&xgauntlet_core::TaskArchiveOptions {
        workspace: ws.to_path_buf(),
        task_id: Some("001".to_string()),
        all_done: false,
        dry_run: false,
    });
    assert!(
        active_arch.is_err(),
        "Archiving an ACTIVE task must be rejected"
    );

    // 4. Mark task 001 DONE and scaffold active task 002
    let content_done = fs::read_to_string(&res1.path)
        .unwrap()
        .replace("status: active", "status: done")
        .replace("**Status**: `ACTIVE`", "**Status**: `DONE`");
    fs::write(&res1.path, content_done).unwrap();

    let opt2 = ScaffoldTaskOptions {
        name: "billing:active-followup".to_string(),
        title: Some("Active Followup".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Keep active".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    };
    let res2 = TaskScaffolder::scaffold(&opt2).unwrap();
    assert_eq!(res2.task_number, 2);

    // 5. Dry-run does not move files
    let dry = TaskScaffolder::archive_tasks(&xgauntlet_core::TaskArchiveOptions {
        workspace: ws.to_path_buf(),
        task_id: None,
        all_done: true,
        dry_run: true,
    })
    .unwrap();
    assert_eq!(dry.archived.len(), 1);
    assert!(res1.path.is_file(), "dry_run must not move file");

    // 6. Real archive moves DONE 001 to tasks/archive/, keeps active 002 in root, and updates specs/billing.md
    let archived = TaskScaffolder::archive_tasks(&xgauntlet_core::TaskArchiveOptions {
        workspace: ws.to_path_buf(),
        task_id: None,
        all_done: true,
        dry_run: false,
    })
    .unwrap();
    assert_eq!(archived.archived.len(), 1);
    assert!(!res1.path.exists());
    assert!(ws.join("tasks/archive/001-invoice-export.md").is_file());
    assert!(res2.path.is_file(), "Active task 002 must stay in root");

    let updated_spec = fs::read_to_string(ws.join("specs/billing.md")).unwrap();
    assert!(
        updated_spec.contains("(../tasks/archive/001-invoice-export.md)"),
        "Living spec link must be updated after archive: {updated_spec}"
    );
    assert!(
        updated_spec.contains("(../tasks/002-active-followup.md)"),
        "Active task link must remain intact: {updated_spec}"
    );
}

#[test]
fn test_task_081_feature_frontmatter_and_legacy_slice_compatibility() {
    use serde_json::json;
    use xgauntlet_core::features::tasks::{
        check_task_specification, parse_task_file, ScaffoldTaskOptions, TaskScaffolder,
    };

    let tmp = TempDir::new("t081_feature_compat");
    let ws = tmp.path();
    fs::write(
        ws.join("CONTEXT.md"),
        "---\ntype: Knowledge Bundle Index\nstatus: stable\n---\n# Context\n**Feature**:\nBoundary.\n_Avoid_: slice.\n",
    )
    .unwrap();
    fs::write(
        ws.join("gauntlet.toml"),
        "stack = \"rust\"\n[architecture]\nfeature_root = \"src/features\"\nisolate_features = true\n",
    )
    .unwrap();
    fs::create_dir_all(ws.join("tests")).unwrap();
    fs::write(ws.join("tests/billing_test.rs"), "#[test] fn ok() {}\n").unwrap();

    let scaffolded = TaskScaffolder::scaffold(&ScaffoldTaskOptions {
        name: "billing:export".to_string(),
        title: Some("Export".to_string()),
        intent: Some("feature".to_string()),
        purpose: Some("Invoice export".to_string()),
        workspace: ws.to_path_buf(),
        force: false,
    })
    .expect("scaffold billing:export");

    let task_md = fs::read_to_string(&scaffolded.path).unwrap();
    assert!(
        task_md.contains("feature: billing") && !task_md.contains("\nslice:"),
        "Scaffolded task must use 'feature:' as primary key, got:\n{task_md}"
    );

    let spec_md = fs::read_to_string(ws.join("specs/billing.md")).unwrap();
    assert!(
        spec_md.contains("# Living Feature Specification: `billing`")
            && spec_md.contains("**Feature Path**:")
            && !spec_md.contains("Living Slice Specification"),
        "Scaffolded living spec must use Feature terminology, got:\n{spec_md}"
    );

    let parsed_new = parse_task_file(&scaffolded.path).unwrap();
    let parsed_json = serde_json::to_value(&parsed_new).unwrap();
    assert_eq!(parsed_json["feature"], json!("billing"));

    // Backward compatibility: legacy 'slice:' YAML frontmatter in existing task still validates cleanly
    let legacy_path = ws.join("tasks/002-legacy.md");
    fs::write(
        &legacy_path,
        "---\ntype: Task Package\ntitle: \"Task 002: Legacy\"\ndescription: \"Legacy\"\nstatus: active\nintent: enhancement\nslice: billing\nspec_ref: \"specs/billing.md\"\ngenerated: { by: process:xgauntlet, at: \"2026-10-01T00:00:00Z\" }\ntags: [billing]\nscope: [\"src/features/billing/mod.rs\"]\ntest_refs: [\"tests/billing_test.rs\"]\n---\n\n# Task 002\n\n## 🎯 Formål\nLegacy compat.\n\n## 📋 Acceptance Criteria\n- [ ] AC1: Works.\n\n## 🚫 Must NOT\n- Never break.\n",
    )
    .unwrap();
    let legacy_parsed = parse_task_file(&legacy_path).unwrap();
    assert_eq!(
        serde_json::to_value(&legacy_parsed).unwrap()["feature"],
        json!("billing")
    );
    let rep = check_task_specification(&legacy_path, ws);
    assert!(rep.is_valid, "Legacy slice: task must remain valid: {:?}", rep.diagnostics);
}
