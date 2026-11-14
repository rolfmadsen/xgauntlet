---
type: Task Package
title: "Task 087: Archive Terminal Task Statuses (Deprecated, Superseded, Rejected)"
description: "Udvid TaskScaffolder::archive_tasks til også at arkivere terminale opgavetilstande (Deprecated, Superseded, Rejected) sammen med Done/Completed, så tasks/ udelukkende indeholder åbne DRAFT/ACTIVE/TODO-opgaver."
status: active
intent: enhancement
feature: tasks
spec_ref: "specs/tasks.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T21:32:00Z" }
tags: [tasks, archiving, lifecycle, deprecated, superseded, rejected, okf]
scope:
  - "crates/xgauntlet-core/src/features/tasks/models.rs"
  - "crates/xgauntlet-core/src/features/tasks/lifecycle.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/checkers.rs"
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
  - "specs/tasks.md"
  - "tasks/087-archive-terminal-task-statuses.md"
test_refs:
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
---

# Task 087: Archive Terminal Task Statuses (Deprecated, Superseded, Rejected)

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

---

## 🎯 Formål
I dag accepterer `TaskScaffolder::archive_tasks` kun `TaskStatus::Done | TaskStatus::Completed`, hvilket efterlader pensionerede opgaver med gyldige terminale statusser som `DEPRECATED` (f.eks. [Task 020](tasks/020-global-plugin-distribution-and-jit-skill-injection.md)), `SUPERSEDED` og `REJECTED` i `tasks/`-roden.

Formålet er at udvide `TaskStatus` og `TaskScaffolder::archive_tasks` til at behandle alle terminale tilstande (`Done`, `Completed`, `Deprecated`, `Superseded`, `Rejected`) ens ved arkivering til `tasks/archive/`, så `tasks/` kun rummer aktive og planlagte opgaver (`DRAFT`, `ACTIVE`, `IN_PROGRESS`, `WIP`, `TODO`, `REOPENED`), uden at pensionerede opgaver skal markeres fejlagtigt som `DONE`.

---

## 📋 Acceptance Criteria
- [ ] **AC1 — Terminal Status Helper (`TaskStatus::is_terminal`):**
  - Tilføj `TaskStatus::is_terminal(&self) -> bool` i `crates/xgauntlet-core/src/features/tasks/models.rs`, som returnerer `true` for `Done`, `Completed`, `Deprecated`, `Superseded` og `Rejected`, og `false` for alle ikke-terminale tilstande (`Draft`, `Active`, `InProgress`, `Wip`, `Todo`, `Reopened`, `Unknown`).
- [ ] **AC2 — Arkivering af alle terminale tilstande i `TaskScaffolder::archive_tasks`:**
  - Opdater `TaskScaffolder::archive_tasks` i `crates/xgauntlet-core/src/features/tasks/lifecycle.rs` til at bruge `info.status.is_terminal()` ved både enkelt-task (`task_id: Some(...)`) og samlet arkivering (`task_id: None`), inklusiv link-opdatering i `specs/**/*.md` og `--dry-run`-adfærd.
  - Ikke-terminale opgaver (`Draft`, `Active`, `InProgress`, `Wip`, `Todo`, `Reopened`, `Unknown`) afvises fortsat fail-closed ved eksplicit `--task`, med opdateret fejltekst der lister de godkendte terminale statusser.
- [ ] **AC3 — CLI & Living Spec Opdatering:**
  - Bevar `--all-done`-flaget på `xgauntlet task archive` af hensyn til bagudkompatibilitet, og opdater hjælpetekster samt `render_task_archive_summary` i `crates/xgauntlet-cli/src/{main.rs,checkers.rs}` og `specs/tasks.md` til at afspejle arkivering af terminale opgaver.
- [ ] **AC4 — Regressionstest:**
  - Udvid `crates/xgauntlet-core/tests/task_lifecycle_test.rs` med tests, der beviser at `DEPRECATED`, `SUPERSEDED` og `REJECTED` tasks arkiveres til `tasks/archive/` (med `specs/`-linkopdatering), mens `DRAFT`, `ACTIVE` og `TODO` forbliver uarkiverede og afvises ved eksplicit arkivering.

---

## 🚫 Must NOT
- **Må IKKE** arkivere ikke-terminale opgaver (`Draft`, `Active`, `InProgress`, `Wip`, `Todo`, `Reopened`, `Unknown`).
- **Må IKKE** ændre eksisterende `task archive --all-done` CLI-flagnavn eller bryde bagudkompatibilitet for `Done` / `Completed` opgaver.
- **Må IKKE** omskrive en pensioneret opgaves `status` til `done` ved arkivering (status i den arkiverede fil skal bevares uændret).
- **Må IKKE** introducere `slice`-terminologi eller bryde OKF v0.2 / `check-spec`-invarianter.

---

## 📝 Revisions
- 2026-11-14: Oprettet opgavepakke som `DRAFT` efter afklaring af Task 020 (`DEPRECATED`).

---

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 087`
- `cargo test -p xgauntlet-core --test task_lifecycle_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

