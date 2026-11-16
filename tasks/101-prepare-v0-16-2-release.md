---
type: Task Package
title: "Task 101: Prepare Patch Release v0.16.2 (Auto-Unbind & Hook Deadlock Fix)"
description: "Klargør patch release v0.16.2 efter Task 100 auto-unbind af DONE active_task og deadlock-fri PreToolUse-policy med manifester, CHANGELOG.md, arkivering af Tasks 099–100, WASM-policy og streng release-gatekeeper."
status: active
intent: enhancement
feature: release
spec_ref: "specs/release.md"
generated: { by: process:antigravity-task-init, at: "2026-11-16T07:45:00Z" }
tags: [release, v0.16.2, changelog, cargo, npm, wasm, hooks]
scope:
  - "tasks/101-prepare-v0-16-2-release.md"
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

# Task 101: Prepare Patch Release v0.16.2 (Auto-Unbind & Hook Deadlock Fix)

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-16`

## 🎯 Formål
Klargøre patch release **xGauntlet v0.16.2** med auto-unbind af afsluttet `.xgauntlet/active_task`, stale-binding recovery-diagnostik og deadlock-fri `PreToolUse`-policy fra Task 100, så `git push origin main v0.16.2` udløser GitHub Release og npmjs-publicering.

## 📋 Acceptance Criteria
- [ ] **AC1 — Manifest-paritetstest opdateret til `0.16.2` (`RED`)**: `test_workspace_manifests_synchronized_to_version_0_16_2` i `release_manifest_parity_test.rs` validerer `0.16.2` (bevist rød før version bump).
- [ ] **AC2 — Manifest-, WASM- & Changelog-synkronisering til `0.16.2`**: Alle manifests, `gauntlet_policy.wasm`, `Cargo.lock` og `CHANGELOG.md` (`[0.16.2] - 2026-11-16` med Task `100`) er synkroniserede, og afsluttede Tasks `099`–`100` er arkiveret i `tasks/archive/`.
- [ ] **AC3 — Release Gatekeeper & Gauntlet Grøn**: `cargo test --workspace`, `cargo run -p xgauntlet-cli -- check-release` og `checkpoint --phase done` passerer uden fejl.
- [ ] **AC4 — Klar til `git push origin main v0.16.2`**: Clean working tree, lokal annoteret tag `v0.16.2` og forseglet evidens klar til brugerens `git push` til GitHub og npmjs.

## 🚫 Must NOT
- Foretag IKKE `git push`, `npm publish` eller destruktive git resets (ADR 0003).

## 📝 Revisions
- 2026-11-16: Oprettet patch release-opgavepakke for `v0.16.2` efter Task 100.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo run -p xgauntlet-cli -- check-release`
