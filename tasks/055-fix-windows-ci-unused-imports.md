---
type: Task Package
title: "Task 055: Fix Windows CI Unused Imports in Parity Test Suite (v0.11.3)"
description: "Retter Windows CI-fejl i v0.11.2 ved ubenyttede imports i cross_platform_ci_parity_test.rs under Clippy -D warnings ved at afgrænse Unix-specifikke symboler og håndhæve mekanisk kildekode-audit lokalt på Linux."
status: done
intent: bug
generated: { by: process:antigravity-task-init, at: "2026-11-02T06:58:00Z" }
tags: [bugfix, windows, ci-parity, clippy, cross-platform, release]
scope:
  - "crates/xgauntlet-core/**"
  - "CHANGELOG.md"
  - "Cargo.toml"
  - "Cargo.lock"
  - "package.json"
  - "packages/cli/package.json"
  - "verification-report.json"
  - "evidence.md"
  - "tasks/055-fix-windows-ci-unused-imports.md"
test_refs:
  - "crates/xgauntlet-core/tests/cross_platform_ci_parity_test.rs"
---

# Task 055: Fix Windows CI Unused Imports in Parity Test Suite (v0.11.3)

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-02`

---

## 🎯 Formål

GitHub Actions CI jobbet `Run Full CI Gauntlet / Test (windows-latest)` fejlede for release `v0.11.2` under `cargo clippy --workspace --all-targets -- -D warnings` med 6 ubenyttede imports i `crates/xgauntlet-core/tests/cross_platform_ci_parity_test.rs`:
- `serde_json::json`
- `xgauntlet_core::features::adapters::antigravity::AntigravityAdapter`
- `xgauntlet_core::features::adapters::models::HarnessAdapter`
- `xgauntlet_core::features::checkpoint::engine::run_checkpoint`
- `CheckpointOptions` og `CheckpointPhase`
- `xgauntlet_core::features::policy::WorkspaceRelativePath`

Disse imports blev udelukkende brugt i `test_macos_style_symlinked_temp_and_workspace_parity()`, som er betinget af `#[cfg(unix)]`. På Windows (`not(unix)`) kompileres testfunktionen ikke, hvorfor top-level imports bliver ubrugte og fejler under `-D warnings`.

Formålet er:
1. Afgrænse disse imports til Unix (`#[cfg(unix)]` eller funktions-internt), så Windows-mål kompilerer fejlfrit under `clippy --all-targets -- -D warnings`.
2. Udvide den lokale mekaniske kildekode-audit i `test_codebase_free_of_collision_prone_temp_paths_and_unix_only_env` med Invariant 4, der lokalt på Linux automatisk opdager testfiler med `#[cfg(unix)]`, hvor top-level imports kun refereres i Unix-blokke.
3. Forberede release `v0.11.3`.

---

## 📋 Acceptance Criteria

- [x] **AC1 — Scope Unix-specifikke imports til Unix:**
  - `crates/xgauntlet-core/tests/cross_platform_ci_parity_test.rs` flytter de 6 imports (`json`, `AntigravityAdapter`, `HarnessAdapter`, `run_checkpoint`, `CheckpointOptions`/`CheckpointPhase`, `WorkspaceRelativePath`) ind i `test_macos_style_symlinked_temp_and_workspace_parity` eller annoterer dem med `#[cfg(unix)]`.
- [x] **AC2 — Mekanisk Invariant 4 i CI Parity Suiten:**
  - `test_codebase_free_of_collision_prone_temp_paths_and_unix_only_env` i `cross_platform_ci_parity_test.rs` udvides med Invariant 4, der scanner testfiler for `#[cfg(unix)]` og verificerer, at imports, som kun benyttes i Unix-gatede funktioner, ikke deklareres unguarded på filniveau.
- [x] **AC3 — Workspace Verifikation & Release `v0.11.3`:**
  - `cargo clippy --workspace --all-targets -- -D warnings` passerer rent lokalt og på tværs af targets.
  - Bump version til `v0.11.3`, forsegl evidens i `evidence.md` og opdater `CHANGELOG.md`.

---

## 🚫 Must NOT

- **Må IKKE** anvende `#[allow(unused_imports)]` til at maskere advarsler (jf. Clean Craftsmanship & Universal Quality & Safety).
- **Må IKKE** foretage `git push`, slette remote branches eller tags (ADR 0003).
- **Må IKKE** slække på eksisterende assertions i paritetssuiten.

---

## 📝 Revisions

- 2026-11-02: Oprettet Task 055 efter Windows CI clippy fejl på v0.11.2 tagget.

---

## 🧪 Verifikation

- `cargo test -p xgauntlet-core --test cross_platform_ci_parity_test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `./target/release/xgauntlet verify --save -t 055`
