---
type: Task Package
title: "Task 014: Phase-Bound TDD Checkpoint Engine"
description: "Etablere features/checkpoint subsystem i crates/xgauntlet-core samt CLI subcommand 'checkpoint' med fase-specifikke pre-flight invariante kontroller (RED fejler, GREEN passerer, REFACTOR intakt) og automatiserede lokale conventional git commits jf. spec.md og ADR 0003"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T22:34:00Z" }
tags: [checkpoint, tdd, git, conventional-commits, pre-flight, fail-closed, adr-0001, adr-0003]
---

# Task 014: Phase-Bound TDD Checkpoint Engine

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere en fail-closed phase-checkpoint motor (`features/checkpoint`) i `crates/xgauntlet-core` samt CLI subcommand `checkpoint` i `crates/xgauntlet-cli` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md) og [ADR 0003](docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md):
1. **Fase-Specifikke Pre-Flight Invariante Kontroller**:
   - Sikre at lokale git commits kun gennemføres, når de metodiske kriterier for den pågældende TDD-fase er matematisk/mekanisk opfyldt:
     - `--phase spec`: Validerer at den aktive opgavefil i `tasks/` passerer `check-spec` med 0 fejl.
     - `--phase red`: Eksekverer testlaget og beviser, at testen rent faktisk *fejler*. Hvis testen uventet passerer, afvises commit med `RED_PHASE_UNMET` ("Expected failing test assertions in RED phase, but tests passed").
     - `--phase green`: Eksekverer testlaget og verificerer, at koden er *grøn* (exit code 0). Hvis testene fejler, afvises commit med `GREEN_PHASE_UNMET`.
     - `--phase refactor`: Verificerer at testene stadig passerer, og at test-assertions ikke er blevet modificeret eller lempet (Self-Mutation Invariant check).
     - `--phase done`: Eksekverer den fulde gauntlet pipeline, genererer `verification-report.json`, opdaterer `evidence.md` og forsegler evidensen.
2. **Automatiserede Lokale Conventional Git Commits**:
   - Automatisk staging af ændrede filer inden for det aktive task-scope (`git add`).
   - Automatisk sammensætning af konventionelle commit-beskeder med opgavenummer og fase-tag:
     - `test(<task>): <message> [RED]`
     - `feat(<task>): <message> [GREEN]`
     - `refactor(<task>): <message> [REFACTOR]`
     - `chore(<task>): <message> [DONE]`
   - Giver en fuldstændig gennemskuelig, atomisk og bisect-bar git-historik i `git log`.
3. **Strikt Håndhævelse af ADR 0003 (Ingen Remote Push)**:
   - Checkpoint-motoren må udelukkende udføre lokale Git-operationer.
   - Enhver handling der forsøger at interagere med remote remotes (`git push`) er forbudt og fysisk blokeret.
4. **CLI & JSON Integration**:
   - `xgauntlet checkpoint --phase <spec|red|green|refactor|done> -m "<besked>" [--workspace <sti>] [--skip-verify] [--json]`
   - Farvekodet terminalfeedback med øjeblikkeligt commit OID hash og filoversigt.

## 📋 Acceptance Criteria
- [ ] `crates/xgauntlet-core/src/features/checkpoint/models.rs` definerer `CheckpointPhase`, `CheckpointOptions`, `CheckpointResult` og `CheckpointError`.
- [ ] `crates/xgauntlet-core/src/features/checkpoint/preflight.rs` implementerer fase-specifikke invariante kontroller (`spec`, `red` skal fejle, `green` skal passere, `refactor` bevarer assertions, `done` fuld gauntlet).
- [ ] `crates/xgauntlet-core/src/features/checkpoint/git.rs` implementerer lokal staging (`git add`) og konventionel commit-generering med aktiv opgave-tagging uden tunge eksterne afhængigheder.
- [ ] `crates/xgauntlet-core/src/features/checkpoint/engine.rs` orkestrerer pre-flight evaluering, staging og commit-eksekvering med sub-5ms koldstart.
- [ ] `xgauntlet checkpoint` subcommand er tilgængelig i `crates/xgauntlet-cli` med understøttelse af `--phase`, `-m`/`--message`, `--skip-verify` og `--json`.
- [ ] Forsøg på at committe en grøn test under `--phase red` afvises med struktureret fejl og exit code 1.
- [ ] Forsøg på at committe en fejlet test under `--phase green` afvises med struktureret fejl og exit code 1.
- [ ] `crates/xgauntlet-core/tests/checkpoint_engine_test.rs` verificerer alle faser (spec, red, green, refactor, done) og commit-generering i et isoleret midlertidigt Git-repository.
- [ ] `cargo run -p xgauntlet-cli -- check-spec -t 014-phase-checkpoint-engine` validerer med 0 fejl.
- [ ] 100% grøn testsuite på tværs af hele workspacet (`cargo test --workspace`).

## 🚫 Must NOT
- Må ALDRIG udføre remote git publication handlinger (`git push`) jf. ADR 0003.
- Må IKKE udføre destruktive git handlinger (`git reset --hard`, `git clean -f`).
- Må IKKE tillade overspringelse af pre-flight verifikation i `--phase red` eller `--phase green` uden eksplicit `--skip-verify` flag.
- Må IKKE introducere eksterne baggrundsdæmoner eller runtime-sockets jf. Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-06: Task oprettet som ACTIVE for Phase-Bound TDD Checkpoint Engine (Task 014).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test checkpoint_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 014-phase-checkpoint-engine`
- `cargo run -p xgauntlet-cli -- checkpoint --help`
