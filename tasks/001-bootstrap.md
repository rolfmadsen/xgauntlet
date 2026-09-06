---
type: Task Package
title: "Task 001: Workspace Setup, WASM Policy Component & Cross-OS CI"
description: Etablere hybrid Cargo & NPM workspace, WASM policy engine crate med WIT kontrakt og GitHub Actions CI matrix
status: stable
generated: { by: process:agent-gauntlet-init, at: "2026-09-06T16:15:00Z" }
tags: [bootstrap, setup, wasm, cargo, npx, ci]
---

# Task 001: Workspace Setup, WASM Policy Component & Cross-OS CI

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere den grundlæggende, platformsuafhængige workspace-arkitektur for xGauntlet:
1. Konfigurere Cargo workspace med `crates/xgauntlet-core`, `crates/xgauntlet-cli` og det nye `crates/gauntlet-policy-engine`.
2. Etablere `wit/gauntlet_policy.wit` og kompilere `gauntlet-policy-engine` til deterministisk WebAssembly med Nul Ambient Authority.
3. Konfigurere NPM workspace med `@xgauntlet/cli` bootstrapper og cross-platform runner tests.
4. Oprette GitHub Actions CI workflow med testmatrix på Ubuntu, macOS og Windows.
5. Køre første grønne gauntlet på workspace (`cargo test` og `npm test`).

## 📋 Acceptance Criteria
- [x] Cargo workspace indeholder `xgauntlet-core`, `xgauntlet-cli` og `gauntlet-policy-engine`.
- [x] `wit/gauntlet_policy.wit` definerer capability request, enforcement context og policy decision jf. specifikationen.
- [x] `crates/gauntlet-policy-engine` implementerer deterministisk policy evaluation i pure Rust uden `std::io` eller systemkald og kan kompileres til WebAssembly.
- [x] `packages/cli` indeholder `@xgauntlet/cli` bootstrapper med cross-platform binær detektering og bestående node tests (`npm test`).
- [x] `.github/workflows/ci.yml` er konfigureret med test-matrix for Linux, macOS og Windows.
- [x] Første fulde workspace build og testsuite kører grønt (`npm run test && cargo test`).

## 🚫 Must NOT
- Må IKKE introducere dæmoner eller OS-specifikke service managers (`systemd`, `launchd`, Windows Service).
- `crates/gauntlet-policy-engine` må IKKE have ambient capabilities (ingen adgang til netværk, filsystem, system-ur eller tilfældighedskilder).
- Må IKKE foretage utilsigtede remote publication kommandoer (`git push`).

## 📝 Revisions
- 2026-09-06: Task opdateret og afstemt med den godkendte Zero-Daemon & Pure-WASM arkitektur.

## 🧪 Verifikation
- `cargo test --workspace`
- `npm test --workspaces`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`
