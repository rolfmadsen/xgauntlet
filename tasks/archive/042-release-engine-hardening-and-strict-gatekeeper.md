---
type: Task Package
title: "Task 042: Release Engine Hardening, Strict Gatekeeper Default & Tag Isolation"
description: "Hærdning af xGauntlets release-motor med komplet manifest- og evidens-staging, isoleret tag-push instruks og obligatorisk strict gatekeeper validering"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-18T15:13:00Z" }
tags: [release, gatekeeper, strict, tagging, evidens, manifests, git]
scope:
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-cli/src/**"
  - ".agents/plugins/xgauntlet/skills/**"
test_refs:
  - "crates/xgauntlet-core/tests/handoff_and_release_test.rs"
---

# Task 042: Release Engine Hardening, Strict Gatekeeper Default & Tag Isolation

**Status**: `DONE`  
**Intent**: `🔄 ENHANCEMENT`  
**Oprettet**: `2026-10-18`  

## 🎯 Formål
Eliminere release-friktion, automatisere evidensforsegling og rense sovereign skills for stack-lækager:
1. **Workspace-Agnostisk Release Git-Instruktion**: Gøre `generate_release_git_instructions` og `generate_release_git_instructions_with_files` universelt polyglotte, så instruktionerne udelukkende stager de faktiske modificerede manifester for det aktuelle workspace uden hardcodet xGauntlet-coupling.
2. **Isoleret Tag-Push**: Permanent udfasning af `--tags` til fordel for `git push origin <branch> v<version>`.
3. **Automatisk Evidensforsegling & Strict Gatekeeper under Release Bump**: `xgauntlet release bump` kører automatisk `verify --save` for den nye bumped version og validerer 0 drift med strict gatekeeper (`strict: true`) før instruktionerne udskrives.
4. **Isoleret Strict Gatekeeper**: `xgauntlet check-release --strict` fanger workspace-drift i CI og lokale audits, og enforced automatisk under release bump.
5. **Polyglot Sovereign Skills**: Rense `release/SKILL.md` og `review/SKILL.md` for xGauntlet-interne stier, så de fungerer transparent på tværs af Rust (Kant), TypeScript, Python og Go.

---

## 📋 Acceptance Criteria
- [x] **AC1 (Workspace-Agnostiske Git-Instruktioner)**: `generate_release_git_instructions` og `generate_release_git_instructions_with_files` genererer rene instruktioner baseret på faktiske workspace-filer og `git add -u` uden hardcodede xGauntlet-stier (`crates/.../wasm`, `packages/cli/...`).
- [x] **AC2 (Isoleret Tag-Push)**: Instruktionen udsteder `git push origin <branch> v<version>` i stedet for det risikable `--tags`.
- [x] **AC3 (Automatisk Evidensforsegling & Strict Gatekeeper)**: `xgauntlet release bump` forsegler automatisk frisk evidens og kører strict release-audit transaktionelt.
- [x] **AC4 (Polyglot Sovereign Skills)**: `.agents/plugins/xgauntlet/skills/release/SKILL.md` og `review/SKILL.md` er fri for interne xGauntlet-stier og understøtter alle stacks.
- [x] **AC5 (Testparitet)**: `handoff_and_release_test.rs` verificerer både universel fallback og dynamiske workspace-filer.

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
