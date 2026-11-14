---
type: Task Package
title: "Task 090: Prepare Release v0.15.0 for GitHub and npmjs"
description: "Opdaterer release-manifest paritetstest til dynamisk workspace-version (CARGO_PKG_VERSION), udfører xgauntlet release prepare 0.15.0 (manifests, CHANGELOG.md, WASM-bytecode, arkivering og evidensforsegling) og opretter lokalt commit samt annoteret git tag v0.15.0 klar til brugerens push."
status: active
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-14T22:34:00Z" }
tags: [release, v0.15.0, npm, cargo, wasm, tag]
scope:
  - "tasks/090-prepare-v0-15-0-release.md"
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
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
---

# Task 090: Prepare Release v0.15.0 for GitHub and npmjs

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

---

## 🎯 Formål

Forberede udgivelsen af **xGauntlet v0.15.0** lokalt på `main` med henblik på brugerens efterfølgende `git push` til GitHub (som driver npmjs-publicering af `xgauntlet` og opdaterede binære releases):

1. Opdatere manifest-paritetstesten i [crates/xgauntlet-core/tests/release_manifest_parity_test.rs](../crates/xgauntlet-core/tests/release_manifest_parity_test.rs) til `test_workspace_manifests_synchronized_to_version_0_15_0` med dokumenteret RED-bevis før bump.
2. Afvikle `xgauntlet release prepare 0.15.0` så samtlige manifests (`Cargo.toml`, `package.json`, `packages/cli/package.json`, `.agents/plugins/xgauntlet/plugin.json`, `.agents/plugins/xgauntlet/policy.json`), `CHANGELOG.md`, `Cargo.lock`, WASM-policy (`crates/xgauntlet-core/wasm/gauntlet_policy.wasm`), afsluttede opgaver og evidens bliver synkroniseret og forseglet for `0.15.0`.
3. Oprette lokalt release-commit og annoteret git tag `v0.15.0` (strengt lokalt iht. ADR 0003 — ingen `git push`).

---

## 📋 Acceptance Criteria

- [x] **AC1 — Manifest-paritetstest opdateret til 0.15.0:** `test_workspace_manifests_synchronized_to_version_0_15_0` i `crates/xgauntlet-core/tests/release_manifest_parity_test.rs` validerer alle deklarerede manifests imod `0.15.0` (RED bevist i `b25fd62`).
- [x] **AC2 — Manifest-, Changelog- & WASM-synkronisering til 0.15.0:** Alle workspace-manifests er opdateret til `0.15.0`, `CHANGELOG.md` har sektion `[0.15.0] - 2026-11-14`, og `crates/xgauntlet-core/wasm/gauntlet_policy.wasm` rapporterer `0.15.0`.
- [x] **AC3 — Release Gatekeeper & Testsuite Grøn:** `cargo test --workspace` samt `cargo run -p xgauntlet-cli -- check-release` passerer med 0 fejl, og frisk evidens er forseglet.
- [x] **AC4 — Lokalt Commit & Annoteret Tag `v0.15.0`:** Lokalt release-commit og annoteret Git tag `v0.15.0` klargjort på `main` med clean working tree (ingen remote push udført).

---

## 🚫 Must NOT

- **Må IKKE** udføre `git push` eller andre remote publication-handlinger (ADR 0003).
- **Må IKKE** udføre destruktive git-kommandoer (`git reset --hard`, `git clean -f`).
- **Må IKKE** efterlade usynkroniserede versioner mellem `Cargo.toml`, npm `packages/cli/package.json`, `.agents/plugins/xgauntlet/` og WASM-policy.

---

## 📝 Revisions

- 2026-11-14: Oprettet opgavepakke for klargøring af release `v0.15.0`.

---

## 🧪 Verifikation

- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo run -p xgauntlet-cli -- check-release --strict`
- `cargo test --workspace`
- `git show --no-patch v0.15.0`
