---
type: Task Package
title: "Task 050: Antigravity IDE 2.6.0 Support and Release"
description: "Tilpasning af adapter og policy til Antigravity IDE 2.6.0 værktøjsskema og udgivelse af version 0.10.0"
status: active
generated: { by: process:antigravity-task-init, at: "2026-11-01T12:03:00Z" }
tags: [antigravity, release, semver, adapters, ide-2.6.0]
scope:
  - "crates/xgauntlet-core/src/features/adapters/antigravity/**"
  - "crates/xgauntlet-core/tests/harness_adapters_test.rs"
  - "crates/xgauntlet-core/tests/antigravity_adapter_test.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/render.rs"
  - "crates/xgauntlet-cli/src/checkers.rs"
  - "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
  - "Cargo.toml"
  - "Cargo.lock"
  - "packages/cli/package.json"
  - "CHANGELOG.md"
  - "verification-report.json"
  - "evidence.md"
  - "tasks/050-antigravity-ide-2-6-0-support-and-release.md"
test_refs:
  - "crates/xgauntlet-core/tests/antigravity_adapter_test.rs"
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
---

# Task 050: Antigravity IDE 2.6.0 Support and Release

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-01`

## 🎯 Formål
Understøtte Google Antigravity IDE 2.6.0 opdaterede tool-skema og workspace-resolution, sikre kodeformatering, forberede release v0.10.0 og klargøre Git tag og commits til brugerens remote push.

## 📋 Acceptance Criteria
- [x] AC1: Antigravity-adapteren genkender og normaliserer nye IDE 2.6.0 værktøjskald (`read_file`, `write_file`, `edit_file`, `ask_question`, `list_plugin_accounts`, `manage_task`).
- [x] AC2: Adapteren udleder det effektive workspace via `workspacePaths`, `cwd` og sti-traversering, og håndhæver fail-safe telemetri ved uoprettede opgaver.
- [x] AC3: Hele kildekoden er formateret jf. `cargo fmt` og passerer samtlige release-invarianter i `release_engine_test`.
- [x] AC4: Manifest-filer (`Cargo.toml`, `packages/cli/package.json`), `CHANGELOG.md` og WASM-bytecode synkroniseres til `0.10.0`, og evidens forsegles i `verification-report.json` og `evidence.md`.
- [x] AC5: Opgaven forsegles som DONE med commit og release-tag `v0.10.0` klar til `git push` jf. ADR 0003.

## 🚫 Must NOT
- ALDRIG udføre remote publication (`git push`) jf. ADR 0003.
- ALDRIG udføre destruktive git handlinger (`git reset --hard` eller `git clean -f`).
- Ikke tillade format- eller versionsafvigelser mellem manifester og WASM bytecode.

## 📝 Revisions
- 2026-11-01: Oprettet opgavepakke for Antigravity IDE 2.6.0 support og v0.10.0 release.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test -p xgauntlet-core --test release_engine_test`
- `cargo fmt --check`
