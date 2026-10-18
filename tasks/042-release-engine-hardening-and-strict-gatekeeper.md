---
type: Task Package
title: "Task 042: Release Engine Hardening, Strict Gatekeeper Default & Tag Isolation"
description: "Hærdning af xGauntlets release-motor med komplet manifest- og evidens-staging, isoleret tag-push instruks og obligatorisk strict gatekeeper validering"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-18T15:13:00Z" }
tags: [release, gatekeeper, strict, tagging, evidens, manifests, git]
scope:
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-cli/src/render.rs"
  - ".agents/plugins/xgauntlet/skills/release/**"
test_refs:
  - "crates/xgauntlet-core/tests/handoff_and_release_test.rs"
---

# Task 042: Release Engine Hardening, Strict Gatekeeper Default & Tag Isolation

**Status**: `ACTIVE`  
**Intent**: `🔄 ENHANCEMENT`  
**Oprettet**: `2026-10-18`  

## 🎯 Formål
Eliminere release-friktion og forebygge regressionsfejl under udgivelser af xGauntlet:
1. **Komplet Manifest- og Evidens-Staging i Release Git-Instruktioner**: Sikre at `generate_release_git_instructions` genererer en autoritativ `git add`-kommando, som omfatter alle modificerede filer under et release-bump: `Cargo.toml`, `Cargo.lock`, `package.json`, `packages/cli/package.json`, `CHANGELOG.md`, `crates/xgauntlet-core/wasm/gauntlet_policy.wasm`, `verification-report.json` og `evidence.md`.
2. **Isoleret Tag-Push**: Erstatte det risikable `git push origin <branch> --tags` med præcis tag-adressering: `git push origin <branch> v<version>`, så utilsigtede eller historisk divergerende tags i et lokalt Git-lager aldrig blokerer for udgivelsen.
3. **Sovereign Skill Strict Governance**: Opdatere `.agents/plugins/xgauntlet/skills/release/SKILL.md` så `check-release --strict` anvendes obligatorisk, og sekvensen mellem versions-bump og evidensforsegling (`verify --save`) er krystalklar.

---

## 📋 Acceptance Criteria
- [ ] **AC1 (Komplet Staging i Git-Instruktioner)**: `generate_release_git_instructions` inkluderer samtlige release-aktiver i `git add` (`Cargo.toml Cargo.lock package.json packages/cli/package.json CHANGELOG.md crates/xgauntlet-core/wasm/gauntlet_policy.wasm verification-report.json evidence.md`) eller `git add -u`.
- [ ] **AC2 (Isoleret Tag-Push Instruks)**: `generate_release_git_instructions` udsteder `git push origin <branch> v<version>` i stedet for `--tags`.
- [ ] **AC3 (Sovereign Skill Strict Audit)**: `.agents/plugins/xgauntlet/skills/release/SKILL.md` instruerer eksplicit i at anvende `xgauntlet check-release --strict` i både release-audit og final verification.
- [ ] **AC4 (Testparitet)**: Enhedstesten `test_generate_release_git_instructions` i `crates/xgauntlet-core/tests/handoff_and_release_test.rs` opdateres til at verificere den isolerede tag-push og komplette staging.

---

## 🚫 Must NOT
- Må IKKE udføre remote git publication (`git push`) jf. ADR 0003.
- Må IKKE fjerne eller slække på `--strict` tjekket i CI (`.github/workflows/release.yml`).

---

## 📝 Revisions
- 2026-10-18: Oprettet opgavepakke efter release-triage af v0.8.5.

---

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test handoff_and_release_test`
- `cargo run -p xgauntlet-cli -- check-spec`
- `cargo run -p xgauntlet-cli -- check-release --strict`
