---
type: Task Package
title: "Task 092: Clean Cargo.toml, Add Dependabot & Prepare Release v0.15.1"
description: "Fjerner ubrugte workspace-felter i Cargo.toml (ed25519-dalek og authors), tilføjer .github/dependabot.yml for automatiske afhængighedsadvarsler og udfører xgauntlet release prepare 0.15.1 med lokalt commit og annoteret tag v0.15.1."
status: active
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-14T23:06:00Z" }
tags: [release, v0.15.1, cargo, dependabot, maintenance, tag]
scope:
  - "tasks/092-prepare-v0-15-1-release.md"
  - "tasks/**"
  - "Cargo.toml"
  - "Cargo.lock"
  - "package.json"
  - "packages/**"
  - ".github/dependabot.yml"
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

# Task 092: Clean Cargo.toml, Add Dependabot & Prepare Release v0.15.1

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

---

## 🎯 Formål

Forberede patch release **xGauntlet v0.15.1** med Windows CLI-rettelsen fra Task 091 samt oprydning og afhængighedsovervågning:

1. Fjerne de to ubrugte workspace-felter i [Cargo.toml](../Cargo.toml) (`workspace.package.authors` og `workspace.dependencies.ed25519-dalek`) så `cargo 1.100.0` bygger helt uden manifest-advarsler.
2. Oprette [.github/dependabot.yml](../.github/dependabot.yml) for automatiske PR/advarsler ved forældede Cargo-, npm- og GitHub Actions-afhængigheder.
3. Opdatere manifest-paritetstesten i [crates/xgauntlet-core/tests/release_manifest_parity_test.rs](../crates/xgauntlet-core/tests/release_manifest_parity_test.rs) til `0.15.1` (`RED`), afvikle `xgauntlet release prepare 0.15.1`, oprette lokalt release-commit og annoteret tag `v0.15.1` (strengt lokalt iht. ADR 0003).

---

## 📋 Acceptance Criteria

- [x] **AC1 — Manifest-paritetstest opdateret til 0.15.1:** `test_workspace_manifests_synchronized_to_version_0_15_1` i `crates/xgauntlet-core/tests/release_manifest_parity_test.rs` validerer alle deklarerede manifests imod `0.15.1` (RED bevist før version bump).
- [x] **AC2 — Ren Cargo.toml & Dependabot konfigureret:** `Cargo.toml` har ingen ubrugte workspace-felter/afhængigheder (0 Cargo-advarsler på `1.100.0`), og `.github/dependabot.yml` dækker `cargo`, `npm` og `github-actions`.
- [x] **AC3 — Manifest-, Changelog- & WASM-synkronisering til 0.15.1:** Alle workspace-manifests er opdateret til `0.15.1`, `CHANGELOG.md` har sektion `[0.15.1] - 2026-11-14`, og `crates/xgauntlet-core/wasm/gauntlet_policy.wasm` rapporterer `0.15.1`.
- [x] **AC4 — Release Gatekeeper & Testsuite Grøn:** `cargo test --workspace` samt `cargo run -p xgauntlet-cli -- check-release --strict` passerer med 0 fejl.
- [x] **AC5 — Lokalt Commit & Annoteret Tag `v0.15.1`:** Lokalt release-commit og annoteret Git tag `v0.15.1` klargjort på `main` med clean working tree (ingen remote push udført).

---

## 🚫 Must NOT

- **Må IKKE** udføre `git push` eller andre remote publication-handlinger (ADR 0003).
- **Må IKKE** udføre destruktive git-kommandoer (`git reset --hard`, `git clean -f`).
- **Må IKKE** efterlade usynkroniserede versioner mellem `Cargo.toml`, npm `packages/cli/package.json`, `.agents/plugins/xgauntlet/` og WASM-policy.

---

## 📝 Revisions

- 2026-11-14: Oprettet release-opgave for `v0.15.1` inkl. `Cargo.toml`-oprydning og `.github/dependabot.yml`.

---

## 🧪 Verifikation

- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-release --strict`
