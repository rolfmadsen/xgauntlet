---
type: Task Package
title: "Task 033: Sovereign Agentic SDLC, Task Scoping & Accessible Design Tokens"
description: "Etablering af suveræne xGauntlet skills, maskinel task scoping og tilgængelige design tokens jf. ADR 0008"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-10-16T18:12:00Z" }
tags: [task-lifecycle, task-scoping, policy-engine, adr-0008, okf, accessibility]
scope:
  - "crates/xgauntlet-core/src/features/tasks/**"
  - "crates/gauntlet-policy-engine/**"
  - "wit/gauntlet_policy.wit"
test_refs:
  - "crates/xgauntlet-core/tests/tasks_engine_test.rs"
  - "crates/xgauntlet-core/tests/policy_engine_test.rs"
---

# Task 033: Sovereign Agentic SDLC, Task Scoping & Accessible Design Tokens

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-16`  

## 🎯 Formål
Etablere grundlaget for xGauntlets suveræne Agentic SDLC jf. ADR 0008 ved at:
1. Udvide OKF v0.2 opgavemodellen med obligatorisk `scope: [...]` og `test_refs: [...]`.
2. Validere i `check-spec`, at aktive opgaver har deklarerede stier og dedikerede testfiler (og forhindre test-dumping i monolitiske filer over 1.000 linjer).
3. Håndhæve opgavens scope deterministisk i WASM policy-motoren (`EnforcementContext`), så agenter fysisk forhindres i at modificere kode uden for det tilladte arbejdsområde.

## 📋 Acceptance Criteria
- [ ] **AC1 (OKF Model Udvidelse)**: `OkfMetadata` i `crates/xgauntlet-core/src/features/tasks/okf.rs` og `TaskPackageInfo` i `models.rs` udvides med `scope: Vec<String>` og `test_refs: Vec<String>` med fail-safe defaults for bagudkompatibilitet.
- [ ] **AC2 (Task Scope & Test-Target Validering)**: `check_task_specification` i `validator.rs` validerer, at enhver aktiv task deklarerer mindst én sti under `scope` og mindst én testfil under `test_refs`.
- [ ] **AC3 (Anti-Monolit Test Guard)**: `check_task_specification` udsteder en diagnostisk fejl (`MONOLITHIC_TEST_FILE_PROHIBITED`), hvis et deklareret `test_refs` mål overskrider 1.000 linjer, medmindre tasken specifikt er en refactoring-opgave.
- [ ] **AC4 (WASM EnforcementContext Udvidelse)**: `EnforcementContext` i `crates/gauntlet-policy-engine/src/lib.rs` udvides med `allowed_scopes: Vec<String>` og `allowed_test_targets: Vec<String>`.
- [ ] **AC5 (WASM Scope Gatekeeper)**: `evaluate()` i policy-motoren afviser (`DecisionVerdict::Deny`, fejlkode `4037`) enhver `WriteFile` handling mod kildekode (`src/**`, `crates/**`) eller tests (`tests/**`), der falder uden for taskens tilladte stier.
- [ ] **AC6 (Regression & Invariant Validering)**: Samtlige eksisterende tests i `tasks_engine_test.rs` og `policy_engine_test.rs` passerer uden regressionsfejl.

## 🚫 Must NOT
- Må IKKE bryde bagudkompatibilitet for eksisterende opgavefiler uden `scope` (skal parses sikkert som tomme lister).
- Må IKKE tillade path traversal (`..` eller absolutte rødder) i `allowed_scopes` matchning.
- Må IKKE foretage remote publication handlinger (`git push`).
- Må IKKE introducere baggrunds-dæmoner eller socket-filer (Zero-Daemon invarianten).

## 📝 Revisions
- 2026-10-16: Task 033 initialiseret jf. ADR 0008 som ACTIVE for Task Scoping og Policy Enforcement.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test tasks_engine_test`
- `cargo test -p xgauntlet-core --test policy_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-spec -t 033-sovereign-agentic-sdlc-task-scoping-and-design-tokens`
