---
type: Task Package
title: "Task 079: Recursive Task Subdirectories & Auto-Archiving in tasks/"
description: "Eliminér den lange flade task-liste ved at understøtte rekursiv scanning af undermapper i tasks/** med numerisk sortering (inkl. >999), valgfri --subdir placering og arkivering af DONE-tasks til tasks/archive/."
status: active
intent: enhancement
slice: tasks
spec_ref: "specs/tasks.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T15:16:00Z" }
tags: [tasks, dx, subdirectories, archiving, lifecycle]
scope:
  - "specs/tasks.md"
  - "tasks/079-recursive-task-subdirectories-and-archiving.md"
  - "crates/xgauntlet-core/src/lib.rs"
  - "crates/xgauntlet-core/src/features/tasks/mod.rs"
  - "crates/xgauntlet-core/src/features/tasks/parser.rs"
  - "crates/xgauntlet-core/src/features/tasks/lifecycle.rs"
  - "crates/xgauntlet-core/src/features/tasks/telemetry.rs"
  - "crates/xgauntlet-core/src/features/tasks/validator.rs"
  - "crates/xgauntlet-core/src/features/checkpoint/engine.rs"
  - "crates/xgauntlet-core/src/features/checkpoint/preflight.rs"
  - "crates/xgauntlet-core/src/features/policy/models.rs"
  - "crates/xgauntlet-core/src/features/doctor/checks/governance.rs"
  - "crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/checkers.rs"
  - "crates/xgauntlet-cli/src/render.rs"
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
  - "crates/xgauntlet-core/tests/harness_adapters_test.rs"
test_refs:
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
---

# Task 079: Recursive Task Subdirectories & Auto-Archiving in tasks/

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Løse udviklerens UX-problem med at skulle scrolle gennem 75–130+ historiske task-filer i en flad `tasks/`-mappe — uden at tvinge tasks ind i kunstige feature-/slice-mapper — ved at løfte `tasks/`-motoren fra 1-niveau `fs::read_dir("tasks")` til fuld rekursiv mappe-understøttelse og rent arkiveringsflow:

1. **Rekursiv Task Discovery (`tasks/**`) med Numerisk Sortering (`001`..`999` → `1000+`):**
   - Alle task-opslag på tværs af `parser.rs`, `lifecycle.rs`, `telemetry.rs`, `validator.rs`, `checkpoint/`, `policy/models.rs`, `doctor/`, og `xgauntlet-cli` bruger én fælles rekursiv discovery-motor under `tasks/**` (ekskl. `README.md` og `TEMPLATE.md`).
   - Sortering sker deterministisk på det ledende numeriske task-præfiks (`Option<u32>`), så `1000 > 999`, og eksakt/præfiks-match (`stem == id` / `stem.starts_with("{id}-")`) prioriteres over substring-match (`contains`), så `-t 001` aldrig kolliderer med `1001-...md`.
   - En task kan dermed ligge i roden (`tasks/079-...md`), i en valgfri undermappe (`tasks/infra/071-...md`) eller i arkivmappen (`tasks/archive/001-...md`), mens `--task 071` fortsat finder den øjeblikkeligt.
2. **Ren Aktiv Rod, Valgfri `--subdir` & Arkivering til `tasks/archive/`:**
   - `xgauntlet task new` opretter som standard aktive tasks direkte i `tasks/<id>-<slug>.md` (med valgfrit `--subdir <mappe>`), mens `slice` forbliver afkoblet fra mappeplacering.
   - `xgauntlet task archive [<task> | --all-done | --dry-run]` flytter færdige (`DONE`/`COMPLETED`) tasks til `tasks/archive/<id>-<slug>.md` og opdaterer automatisk relative task-links i `specs/**/*.md` (f.eks. `../tasks/<id>.md` $\to$ `../tasks/archive/<id>.md`), så to-vejs sporbarhed bevares.
   - `AntigravityAdapter::render_blockquote_hud` linker direkte til `telemetry.file_path` (f.eks. `[Task](tasks/079-...md)` eller `[Task](tasks/archive/...)`).

## 📋 Acceptance Criteria
- [ ] **AC1 (Rekursiv Scanning, Numerisk Sortering >999 & Globalt ID-opslag):**
  - `collect_task_files`, `resolve_task_file_path`, `resolve_task_path`, `list_workspace_tasks`, `has_active_task`, `resolve_active_task_id`, `resolve_latest_done_task_id`, `resolve_task_contract`, `check_all_tasks`, `EnforcementContext::from_workspace`, `load_task_package_info` og `run_check_spec` finder `.md`-opgavefiler i vilkårlige undermapper under `tasks/` (bortset fra `README.md`/`TEMPLATE.md`).
  - Sortering sker numerisk på det ledende task-nummer (så `1000 > 999`), og eksakt/præfiks-opslag (`001`) prioriteres over substring-match (`1001`).
  - Næste ledige task-nummer (`detect_next_task_number`) beregnes som `max(id) + 1` på tværs af samtlige undermapper i `tasks/**`, så ID-kollisioner aldrig kan opstå.
- [ ] **AC2 (Arkiveringsflow & Valgfri Undermapper):**
  - `xgauntlet task archive` flytter færdige (`DONE`/`COMPLETED`) tasks til `tasks/archive/` (med `--dry-run`, enkelt-ID eller `--all-done`), afviser at arkivere ikke-færdige tasks, og opdaterer relative links i `specs/**/*.md`.
  - `xgauntlet task new --subdir <dir>` kan oprette tasks i en sikker relativ undermappe under `tasks/` (afviser path traversal).
  - Cockpit HUD `[Task](...)` linker direkte til `telemetry.file_path`.
- [ ] **AC3 (Isolerede Integrationstests):**
  - Udvid `crates/xgauntlet-core/tests/task_lifecycle_test.rs` med tests for rekursiv task-opslag, globalt ID-inkrement på tværs af undermapper, `>999` numerisk sortering & `001` vs `1001` match-prioritet, `EnforcementContext` i undermapper, `--subdir` scaffolding og `archive_tasks` inkl. opdatering af `specs/*.md`-links.

## 🚫 Must NOT
- Må IKKE bryde eksisterende projekter, der bruger en flad `tasks/`-mappe (100% bagudkompatibel).
- Må IKKE genbruge et eksisterende task-ID, selvom den tidligere task ligger i `tasks/archive/` eller en undermappe.
- Må IKKE arkivere aktive/draft/ikke-`DONE` tasks.
- Må IKKE tvinge tasks ind i `tasks/<slice>/`-undermapper eller koble `tasks/`-mappestrukturen til `slice`.
- Må IKKE kræve ændringer i `gauntlet_policy.wasm` (da `target_str.starts_with("tasks/")` allerede dækker alle undermapper under `tasks/`).

## 📝 Revisions
- 2026-11-14: Oprettet opgavepakke for at eliminere den lange flade scroll-liste i `tasks/`.
- 2026-11-14: Afklaret design efter arkitektonisk sparring: afkoblet fra `slice`-begrebet, tilføjet numerisk `>999` sortering, `xgauntlet task archive` og komplet scope (`policy/models.rs`, `checkers.rs`, `doctor`, `antigravity`). Sat til `ACTIVE`.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 079`
- `cargo test -p xgauntlet-core --test task_lifecycle_test`
- `cargo run -p xgauntlet-cli -- verify --task 079`
