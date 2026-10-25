---
type: Task Package
title: "Task 049: Mechanical Monotonic Ratchet Enforcement"
description: "Implementere den manglende AC4 fra Task 046: mekanisk afvisning af hævede ratchet-lofter uden eksplicit ADR-dækning, og rulle inflationen tilbage"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-25T19:17:00Z" }
tags: [architecture, ratchet, adr, governance, fail-closed, adr-0010]
scope:
  - "crates/xgauntlet-core/src/features/architecture/**"
  - "gauntlet.toml"
  - "docs/adr/0010-adr-bound-architectural-exemptions-and-monotonic-ratchets.md"
  - "tasks/046-adr-bound-architectural-exemptions-and-monotonic-ratchets.md"
  - "tasks/049-mechanical-monotonic-ratchet-enforcement.md"
test_refs:
  - "crates/xgauntlet-core/tests/architecture_ratchet_test.rs"
---

# Task 049: Mechanical Monotonic Ratchet Enforcement

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-25`

## 🎯 Formål
Code review (fund 2) viste, at Task 046 AC4 er markeret `[x]` uden implementering:
`checker.rs` sammenligner kun filstørrelse med loft, aldrig loft med tidligere loft. Historikken
viser inflation 2100 → 2160 (9dc6319) → 2190 (de771be, RED-commit uden for scope, uden ADR).
`main.rs` er i dag 2180 linjer.

## 📋 Acceptance Criteria
- [ ] **AC1 (Baseline-sammenligning)**: `check-architecture` sammenligner hver exemptions `max_lines` med loftet i baseline-`gauntlet.toml`. Baseline = merge-base med upstream (`@{upstream}`), ellers `HEAD`.
- [ ] **AC2 (Afvisning af inflation)**: En stigning afvises som violation, medmindre den refererede ADR indeholder en eksplicit linje `ratchet: <path> <= <ny værdi>`.
- [ ] **AC3 (Tilladte ændringer)**: Uændrede eller sænkede lofter, og nye exemptions med gyldig ADR, passerer.
- [ ] **AC4 (Workspace-agnosticisme)**: Uden git-repo springes sammenligningen over med en eksplicit notits; findes repo, men kan baseline ikke læses, fejler checket lukket.
- [ ] **AC5 (Tilbagerulning)**: `main.rs`-loftet i `gauntlet.toml` sænkes til 2180, og ADR 0010 dokumenterer ratchet-linje-konventionen.
- [ ] **AC6 (Ærlig evidens)**: Task 046 AC4 sættes til `[ ]` med revisionsnote, der henviser til Task 049.

## 🚫 Must NOT
- Må IKKE acceptere en stigning blot fordi ADR-filen er ændret i samme diff.
- Må IKKE kræve netværksadgang (ingen `git fetch`).
- Må IKKE bryde serde-bagudkompatibilitet for `ArchitectureExemption`.
- Må IKKE foretage remote publication (`git push`).

## 📝 Revisions
- 2026-10-25: Task oprettet på baggrund af code review af `origin/main..HEAD` (fund 2).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test architecture_ratchet_test`
- `cargo test -p xgauntlet-core --test architecture_gate_test`
- `cargo run -p xgauntlet-cli -- check-architecture`
- `cargo run -p xgauntlet-cli -- check-spec -t 049-mechanical-monotonic-ratchet-enforcement`
- `cargo test --workspace`
