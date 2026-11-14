---
type: Task Package
title: "Task 068: Release v0.13.1 Preparation & Manifest Bump"
description: "Opgradere samtlige versionsmanifester til 0.13.1, genkompilere WebAssembly policy bytecode, rotere CHANGELOG.md og forsegle release-readiness."
status: done
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-08T13:00:00Z" }
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
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "tasks/068-release-v0-13-1.md"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
---

# Task 068: Release v0.13.1 Preparation & Manifest Bump

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Forberede udgivelse af version `0.13.1` (patch bump) for at muliggøre publicering til NPM Registry, synkronisere samtlige 8 manifester, genkompilere WebAssembly policy engine til `0.13.1`, opdatere `CHANGELOG.md` og validere 100% release readiness via `xgauntlet check-release`.

## 📋 Acceptance Criteria
- [x] **Acceptance Test for Version 0.13.1**:
  - [x] `release_manifest_parity_test.rs` opdateres til at kræve version `0.13.1` på tværs af alle manifester.
- [x] **Manifest-opgradering til 0.13.1**:
  - [x] Opdatere alle 8 manifester til `0.13.1` (`Cargo.toml`, `package.json`, `packages/cli/package.json`, `plugin.json`, `policy.json` mv.).
- [x] **WASM Policy Bytecode Rebuild**:
  - [x] Genkompilere `gauntlet-policy-engine` til version `0.13.1` og opdatere `crates/xgauntlet-core/wasm/gauntlet_policy.wasm`.
- [x] **Changelog Rotation**:
  - [x] `CHANGELOG.md` opdateres med `[0.13.1] - 2026-11-08` dækkende Task 066 (`architecture.mermaid` sync) og Task 067 (NPM README paritet).
- [x] **Release Gatekeeper & Verifikation**:
  - [x] `xgauntlet check-release` rapporterer fuld release-paritet og 100% ADR dækning for `0.13.1`.
  - [x] Fuld gauntlet-verifikation `xgauntlet verify --task 068 --save` passerer.

## 🚫 Must NOT
- Må IKKE udføre remote publication (`git push`) jf. ADR 0003.
- Må IKKE efterlade manifest-uoverensstemmelser mellem Cargo, npm og .agents plugins.

## 📝 Revisions
- 2026-11-08: Oprettet release-opgave for v0.13.1.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo run -p xgauntlet-cli -- check-release`
