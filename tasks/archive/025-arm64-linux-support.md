---
type: Task Package
title: "Task 025: Native ARM64 Linux CI Pipeline and Release Distribution"
description: "Etablere native Linux ARM64 CI-verifikation og release distribution af xgauntlet-linux-arm64.tar.gz for komplet out-of-the-box cross-platform paritet"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-09-13T19:54:25Z" }
tags: [cross-platform, arm64, linux, ci, release, npm, launcher]
---

# Task 025: Native ARM64 Linux CI Pipeline and Release Distribution

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-13`

## 🎯 Formål
Etablere native Linux ARM64 CI-verifikation og release distribution af xgauntlet-linux-arm64.tar.gz for komplet out-of-the-box cross-platform paritet

## 📋 Acceptance Criteria
- [x] Gøre `getPlatformInfo` i `packages/cli/bin/xgauntlet.js` deterministisk testbar med valgfrie platform- og arch-parametre og eksportere funktionen.
- [x] Tilføje tests i `packages/cli/test/launcher.test.js`, der beviser platform-opløsning for `linux` + `arm64` (`xgauntlet-linux-arm64.tar.gz`) samt de øvrige understøttede platforme.
- [x] Tilføje `ubuntu-24.04-arm` runner til `.github/workflows/ci.yml` matrixen for native CI-test på Linux ARM64.
- [x] Tilføje `aarch64-unknown-linux-gnu` target med `ubuntu-24.04-arm` runner til `.github/workflows/release.yml` matrixen for automatisk byg og upload af `xgauntlet-linux-arm64.tar.gz`.
- [x] Verificere at samtlige tests og invariant-tjek passerer (`npm test --workspaces`, `cargo test --workspace`, `cargo clippy`, `check-spec`).

## 🚫 Must NOT
- Må IKKE bryde eksisterende arkitektur-invarianter eller API-kontrakter.
- Må IKKE foretage remote publication handlinger (`git push`).
- Må IKKE introducere baggrunds-dæmoner jf. Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-13: Task oprettet som ACTIVE for Task 025: Native ARM64 Linux CI Pipeline and Release Distribution.

## 🧪 Verifikation
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-spec -t 025-arm64-linux-support`
