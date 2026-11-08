---
type: Task Package
title: "Task 067: Synchronize NPM Package README & Add Parity Test Gate"
description: "Synkronisere packages/cli/README.md med rod-README.md og etablere automatisk paritetstest i testsuiten samt release-gatekeeperen."
status: active
intent: bug
generated: { by: process:antigravity-task-init, at: "2026-11-08T12:56:00Z" }
tags: [npm, readme, release, parity, gatekeeper, documentation]
scope:
  - "packages/cli/README.md"
  - "crates/xgauntlet-core/src/features/release/engine.rs"
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "tasks/067-sync-npm-package-readme-and-parity-gate.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
---

# Task 067: Synchronize NPM Package README & Add Parity Test Gate

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Sikre at forsiden på NPM Registry (`xgauntlet`) altid afspejler repositoryets aktuelle dokumentation ved at synkronisere `packages/cli/README.md` med rod-`README.md`. Samtidig etableres en mekanisk test i testsuiten samt en kontrol i release readiness gatekeeperen (`xgauntlet check-release`), der forhindrer enhver fremtidig dokumentations-drift mellem monorepoets rod og NPM-pakken.

## 📋 Acceptance Criteria
- [ ] **Acceptance Test for README Paritet**:
  - [ ] En test i `release_manifest_parity_test.rs` verificerer, at `packages/cli/README.md` er 100% identisk med rod-`README.md`.
- [ ] **Synkronisering af `packages/cli/README.md`**:
  - [ ] `packages/cli/README.md` opdateres, så den matcher rod-`README.md` byte-for-byte.
- [ ] **Release Gatekeeper Enforcement**:
  - [ ] `xgauntlet check-release` (`ReleaseReadinessEngine`) validerer, at `packages/cli/README.md` matcher rod-`README.md`, og rapporterer en blokerende `ReleaseFinding`, hvis der detekteres drift.
- [ ] **Verifikation**:
  - [ ] `cargo test -p xgauntlet-core --test release_manifest_parity_test` passerer.
  - [ ] `xgauntlet check-release` rapporterer fuld release-paritet.

## 🚫 Must NOT
- Må IKKE udføre remote publication (`git push`) jf. ADR 0003.
- Må IKKE ændre eksisterende manifest-versioner.

## 📝 Revisions
- 2026-11-08: Oprettet opgavepakke efter opdagelse af README-drift i NPM CLI pakken.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo run -p xgauntlet-cli -- check-release`
