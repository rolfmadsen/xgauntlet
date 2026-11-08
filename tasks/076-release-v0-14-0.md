---
type: Task Package
title: "Task 076: Release v0.14.0 Preparation & Manifest Bump"
description: "Opgradere samtlige versionsmanifester til v0.14.0, genkompilere WebAssembly policy bytecode, rotere CHANGELOG.md for Tasks 071-075 og forsegle release-readiness via xgauntlet check-release."
status: active
intent: enhancement
slice: release
spec_ref: "specs/release.md"
generated: { by: process:antigravity-task-init, at: "2026-11-08T22:30:00Z" }
tags: [release, semver, bump, minor, changelog, wasm, manifests, mermaid, topology]
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
  - "tasks/076-release-v0-14-0.md"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
---

# Task 076: Release v0.14.0 Preparation & Manifest Bump

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Forberede udgivelse af version `0.14.0` (Minor semver bump på `0.13.2`) for at publicere de nye AST topology extraction, trait realization og Mermaid domænemodellerings-kapabiliteter:
1. **Manifest Parity**: Opdatere samtlige 8 versionsmanifester (`Cargo.toml` root & crates, `package.json`, `packages/cli/package.json`, `.agents/plugins/xgauntlet/plugin.json`, `.agents/plugins/xgauntlet/policy.json`) til `0.14.0`.
2. **WASM Policy Engine**: Genkompilere `gauntlet-policy-engine` til version `0.14.0` og verificere at embedded `gauntlet_policy.wasm` matcher versionen.
3. **Changelog Rotation**: Rotere `CHANGELOG.md` under `## [0.14.0] - 2026-11-08` dækkende Tasks 071-075.
4. **Release Gatekeeper & Evidens**: Validere 100% release readiness via `xgauntlet check-release` og forsegle evidens.

## 📋 Acceptance Criteria
- [ ] **Acceptance Test for Version 0.14.0 (RED phase)**:
  - [ ] `crates/xgauntlet-core/tests/release_manifest_parity_test.rs` opdateres til at kræve version `0.14.0` på tværs af alle manifester og bevises rød før ændringer.
- [ ] **Manifest-opgradering til 0.14.0**:
  - [ ] Opdatere alle 8 manifester til `0.14.0`.
- [ ] **WASM Policy Bytecode Rebuild**:
  - [ ] Genkompilere `gauntlet-policy-engine` til version `0.14.0` og opdatere `crates/xgauntlet-core/wasm/gauntlet_policy.wasm`.
- [ ] **Changelog Rotation**:
  - [ ] `CHANGELOG.md` opdateres med `[0.14.0] - 2026-11-08` dækkende Tasks 071-075.
- [ ] **Release Gatekeeper & Verifikation**:
  - [ ] `xgauntlet check-release` rapporterer fuld release-paritet og 100% ADR dækning for `0.14.0`.
  - [ ] `target/debug/xgauntlet verify --task 076 --save` passerer med `Self-mutation status: CLEAN`.

## 🚫 Must NOT
- Må IKKE udføre remote publication (`git push`) jf. ADR 0003.
- Må IKKE efterlade manifest-uoverensstemmelser mellem Cargo, npm og .agents plugins.

## 📝 Revisions
- 2026-11-08: Oprettet release-opgave for v0.14.0.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `target/debug/xgauntlet check-release`
- `target/debug/xgauntlet verify --task 076 --save`
