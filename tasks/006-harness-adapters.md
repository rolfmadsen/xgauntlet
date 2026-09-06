---
type: Task Package
title: "Task 006: Harness Adapters"
description: "Implementere autonome vertikale feature-slices for Google Antigravity IDE, Claude Code og OpenAI Codex, herunder normalisering, mekanisk plugin- og manifest-validering, fail-closed hook evaluering og conformance testsuite i crates/xgauntlet-core og xgauntlet-cli"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T17:50:00Z" }
tags: [adapters, antigravity, claude-code, codex, harness, normalization, plugin-validation, hook, fail-closed, adr-0001, adr-0004, adr-0006]
---

# Task 006: Harness Adapters

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere Harness Adapters (`features/adapters`) i `crates/xgauntlet-core` samt CLI-kommandoerne `hook` og `validate-plugin` i `crates/xgauntlet-cli` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md) samt [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):
1. Etablere de harness-agnostiske modeller i `features/adapters/models.rs`:
   - `NormalizedToolCall`, `ValidationSeverity`, `ValidationIssue`, `AdapterValidationResult`, `AdapterHookVerdict`.
   - `HarnessAdapter` trait med metoder for normalisering, evaluering, plugin-validering og hook-eksekvering.
2. Etablere adapter-registry i `features/adapters/mod.rs` med `SUPPORTED_HARNESSES` (`antigravity`, `claude_code`, `codex`) og `get_adapter`.
3. Implementere Google Antigravity IDE vertical slice (`features/adapters/antigravity`):
   - Normalisering af canonical `toolCall` og flat payload.
   - Evaluering mod WebAssembly policy engine og trusted enforcement context.
   - Fail-closed stdout/exit-code hook-kontrakt (`{"decision": "allow"}` ved 0, `{"decision": "deny"}` ved 1).
   - Mekanisk validering af `plugin.json` og `hooks.json`.
4. Implementere Claude Code vertical slice (`features/adapters/claude_code`):
   - Normalisering af Claude Code værktøjer (`Bash`, `FileEdit`, `FileWrite`, `FileRead`, `LS`, `GrepTool`, etc.).
   - Fail-closed exit code 1 med struktureret afvisning.
   - Validering af Claude Code konfigurationer og manifest.
5. Implementere OpenAI Codex vertical slice (`features/adapters/codex`):
   - Normalisering af OpenAI function/tool calling formater.
   - Fail-closed exit code 1 med struktureret afvisning.
   - Validering af Codex plugin manifest (`ai-plugin.json` / spec).
6. Implementere parameteriseret Shared Conformance Test Suite (ADR 0006 §4) testende samtlige adaptere for typed translation, fail-closed ved malformed/tomme input, immutability af context og read-only / command denial håndhævelse.
7. Tilføje `hook` og `validate-plugin` subcommands til `crates/xgauntlet-cli`.

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/adapters/models.rs` definerer `NormalizedToolCall`, `ValidationSeverity`, `ValidationIssue`, `AdapterValidationResult`, `AdapterHookVerdict` og `HarnessAdapter` trait.
- [x] `crates/xgauntlet-core/src/features/adapters/mod.rs` definerer `SUPPORTED_HARNESSES` og funktion til instantiere adaptere (`get_adapter`).
- [x] `crates/xgauntlet-core/src/features/adapters/antigravity/` normaliserer Antigravity tool-kald, validerer `plugin.json` og `hooks.json` og håndterer lifecycle hooks fail-closed.
- [x] `crates/xgauntlet-core/src/features/adapters/claude_code/` normaliserer Claude Code tool-kald, validerer Claude manifest og håndterer hooks fail-closed.
- [x] `crates/xgauntlet-core/src/features/adapters/codex/` normaliserer OpenAI function/tool-kald, validerer Codex manifest og håndterer hooks fail-closed.
- [x] Samtlige adaptere overholder ADR 0006 om eksplicit separation mellem CapabilityRequest og Trusted Enforcement Context (kan ikke overskrives af untrusted input).
- [x] Shared Conformance Test Suite i `crates/xgauntlet-core/tests/harness_adapters_test.rs` verificerer alle adaptere for typet oversættelse, fail-closed adfærd og kontekst-immutability.
- [x] `crates/xgauntlet-cli` understøtter `hook` og `validate-plugin` subcommands.
- [x] 100% grøn testsuite og gauntlet verification pass rate.

## 🚫 Must NOT
- Må IKKE give adaptere kryptografisk autoritet til signering eller ændring af verifikationsrapporter jf. ADR 0006.
- Må IKKE tillade untrusted tool payloads at overskrive `EnforcementContext` parametre (workspace-id, active task, read-only).
- Må IKKE fejle åbent (fail-open) ved malformed, ukendt eller tomt JSON input; skal altid returnere fail-closed med exit code 1.
- Må IKKE introducere baggrunds-dæmoner eller socket-afhængigheder (Zero-Daemon Invariant).

## 📝 Revisions
- 2026-09-06: Oprettet som ACTIVE task for Harness Adapters (Task 006).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 006-harness-adapters`
- `cargo run -p xgauntlet-cli -- validate-plugin -p .agents --harness antigravity`
- `cargo run -p xgauntlet-cli -- verify --save`
