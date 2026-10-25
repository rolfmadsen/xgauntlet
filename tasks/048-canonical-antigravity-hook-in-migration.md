---
type: Task Package
title: "Task 048: Canonical Antigravity Hook in Migration"
description: "Rette migrationens defekte PreInvocation-hook, normalisere eksisterende defekte hooks og fjerne tavs fejlhåndtering i doctor --fix"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-25T19:17:00Z" }
tags: [migration, doctor, antigravity, hooks, single-source-of-truth, fail-closed]
scope:
  - "crates/xgauntlet-core/src/features/migration/**"
  - "crates/xgauntlet-core/src/features/adapters/antigravity/**"
  - "crates/xgauntlet-core/src/features/scaffold/templates.rs"
  - "crates/xgauntlet-core/src/features/doctor/**"
  - "crates/xgauntlet-core/tests/doctor_diagnostics_test.rs"
  - "tasks/048-canonical-antigravity-hook-in-migration.md"
test_refs:
  - "crates/xgauntlet-core/tests/migration_hook_canonical_test.rs"
---

# Task 048: Canonical Antigravity Hook in Migration

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-25`

## 🎯 Formål
Code review (fund 1) viste, at `migration/engine.rs` indsætter
`xgauntlet telemetry --harness antigravity-hook`. `antigravity-hook` er et *format*; CLI'en falder
tilbage til `box` og udskriver et ANSI-kort i stedet for JSON-payload. Desuden betragter
`has_telemetry` enhver `xgauntlet telemetry`-kommando som gyldig, så gentagen migration aldrig
retter fejlen. Kommandostrengen er duplikeret i adapter, scaffold og migration.

Inkluderer review 🟡 #1 (`doctor --fix` smider migrationsfejl væk) og 🟡 #2 (tavs overskrivning ved
samtidig legacy- og kanonisk hooks-gruppe).

## 📋 Acceptance Criteria
- [ ] **AC1 (Single source of truth)**: Én offentlig konstant for Antigravity PreInvocation-kommandoen (`xgauntlet telemetry --format antigravity-hook`) i `adapters/antigravity`, genbrugt af adapter, scaffold-template og migration.
- [ ] **AC2 (Korrekt injektion)**: Migration af `hooks.json` uden telemetry-hook indsætter præcis den kanoniske kommando.
- [ ] **AC3 (Normalisering)**: Migration af `hooks.json` med `--harness antigravity-hook` (eller `--harness antigravity`) erstatter den med den kanoniske kommando, rapporterer ændringen og er idempotent ved anden kørsel.
- [ ] **AC4 (Ingen tavs overskrivning)**: Findes både `agent-gauntlet-gatekeeper` og `xgauntlet`, bevares den kanoniske gruppes hooks (merge uden dubletter), og ændringslisten nævner sammenfletningen.
- [ ] **AC5 (doctor --fix er ærlig)**: Fejler migrationen under `doctor --fix`, rapporteres fejlen i doctor-resultatet i stedet for at blive kasseret.
- [ ] **AC6 (Fixture-korrektion)**: `doctor_diagnostics_test.rs` bruger ikke længere den defekte streng som "korrekt" fixture.

## 🚫 Must NOT
- Må IKKE ændre CLI'ens `--harness`-semantik i `main.rs` (fejlen rettes ved kilden).
- Må IKKE slette brugerdefinerede hooks, der ikke er xgauntlet-telemetry.
- Må IKKE bryde idempotens for allerede kanoniske `hooks.json`.
- Må IKKE foretage remote publication (`git push`).

## 📝 Revisions
- 2026-10-25: Task oprettet på baggrund af code review af `origin/main..HEAD` (fund 1, 🟡 #1, 🟡 #2).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test migration_hook_canonical_test`
- `cargo test -p xgauntlet-core --test migration_engine_test --test doctor_diagnostics_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 048-canonical-antigravity-hook-in-migration`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
