---
type: Task Package
title: "Task 082: Archive Completed Tasks to tasks/archive/"
description: "Ryd op i roden af tasks/ ved at arkivere samtlige historiske DONE/COMPLETED-tasks til tasks/archive/ og opdatere relative task-links i specs/**/*.md efter reglerne fra Task 079."
status: done
intent: chore
slice: tasks
spec_ref: "specs/tasks.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T16:56:00Z" }
tags: [tasks, archive, cleanup, dx, housekeeping]
scope:
  - "tasks/**"
  - "specs/**"
  - "evidence.md"
  - "verification-report.json"
test_refs:
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
---

# Task 082: Archive Completed Tasks to tasks/archive/

**Status**: `DONE`
**Intent**: `🧹 CHORE`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Anvende den nye arkiveringsmotor fra Task 079 (`xgauntlet task archive --all-done`) til at rydde op i roden af `tasks/`, så kun igangværende eller planlagte (`ACTIVE`/`DRAFT`) opgaver samt skabeloner (`README.md`, `TEMPLATE.md`) ligger i topniveauet, mens færdige (`DONE`) historiske opgaver flyttes til `tasks/archive/` med automatisk opdatering af relative links i `specs/**/*.md`.

## 📋 Acceptance Criteria
- [x] **AC1 (Dry-Run Preview & Arkivering af Historiske `DONE`-Tasks):**
  - Alle `DONE`/`COMPLETED`-tasks i roden af `tasks/` flyttes til `tasks/archive/` via `xgauntlet task archive --all-done`.
- [x] **AC2 (Bevarelse af `DRAFT`/`ACTIVE`-Tasks & Link-Integritet):**
  - `DRAFT`-tasks (`080`, `081`) samt `README.md` og `TEMPLATE.md` forbliver urørt i roden af `tasks/`, og relative task-links i `specs/**/*.md` peger korrekt på `../tasks/archive/...`.
- [x] **AC3 (Fuldt Verificeret Repo efter Oprydning):**
  - `check-spec --context`, `check-architecture` og `cargo test --workspace` består efter arkiveringen uden regressioner.

## 🚫 Must NOT
- Må IKKE flytte eller ændre `DRAFT`- eller `ACTIVE`-tasks (`080`, `081`) til `tasks/archive/`.
- Må IKKE ændre Rust-kildekode i `crates/` under denne oprydningsopgave.
- Må IKKE slette historiske opgavefiler eller nulstille task-nummereringen.

## 📝 Revisions
- 2026-11-14: Oprettet oprydnings-task 082 for at arkivere eksisterende `DONE`-tasks til `tasks/archive/`.
- 2026-11-14: Arkiveret 76 færdige tasks til `tasks/archive/` og opdateret relative links i `specs/architecture.md`, `specs/release.md`, `specs/tasks.md` og `specs/topology.md`. Sat til `DONE`.

## 🧪 Verifikation
- `target/debug/xgauntlet task archive --all-done --dry-run`
- `target/debug/xgauntlet task archive --all-done`
- `target/debug/xgauntlet check-spec --context`
- `cargo test --workspace`

