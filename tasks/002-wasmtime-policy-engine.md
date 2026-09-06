---
type: Task Package
title: "Task 002: Embedded Wasmtime Host & Deterministic Policy Engine"
description: "Implementere Wasmtime runtime host (features/wasm) og deterministisk policy engine (features/policy) i crates/xgauntlet-core med Nul Ambient Authority"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T17:10:00Z" }
tags: [wasm, wasmtime, policy, security, gatekeeper, zero-ambient-authority]
---

# Task 002: Embedded Wasmtime Host & Deterministic Policy Engine

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere den sikre, deterministiske policy-evalueringsmotor i `crates/xgauntlet-core`:
1. Implementere `features/wasm`: Indlejre `wasmtime` til in-memory eksekvering af WebAssembly-komponenter (`gauntlet_policy_engine.wasm`) med absolut Nul Ambient Authority (nul WASI-privilegier, intet filsystem, intet netværk, intet system-ur, ingen tilfældighedskilder).
2. Implementere `features/policy`: Domænemodeller for `CapabilityRequest`, `EnforcementContext`, `PolicyDecision`, samt evalueringstrait `PolicyEvaluator` og `WasmPolicyEngine`.
3. Indlejre prekompileret deterministisk WebAssembly bytecode i `xgauntlet-core`, så motoren fungerer 100% self-contained uden eksterne filer på værtsmaskinen.
4. Håndhæve gatekeeper-regler med fail-closed semantik:
   - Frit læseadgang (`ReadFile`).
   - Read-only workspace enforcement (`ctx.read_only == true` blokerer skrivning og kommandoer).
   - Skrivebeskyttelse af produktion (`src/`, `tests/`, `crates/`, `packages/`, `.agents/`) kræver aktiv task.
   - Dokumentations- og opgavestier (`tasks/`, `docs/`, `spec.md`, `CONTEXT.md`, etc.) tillades under planlægning.
   - Hård blokering af destruktive kommandoer (`git push`, `git reset --hard`, `git clean -f`, `git branch -D`, `rm -rf /`).
   - Whitelist af ufarlige inspektions- og verifikationskommandoer (`cargo test`, `git status`, `xgauntlet verify`, etc.).
5. Fuld sort-boks testdækning og paritetsvalidering mellem WebAssembly-motoren og reference-evaluatoren.

## 📋 Acceptance Criteria
- [x] `Cargo.toml` konfigureret med `wasmtime` og workspace-afhængigheder.
- [x] `crates/xgauntlet-core/src/features/wasm/mod.rs` indeholder `WasmRuntimeHost` med in-memory eksekvering og sikker linear memory håndtering (`alloc`, `dealloc`, `evaluate_json_wasm`).
- [x] `crates/xgauntlet-core/src/features/policy/mod.rs` indeholder typed `CapabilityRequest`, `EnforcementContext`, `PolicyDecision` og `WasmPolicyEngine`.
- [x] Policy engine afviser destruktive git-kommandoer (`git push`, `git reset --hard`) uanset task-status med grundkode 4039.
- [x] Policy engine tillader skrivning til dokumentation (`tasks/`, `spec.md`, etc.) uden aktiv task med grundkode 2001.
- [x] Policy engine afviser ændringer i kildekode (`src/`, `crates/`, etc.) uden aktiv task med grundkode 4031, men tillader under aktiv task med grundkode 2002.
- [x] Policy engine respekterer `ctx.read_only == true` og afviser muteringer med grundkode 4030.
- [x] Sub-5ms in-memory koldstart og evalueringstid.
- [x] 100% grøn testsuite i `crates/xgauntlet-core` og på tværs af hele workspace.

## 🚫 Must NOT
- WebAssembly modulet må IKKE tildeles nogen ambient capabilities (ingen filadgang, ingen netværksadgang, intet ur, ingen systemkald).
- Må IKKE introducere baggrundsdæmoner, Unix sockets eller OS service managers.
- Må IKKE fejle åbent (fail-open); enhver fejl i parsing eller memory SKAL resultere i en afvisning (`Deny`).
- Må IKKE foretage remote git-operationer (`git push`).

## 📝 Revisions
- 2026-09-06: Task oprettet og forankret i spec.md og ADR 0006/0007.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `npm test --workspaces`
