---
type: Task Package
title: "Task 097: Prepare Release v0.16.0 (Manifest Sync, Changelog, Archive & Local Tag)"
description: "Klargør minor release v0.16.0 (SemVer for additive features i Tasks 093–096) med manifester, CHANGELOG.md, WASM-policy, arkivering af afsluttede tasks, streng release-gatekeeper og lokalt annoteret Git tag v0.16.0 klar til brugerens git push."
status: active
intent: enhancement
feature: release
spec_ref: "specs/release.md"
generated: { by: process:antigravity-task-init, at: "2026-11-15T15:48:00Z" }
tags: [release, v0.16.0, changelog, cargo, npm, tag]
scope:
  - "tasks/097-prepare-v0-16-0-release.md"
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
  - "README.md"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
---

# Task 097: Prepare Release v0.16.0 (Manifest Sync, Changelog, Archive & Local Tag)

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-15`

## 🎯 Formål
Klargøre minor release **xGauntlet v0.16.0** (SemVer for de additive features og DX-forbedringer i Tasks `093`–`096`) så brugeren direkte kan køre `git push origin main v0.16.0` for automatisk GitHub Release & npmjs-distribution via [.github/workflows/release.yml](../.github/workflows/release.yml):

1. Opdatere manifest-paritetstesten i [release_manifest_parity_test.rs](../crates/xgauntlet-core/tests/release_manifest_parity_test.rs) til `0.16.0` (`RED`).
2. Opdatere [CHANGELOG.md](../CHANGELOG.md) med præcise release notes for Tasks `093`–`096` og køre `xgauntlet release prepare 0.16.0` (manifester, arkivering til `tasks/archive/`, WASM-rebuild, `Cargo.lock` & verifikation).
3. Validere hele workspaces test- og lint-suite samt `xgauntlet check-release`, forsegle Task 097 (`DONE`), og oprette lokalt annoteret tag `v0.16.0` med clean working tree (strengt lokalt iht. ADR 0003).

## 📋 Acceptance Criteria
- [ ] **AC1 — Manifest-paritetstest opdateret til `0.16.0` (`RED`)**: `test_workspace_manifests_synchronized_to_version_0_16_0` i `release_manifest_parity_test.rs` verificerer `0.16.0` på tværs af alle deklarerede manifests (bevist rød før version bump).
- [ ] **AC2 — Manifest-, WASM- & Changelog-synkronisering til `0.16.0`**: `Cargo.toml`, `Cargo.lock`, `package.json`, `packages/cli/package.json`, `.agents/plugins/xgauntlet/{plugin,policy}.json`, `crates/xgauntlet-core/wasm/gauntlet_policy.wasm` og `CHANGELOG.md` (`[0.16.0] - 2026-11-15` med Tasks `093`–`096`) er fuldt synkroniserede.
- [ ] **AC3 — Strict Release Gatekeeper & Gauntlet Grøn**: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo run -p xgauntlet-cli -- verify --task 097 --save` og `cargo run -p xgauntlet-cli -- check-release` passerer uden fejl.
- [ ] **AC4 — Lokalt Release-Commit & Annoteret Tag `v0.16.0`**: Lokalt release-commit på `main` og annoteret tag `v0.16.0` oprettet med 100% clean working tree (ingen remote `git push` eller `npm publish` udført).

## 🚫 Must NOT
- Foretag IKKE `git push`, `npm publish` eller andre remote publication-handlinger (ADR 0003).
- Foretag IKKE destruktive git-kommandoer (`git reset --hard`, `git clean -f`).
- Efterlad IKKE usynkroniserede versioner mellem Rust (`Cargo.toml`), npm (`packages/cli/package.json`), plugins/policy (`.agents/plugins/xgauntlet/`) og WASM-binær.

## 📝 Revisions
- 2026-11-15: Oprettet og opdateret release-opgavepakke til SemVer `v0.16.0` (Tasks 093–096) efter Gate 1-godkendelse.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo run -p xgauntlet-cli -- check-release`
- `cargo run -p xgauntlet-cli -- verify --task 097 --save`
