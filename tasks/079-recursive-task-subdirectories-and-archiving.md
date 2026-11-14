---
type: Task Package
title: "Task 079: Recursive Feature Subdirectories & Auto-Archiving in tasks/"
description: "Eliminér den lange flade task-liste ved at understøtte rekursiv scanning af undermapper i tasks/ (f.eks. tasks/<slice>/ og tasks/archive/) samt valgfri automatisk arkivering af afsluttede DONE-tasks."
status: draft
intent: enhancement
slice: tasks
spec_ref: "specs/tasks.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T15:16:00Z" }
tags: [tasks, dx, subdirectories, archiving, slice, lifecycle]
scope:
  - "specs/tasks.md"
  - "tasks/079-recursive-task-subdirectories-and-archiving.md"
  - "crates/xgauntlet-core/src/features/tasks/parser.rs"
  - "crates/xgauntlet-core/src/features/tasks/lifecycle.rs"
  - "crates/xgauntlet-core/src/features/tasks/telemetry.rs"
  - "crates/xgauntlet-core/src/features/tasks/validator.rs"
  - "crates/xgauntlet-core/src/features/checkpoint/engine.rs"
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
test_refs:
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
---

# Task 079: Recursive Feature Subdirectories & Auto-Archiving in tasks/

**Status**: `DRAFT`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Løse udviklerens UX-problem med at skulle scrolle gennem 75–130+ historiske task-filer i en flad `tasks/`-mappe ved at løfte `tasks/`-motoren fra 1-niveau `fs::read_dir("tasks")` til fuld rekursiv mappe-understøttelse:

1. **Rekursiv Task Discovery (`tasks/**`):**
   - Alle task-opslag i `parser.rs`, `lifecycle.rs`, `telemetry.rs` og `validator.rs` (`has_active_task`, `resolve_active_task_id`, `resolve_latest_done_task_id`, `resolve_task_path`, `list_workspace_tasks`, `check_all_tasks` og næste ledige task-ID) scanner undermapper under `tasks/` rekursivt og sorterer deterministisk på det numeriske task-præfiks (`001`..`999`).
   - En task kan dermed ligge i roden (`tasks/079-...md`), i en feature-/slice-undermappe (`tasks/topology/071-...md`) eller i en arkivmappe (`tasks/archive/001-...md` eller `tasks/archive/topology/071-...md`), mens `--task 071` fortsat finder den øjeblikkeligt.
2. **Feature-kategorisering & Arkivering ved `DONE`:**
   - `xgauntlet task new <slug> --slice <slice>` kan placere eller arkivere tasks organiseret efter slice/kategori.
   - Mulighed for at holde roden af `tasks/` ren for færdige tasks (f.eks. ved at `DONE`-tasks flyttes til `tasks/archive/` eller `tasks/<slice>/`), så udvikleren i sin IDE kun ser de aktuelle/aktive tasks øverst.

## 📋 Acceptance Criteria
- [ ] **AC1 (Rekursiv Scanning & Globalt ID-opslag):**
  - `resolve_task_path`, `list_workspace_tasks`, `has_active_task`, `resolve_active_task_id` og `check_all_tasks` finder `.md`-opgavefiler i vilkårlige undermapper under `tasks/` (bortset fra evt. `README.md`/`TEMPLATE.md`).
  - Næste ledige task-nummer (`next_task_number`) beregnes som `max(id) + 1` på tværs af samtlige undermapper i `tasks/**`, så ID-kollisioner aldrig kan opstå.
- [ ] **AC2 (Kategorisering & Arkiveringsflow):**
  - Tasks kan organiseres i undermapper under `tasks/` (f.eks. `tasks/<slice>/` eller `tasks/archive/`) uden at bryde `xgauntlet check-spec`, `xgauntlet verify`, `xgauntlet checkpoint` eller Cockpit HUD-links.
- [ ] **AC3 (Isolerede Integrationstests):**
  - Udvid `crates/xgauntlet-core/tests/task_lifecycle_test.rs` med tests for rekursiv task-opslag, globalt ID-inkrement på tværs af undermapper og aktiv/done-opløsning i undermapper.

## 🚫 Must NOT
- Må IKKE bryde eksisterende projekter, der bruger en flad `tasks/`-mappe (100% bagudkompatibel).
- Må IKKE genbruge et eksisterende task-ID, selvom den tidligere task ligger i `tasks/archive/` eller `tasks/<slice>/`.
- Må IKKE kræve ændringer i `gauntlet_policy.wasm` (da `target_str.starts_with("tasks/")` allerede dækker alle undermapper under `tasks/`).

## 📝 Revisions
- 2026-11-14: Oprettet opgavepakke for at eliminere den lange flade scroll-liste i `tasks/`.

## 🧪 Verifikation
- `target/debug/xgauntlet check-spec -t 079`
- `cargo test -p xgauntlet-core --test task_lifecycle_test`
- `target/debug/xgauntlet verify --task 079`

