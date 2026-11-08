---
type: Task Package
title: "Task 065: Release v0.13.0 Preparation & Manifest Synchronization"
description: "Klargøre og forberede udgivelse af version 0.13.0 med opdaterede manifester, roteret CHANGELOG.md og forseglet evidens."
status: active
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-08T12:22:00Z" }
tags: [release, semver, changelog, manifests, attestation]
scope:
  - "CHANGELOG.md"
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
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "tasks/058-release-manifest-parity-and-global-distribution-clarity.md"
  - "tasks/059-architecture-governance-docs-and-scaffold-template.md"
  - "tasks/065-release-v0-13-0.md"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
---

# Task 065: Release v0.13.0 Preparation & Manifest Synchronization

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Forberede udgivelsen af xGauntlet version `0.13.0` ved at dokumentere alle fuldførte opgaver (Task 060-064) i `CHANGELOG.md`, synkronisere samtlige versionsmanifester (Cargo, npm og agent-plugins), køre release-gatekeeperen (`xgauntlet check-release`) og forsegle evidens for den nye udgivelse.

## 📋 Acceptance Criteria
- [x] Opdatere `CHANGELOG.md` med udførlige release notes for `0.13.0` under Keep a Changelog standarden.
- [x] Synkronisere alle versionsmanifester til `0.13.0` i `Cargo.toml`, `package.json`, `.agents/plugins/xgauntlet/plugin.json` og `policy.json`.
- [x] Køre `xgauntlet check-release` og bekræfte at alle manifester er i harmoni, og ADR coverage er 100%.
- [x] Køre fuld verifikation `xgauntlet verify --task 065 --save` og bekræfte `[PASSED]`.

## 🚫 Must NOT
- Må IKKE bryde manifest-paritet mellem Cargo, npm og `.agents` plugins.
- Må IKKE udføre remote `git push` uden eksplicit bruger-handling jf. ADR 0003.

## 📝 Revisions
- 2026-11-08: Oprettet release-opgave for v0.13.0.

## 🧪 Verifikation
- `target/debug/xgauntlet check-release`
- `target/debug/xgauntlet verify --task 065 --save`
