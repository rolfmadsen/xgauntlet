---
type: Task Package
title: "Task 083: Auto-Archive DONE Tasks During Release Preparation"
description: "Arkivér automatisk alle færdige (DONE/COMPLETED) tasks til tasks/archive/ og opdatér specs/**/*.md-links som led i xgauntlet release bump / release prepare før den endelige evidensforsegling."
status: active
intent: enhancement
slice: release
spec_ref: "specs/release.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T17:13:00Z" }
tags: [release, tasks, archive, dx, transactional-staging, evidence]
scope:
  - "specs/release.md"
  - "tasks/083-auto-archive-done-tasks-on-release.md"
  - "crates/xgauntlet-core/src/lib.rs"
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/render.rs"
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
  - ".agents/plugins/xgauntlet/skills/release/SKILL.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
---

# Task 083: Auto-Archive DONE Tasks During Release Preparation

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Eliminere behovet for at huske manuel `tasks/`-oprydning i hverdagen ved at integrere arkivering af alle færdige (`DONE`/`COMPLETED`) tasks direkte i release-tidspunktet (`xgauntlet release bump` og `xgauntlet release prepare`), lige før den endelige evidensforsegling og `check-release --strict`:

1. **Automatisk Arkivering før Evidensforsegling:**
   - Når `release bump` eller `release prepare` kører (uden `--dry-run` og uden `--no-archive`), kaldes `TaskScaffolder::archive_tasks` (`all_done = true`) **før** den afsluttende verifikation og forsegling af `verification-report.json` / `evidence.md`.
   - Eventuel aktiv release-task (`status: active`) eller fremtidige `DRAFT`-tasks forbliver urørt i roden af `tasks/`, mens samtlige færdige tasks fra releasen flyttes til `tasks/archive/` med opdaterede links i `specs/**/*.md`.
   - Da arkiveringen sker før evidensforsegling og `check-release --strict`, opstår der **0 source drift** mellem manifest-digest og det endelige git-tag (`git tag vX.Y.Z` før `git push`).
2. **Transaktionel Sikkerhed & `--dry-run` / `--no-archive`:**
   - Ved `--dry-run` køres `archive_tasks` kun i `dry_run = true` og viser hvilke `DONE`-tasks og `specs/*.md`-filer der ville blive arkiveret/opdateret, uden at flytte filer på disk.
   - Hvis release-forberedelsen fejler under `check-release --strict` eller manifest-opdatering, rulles både flyttede task-filer og opdaterede `specs/**/*.md` rent tilbage via den eksisterende transaktionelle backup-mekanisme (uden destruktive git-kommandoer jf. `ADR 0003`).
   - `--no-archive`-flag på `release bump` og `release prepare` gør det muligt at springe arkiveringen over i særlige tilfælde.

## 📋 Acceptance Criteria
- [ ] **AC1 (Auto-Archive i `release bump` & `release prepare`):**
  - `xgauntlet release bump <patch|minor|major>` og `xgauntlet release prepare <version>` arkiverer automatisk alle `DONE`/`COMPLETED` tasks til `tasks/archive/` (og opdaterer relative links i `specs/**/*.md`) før endelig evidensforsegling, medmindre `--no-archive` er angivet.
  - `verification-report.json` og `check-release --strict` viser **0 source drift** efter release-forberedelsen.
- [ ] **AC2 (Dry-Run & Transaktionel Rollback):**
  - `--dry-run` forhåndsviser de berørte `DONE`-tasks uden disk-mutationer.
  - Fejler efterfølgende release-gates, rulles også arkiverede task-filer og opdaterede `specs/*.md`-links tilbage via isoleret disk-staging uden `git reset --hard` eller `git clean -f`.
- [ ] **AC3 (Dokumentation & Release Skill):**
  - [specs/release.md](../specs/release.md) og [.agents/plugins/xgauntlet/skills/release/SKILL.md](../.agents/plugins/xgauntlet/skills/release/SKILL.md) dokumenterer den automatiske release-arkivering og `--no-archive`.
- [ ] **AC4 (Isolerede Release-Tests):**
  - `crates/xgauntlet-core/tests/release_engine_test.rs` dækker automatisk arkivering ved `release prepare`/`bump`, `--no-archive`, `--dry-run`, bevaring af `ACTIVE`/`DRAFT`-tasks og 0 source drift.

## 🚫 Must NOT
- Må IKKE arkivere `ACTIVE`, `DRAFT` eller ikke-`DONE` tasks (herunder en evt. igangværende release-task).
- Må IKKE introducere `Source Drift` mellem `verification-report.json` og workspace-manifestet efter `release bump`/`prepare`.
- Må IKKE udføre `git push`, `git reset --hard` eller `git clean -f` ved fejl eller rollback.
- Må IKKE bryde manuel `xgauntlet task archive --all-done` mellem releases.

## 📝 Revisions
- 2026-11-14: Oprettet `DRAFT`-opgavepakke efter afklaring af at release-tidspunktet (`release bump` / `release prepare` før evidensforsegling & tag) er det ideelle tidspunkt til automatisk oprydning af `DONE`-tasks.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec --context -t 083`
- `cargo test -p xgauntlet-core --test release_engine_test`
- `cargo run -p xgauntlet-cli -- verify --task 083`

