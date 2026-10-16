---
type: Task Package
title: "Task 035: Mechanical Session Handoff and Deterministic Release Engine"
description: "Mekanisk generering af SESSION HANDOFF kort, automatiseret git push instruks, og deterministisk SemVer release-motor i xGauntlet CLI"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-10-16T21:32:00Z" }
tags: [handoff, release, semver, git, checkpoint, cli, adr-0003]
scope:
  - "crates/xgauntlet-core/src/features/checkpoint/**"
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-cli/src/**"
test_refs:
  - "crates/xgauntlet-core/tests/handoff_and_release_test.rs"
---

# Task 035: Mechanical Session Handoff and Deterministic Release Engine

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-16`  

## 🎯 Formål
Fjerne al afhængighed af LLM'ens hukommelse og vilkårlige fortolkning ved afslutning af opgaver og ved release-håndtering:
1. **Mekanisk Session Handoff**: Når en opgave forsegles med `xgauntlet checkpoint --phase done` (eller via `xgauntlet handoff`), udlæser værktøjet deterministisk det aktuelle git-branch, commits foran remote, og genererer det autoritative `🏁 SESSION HANDOFF` Markdown-kort med den eksakte `git push origin <branch>` instruks.
2. **Deterministisk Release Engine**: `xgauntlet release` udvides med mulighed for at udføre SemVer-bump (`patch`, `minor`, `major`), synkronisere alle projektmanifester (`Cargo.toml`, `package.json`, `pyproject.toml`), rotere `CHANGELOG.md` fra `[Unreleased]` til `[<version>] - <date>`, og udskrive de eksakte release git-kommandoer (`git tag -a vX.Y.Z` og `git push origin <branch> --tags`).

## 📋 Acceptance Criteria
- [ ] **AC1 (Session Handoff Model & Engine)**: `crates/xgauntlet-core/src/features/checkpoint/` beriges med en `SessionHandoff` model, der via git-inspektion udleder `branch`, `commit_oid`, `commits_ahead`, `is_clean`, `next_role` og genererer det færdige Markdown-kort.
- [ ] **AC2 (Automatisk Handoff ved Checkpoint Done)**: Når `xgauntlet checkpoint --phase done` eksekveres, indeholder resultatet automatisk `SessionHandoff`, og CLI'en printer det fulde `🏁 SESSION HANDOFF` kort i terminalen.
- [ ] **AC3 (Dedikeret CLI Subcommand `xgauntlet handoff`)**: `xgauntlet handoff` CLI-kommandoen kan til enhver tid kaldes for at generere og printe handoff-kortet for den aktive eller senest afsluttede opgave.
- [ ] **AC4 (Deterministisk SemVer & Manifest Bumping)**: `crates/xgauntlet-core/src/features/release/` beriges med funktioner til at beregne SemVer-bump (`bump_semver`), opdatere deklarerede versioner i manifester (`update_manifest_versions`) og opdatere `CHANGELOG.md` (`prepare_changelog_release`).
- [ ] **AC5 (CLI Subcommand `xgauntlet release`)**: `xgauntlet release prepare <version>` / `xgauntlet release bump --patch|--minor|--major` opdaterer manifester og changelog deterministisk, og printer de autoritative git commit & tag kommandoer til brugeren.
- [ ] **AC6 (Regression & Verification Gauntlet)**: `handoff_and_release_test.rs` og hele workspacet passerer uden fejl, og koden er 100% `cargo clippy` ren.

## 🚫 Must NOT
- Må IKKE udføre remote `git push` eller `git push --tags` automatisk (bryder ADR 0003 - alle remote handlinger forbliver brugerens eksplicitte suverænitet).
- Må IKKE antage et fast branch-navn (skal dynamisk inspicere HEAD).
- Må IKKE tillade udstedelse af releases hvis manifester er i uoverensstemmelse eller hvis der er uafklarede release-blokkere.

## 📝 Revisions
- 2026-10-16: Task 035 oprettet for at mekanisere session handoff og release bump.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test handoff_and_release_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 035-mechanical-session-handoff-and-release-engine`
- `cargo run -p xgauntlet-cli -- handoff`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
