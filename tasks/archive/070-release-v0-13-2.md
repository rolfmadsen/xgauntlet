---
type: Task Package
title: "Task 070: Release v0.13.2 Preparation & Manifest Bump"
description: "Opgradere samtlige versionsmanifester til 0.13.2, genkompilere WebAssembly policy bytecode, rotere CHANGELOG.md og forsegle release-readiness."
status: done
intent: enhancement
slice: release
spec_ref: "specs/release.md"
generated: { by: process:antigravity-task-init, at: "2026-11-08T15:17:00Z" }
tags: [release, semver, bump, changelog, wasm, manifests]
scope:
  - "Cargo.toml"
  - "Cargo.lock"
  - "crates/gauntlet-policy-engine/Cargo.toml"
  - "crates/xgauntlet-cli/Cargo.toml"
  - "crates/xgauntlet-core/Cargo.toml"
  - "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
  - "package.json"
  - "packages/cli/package.json"
  - ".agents/plugins/xgauntlet/plugin.json"
  - ".agents/plugins/xgauntlet/policy.json"
  - "CHANGELOG.md"
  - "specs/**"
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "tasks/069-ddd-slice-taxonomy-and-dogfooding.md"
  - "tasks/070-release-v0-13-2.md"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
---

# Task 070: Release v0.13.2 Preparation & Manifest Bump

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Forberede udgivelse af version `0.13.2` (patch/bugfix bump på `0.13.1`) for at muliggøre publicering til GitHub og NPM Registry, synkronisere samtlige 8 manifester, genkompilere WebAssembly policy engine til `0.13.2`, opdatere `CHANGELOG.md` med Task 069 og validere 100% release readiness via `xgauntlet check-release`.

## 📋 Acceptance Criteria
- [x] **Acceptance Test for Version 0.13.2**:
  - [x] `release_manifest_parity_test.rs` opdateres til at kræve version `0.13.2` på tværs af alle manifester.
- [x] **Manifest-opgradering til 0.13.2**:
  - [x] Opdatere alle 8 manifester til `0.13.2` (`Cargo.toml`, `package.json`, `packages/cli/package.json`, `plugin.json`, `policy.json` mv.).
- [x] **WASM Policy Bytecode Rebuild**:
  - [x] Genkompilere `gauntlet-policy-engine` til version `0.13.2` og opdatere `crates/xgauntlet-core/wasm/gauntlet_policy.wasm`.
- [x] **Changelog Rotation**:
  - [x] `CHANGELOG.md` opdateres med `[0.13.2] - 2026-11-08` dækkende Task 069 (DDD/VSA taksonomi, 1- og 2-niveau slice-isolation, glob exemptions, valgfri `subdomain` i `spec_ref`, parentetiske model-aliaser i `CONTEXT.md` og dogfooding).
- [x] **Release Gatekeeper & Verifikation**:
  - [x] `xgauntlet check-release` rapporterer fuld release-paritet og 100% ADR dækning for `0.13.2`.

## 🚫 Must NOT
- Må IKKE udføre remote publication (`git push`) jf. ADR 0003.
- Må IKKE efterlade manifest-uoverensstemmelser mellem Cargo, npm og .agents plugins.

## 📝 Revisions
- 2026-11-08: Oprettet release-opgave for v0.13.2.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo run -p xgauntlet-cli -- check-release`
