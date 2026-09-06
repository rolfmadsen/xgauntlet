---
type: Task Package
title: "Task 005: Gauntlet Execution Engine"
description: "Implementere det deterministiske multi-layer gauntlet pipeline løb, processtyring med timeouts og exit-koder, diagnostic finding aggregation, samt kobling til pre/post manifest og self-mutation invarianten i crates/xgauntlet-core og xgauntlet-cli"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T17:40:00Z" }
tags: [gauntlet, pipeline, multi-layer, process-runner, timeouts, diagnostics, self-mutation, verification-report, adr-0001, adr-0005]
---

# Task 005: Gauntlet Execution Engine

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere Gauntlet Execution Engine (`features/gauntlet`) og udvide Diagnostics Engine (`features/diagnostics`) i `crates/xgauntlet-core` samt CLI-kommandoen `verify` i `crates/xgauntlet-cli` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md) samt [ADR 0001](docs/adr/0001-package-by-feature-architecture.md) og [ADR 0005](docs/adr/0005-two-tier-verification-and-attestation-model.md):
1. Implementere domænemodeller for lag og kørsel (`LayerExecutionStatus`, `LayerRequirement`, `LayerDefinition`, `LayerResult`, `GauntletReport`).
2. Implementere nul-afhængigheds parser for `gauntlet.toml` og `gauntlet.json` med standardlag for almindelige stacks.
3. Implementere asynkron processtyring med tidsgrænser (`timeout_seconds`), subprocess dræbning ved timeout, og præcise exit-koder:
   - `0`: Succes (`PASSED`)
   - `>0`: Fejl (`FAILED`)
   - `124`: Timeout overskredet (`TIMED_OUT`)
   - `127`: Kommando ikke fundet (`UNAVAILABLE`)
4. Implementere sekventiel pipeline-eksekvering med fail-closed semantik, der standser ved første obligatoriske fejl (`REQUIRED` og ikke `optional`).
5. Implementere handlingsorienteret diagnostik-ekstraktion (`features/diagnostics`) for linters (clippy, ruff, eslint), typecheckere (cargo check, tsc, pyright), test runners (cargo test, pytest, vitest), mutation testing (cargo mutants, mutants.py) og invarianter (proptest, hypothesis).
6. Sammenkoble kørslen med Workspace Manifest Engine:
   - Beregne pre-verifikationsmanifest (`compute_workspace_manifest`)
   - Opløse aktiv opgavekontrakt (`resolve_task_contract`)
   - Køre gauntlet-lagene
   - Beregne post-verifikationsmanifest (`compute_workspace_manifest`)
   - Håndhæve Self-Mutation Invarianten (`verify_self_mutation`)
   - Udlede samlet resultat (`PASSED`, `FAILED`, `PARTIAL`, `INCOMPLETE`)
   - Generere Schema v2 `VerificationReport` samt `evidence.md`
7. Tilføje `verify` kommando til `crates/xgauntlet-cli` med terminal HUD, opsummering og parametre (`--workspace`, `--task`, `--save`, `--json`, `--layer`).

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/gauntlet/models.rs` definerer `LayerExecutionStatus`, `LayerRequirement`, `LayerDefinition`, `LayerResult` og `GauntletReport`.
- [x] `crates/xgauntlet-core/src/features/gauntlet/config.rs` indlæser og parser `gauntlet.toml` og `gauntlet.json` uden eksterne parser-afhængigheder.
- [x] `crates/xgauntlet-core/src/features/gauntlet/runner.rs` eksekverer kommandoer asynkront med `tokio::process::Command`, håndhæver timeouts (exit code 124) og håndterer manglende binære filer (exit code 127).
- [x] Runneren standser øjeblikkeligt ved fejl på obligatoriske lag (fail-closed), mens valgfrie lag (`optional: true`) tillader fortsat eksekvering og udløser `PARTIAL` status.
- [x] `crates/xgauntlet-core/src/features/diagnostics/parser.rs` ekstraherer strukturerede `DiagnosticFinding` fra linter-, compiler- og testoutput med fil, linje og udbedringstip.
- [x] `crates/xgauntlet-core/src/features/gauntlet/pipeline.rs` sammenkobler pre- og post-manifest beregning, opløser opgavekontrakter og afviser kørslen ved brud på Self-Mutation Invarianten.
- [x] Pipeline genererer Schema v2 `VerificationReport` med fuld binding af manifest-hashes og diagnostic findings.
- [x] `crates/xgauntlet-cli` indeholder en velfungerende `verify` subcommand (`--workspace`, `--task`, `--save`, `--json`, `--layer`).
- [x] 100% grøn testsuite i `crates/xgauntlet-core/tests/gauntlet_execution_test.rs` og samtlige workspace checks.

## 🚫 Must NOT
- Må IKKE introducere eksterne runtime dæmoner eller baggrundsprocesser.
- Må IKKE tillade bestået verifikation (`PASSED`), hvis kildekode eller konfiguration er ændret under gauntlet-kørslen (Self-Mutation Invariant).
- Må IKKE introducere hardcodede hemmelige HMAC-nøgler jf. ADR 0005.
- Må IKKE foretage uautoriseret `git push` jf. ADR 0003.

## 📝 Revisions
- 2026-09-06: Oprettet som ACTIVE task for Gauntlet Execution Engine (Task 005).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test gauntlet_execution_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 005-gauntlet-execution-engine`
- `cargo run -p xgauntlet-cli -- verify --save`
- `cargo run -p xgauntlet-cli -- check-evidence`
