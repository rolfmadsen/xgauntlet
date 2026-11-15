---
type: Task Package
title: "Task 099: Prepare Patch Release v0.16.1 (Windows Worktree Fix)"
description: "Klargør patch release v0.16.1 efter Task 098 Windows worktree verbatim-path fix med manifester, CHANGELOG.md, arkivering af Tasks 097–098, WASM-policy og streng release-gatekeeper."
status: active
intent: enhancement
feature: release
spec_ref: "specs/release.md"
generated: { by: process:antigravity-task-init, at: "2026-11-15T16:08:00Z" }
tags: [release, v0.16.1, changelog, cargo, npm, windows]
scope:
  - "tasks/099-prepare-v0-16-1-release.md"
  - "tasks/**"
  - "Cargo.toml"
  - "Cargo.lock"
  - "package.json"
  - "packages/**"
  - ".agents/plugins/xgauntlet/plugin.json"
  - ".agents/plugins/xgauntlet/policy.json"
  - "CHANGELOG.md"
  - "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "specs/**"
  - "architecture.mermaid"
  - "architecture.md"
  - "docs/architecture/**"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
---

# Task 099: Prepare Patch Release v0.16.1 (Windows Worktree Fix)

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-15`

## 🎯 Formål
Klargøre patch release **xGauntlet v0.16.1** med Windows `worktree create` verbatim-path rettelsen fra Task 098, så `git push origin main v0.16.1` udløser grøn Windows/Linux/macOS CI + GitHub Release + npmjs.

## 📋 Acceptance Criteria
- [x] **AC1 — Manifest-paritetstest opdateret til `0.16.1` (`RED`)**: `test_workspace_manifests_synchronized_to_version_0_16_1` i `release_manifest_parity_test.rs` validerer `0.16.1` (bevist rød før version bump).
- [x] **AC2 — Manifest-, WASM- & Changelog-synkronisering til `0.16.1`**: Alle manifests, `gauntlet_policy.wasm` og `CHANGELOG.md` (`[0.16.1] - 2026-11-15` med Task `098`) er synkroniserede.
- [x] **AC3 — Release Gatekeeper & Gauntlet Grøn**: `cargo test --workspace`, `cargo run -p xgauntlet-cli -- check-release` og `checkpoint --phase done` passerer uden fejl.
- [x] **AC4 — Klar til `git tag v0.16.1` & `git push`**: Clean working tree og forseglet evidens klar til brugerens `git tag` + `git push origin main v0.16.1`.

## 🚫 Must NOT
- Foretag IKKE `git push`, `npm publish` eller destruktive git resets (ADR 0003).

## 📝 Revisions
- 2026-11-15: Oprettet patch release-opgavepakke for `v0.16.1` efter Task 098.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo run -p xgauntlet-cli -- check-release`
