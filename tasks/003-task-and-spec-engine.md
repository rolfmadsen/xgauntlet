---
type: Task Package
title: "Task 003: Task & Spec Engine"
description: "Implementere OKF v0.2 Markdown parser for tasks/*.md, validering af formål, acceptkriterier og Must NOT, samt mekanisk Aristoteles validering af CONTEXT.md (genus et differentiam) og tilhørende check-spec valideringslogik i crates/xgauntlet-core"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T17:15:00Z" }
tags: [tasks, spec, okf, aristotle, context, parser, check-spec]
---

# Task 003: Task & Spec Engine

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere Task & Spec Engine (`features/tasks`) samt diagnostikmotor (`features/diagnostics`) i `crates/xgauntlet-core`:
1. Implementere en zero-dependency, deterministisk pure-Rust parser for Open Knowledge Format (OKF v0.2) YAML-frontmatter (`---` afgrænsning, type, status, actor formater, ISO 8601 UTC tidsstempler).
2. Implementere markdown parser for opgaver i `tasks/*.md` med udtræk af titel, status, formål (`## 🎯 Formål`), eksekverbare acceptkriterier (`- [ ]` / `- [x]`) og negative forretningsregler (`## 🚫 Must NOT`).
3. Implementere mekanisk validering af `CONTEXT.md` jf. Aristoteles' formel (*definitio per genus et differentiam* med obligatoriske `_Avoid_:` linjer for forbudte synonymer).
4. Implementere `check-spec` valideringsmotoren (`check_task_specification` og `check_all_tasks`), der udsender strukturerede `DiagnosticFinding` poster ved formaterings- eller indholdsfejl.
5. Integrere `check-spec` subcommand i `xgauntlet-cli` med både menneskevenlig og JSON-formateret visning.
6. Forbinde opgaveparseren til `EnforcementContext`, så policy-motoren automatisk kan detektere aktive opgaver i workspace.

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/diagnostics/` indeholder typed `DiagnosticFinding`, `FindingType` og `DiagnosticReport`.
- [x] `crates/xgauntlet-core/src/features/tasks/okf.rs` parser og validerer OKF v0.2 frontmatter, actors (`human:<id>`, `process:<id>`, `<agent>/<version>`) og ISO 8601 UTC timestamps.
- [x] `crates/xgauntlet-core/src/features/tasks/parser.rs` udtrækker formål, acceptkriterier (`- [ ]`), uafsluttede kriterier, negative invarianter (`Must NOT`) og opgavestatus.
- [x] `crates/xgauntlet-core/src/features/tasks/validator.rs` validerer mekanisk at `CONTEXT.md` følger Aristoteles' formel med `**Term**:`, gyldig definition og `_Avoid_:` linje (udløser `ARISTOTLE_FORMAT_VIOLATION` ved overtrædelse).
- [x] `check_task_specification` validerer eksistens af opgavefil (`TASK_FILE_NOT_FOUND`), OKF metadata (`INVALID_OKF_METADATA`), formål (`MISSING_PURPOSE`), kriterier (`MISSING_ACCEPTANCE_CRITERIA`), negative regler (`MISSING_MUST_NOT`) og `CONTEXT.md`.
- [x] `crates/xgauntlet-cli` har en fungerende `check-spec` subcommand (`--task`, `--all`, `--json`, `--workspace`).
- [x] 100% grøn testsuite med omfattende dækning i `crates/xgauntlet-core/tests/tasks_engine_test.rs`.

## 🚫 Must NOT
- Må IKKE introducere tunge eksterne C-afhængigheder eller upålidelige runtime-evalueringer til parsing.
- Må IKKE fejle åbent (fail-open); ufuldstændige eller syntaktisk ugyldige specifikationer SKAL afvises som ugyldige (`is_valid = false`).
- Må IKKE ændre på eksisterende policy engine adfærd eller tillade kildekodemutering uden aktiv opgave.
- Må IKKE foretage utilsigtede git remote kommandoer (`git push`).

## 📝 Revisions
- 2026-09-06: Task oprettet jf. spec.md og ADR 0001 (Package-by-Feature).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 003-task-and-spec-engine`
