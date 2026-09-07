---
type: Task Package
title: "Task 012: Dynamic Cockpit HUD & TDD Checkpoint Protocol"
description: "Etablere normative retningslinjer og scaffold-skabeloner for det udvidede Response HUD samt lokal fase-bunden TDD commit-disciplin og idé-fase intent guide jf. ADR 0003"
status: completed
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T22:30:00Z" }
tags: [hud, tdd, git-checkpoints, intent, guidelines, scaffold, adr-0003]
---

# Task 012: Dynamic Cockpit HUD & TDD Checkpoint Protocol

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere de normative retningslinjer og scaffold-skabeloner for det udvidede Response HUD, den lokale fase-bundne TDD commit-disciplin samt idé-fase intent vejledning i `xGauntlet`:
1. **Dynamisk Cockpit Response HUD**:
   - Forbedre den standardiserede Response HUD i `.agents/AGENTS.md` fra et overvejende statisk badge til et levende cockpit.
   - HUD skal inkludere operationel telemetri:
     - Git context: Aktuel branch, HEAD-commit OID og antal dirty/staged filer (`Git: branch@hash • Dirty: N files`).
     - Kriterie-fremdrift: Akkumuleret acceptkriterie status fra aktiv task (`Criteria: X/Y [■■□□□]`).
     - Scope Boundary: Eksplicit angivelse af berørte moduler eller filer (`Scope: crates/xgauntlet-core/...`).
     - Næste handling: Én linje med agentens umiddelbare næste handling (`💡 Next Action: ...`).
     - Bevarelse af de centrale links: Task, Spec, Glossary, ADR og Evidence.
2. **Lokal TDD Phase Checkpoint Protokol**:
   - Definere en eksplicit, atomisk lokal git commit-protokol bundet til faserne i agentens TDD-løkke jf. [ADR 0003](docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md):
     - `SPEC`: `task(<id>): initialize task specification and criteria`
     - `RED`: `test(<id>): add failing acceptance test for <feature> [RED]`
     - `GREEN`: `feat(<id>): implement minimal logic to satisfy test [GREEN]`
     - `REFACTOR`: `refactor(<id>): clean up module boundaries and types [REFACTOR]`
     - `DONE`: `chore(<id>): seal evidence and mark task DONE`
   - Præcisere at agenten aktivt udfører `git add` og `git commit` lokalt for at bevare detaljeret historik, men **aldrig** foretager `git push` jf. ADR 0003.
3. **Intent-to-Task Sparringsprocedure**:
   - Dokumentere en struktureret 4-trins metode til idéfasen, hvor agenten fungerer som sparringspartner for ustrukturerede brugerønsker (Formål, Invarianter/Must NOT, RED test-hypotese og ADR-triggere) før opgaven formuleres.
4. **Scaffold Template Paritet**:
   - Opdatere `crates/xgauntlet-core/src/features/scaffold/templates.rs` (`render_agents_md`), så nye projekter initialiseret med `xgauntlet init` automatisk modtager disse retningslinjer.

## 📋 Acceptance Criteria
- [x] `.agents/AGENTS.md` indeholder den udvidede dynamiske Cockpit Response HUD specifikation (Git context, Criteria progress bar, Scope indicator og Next Action linje).
- [x] `.agents/AGENTS.md` indeholder en formaliseret lokal TDD Phase Checkpoint protokol med faste Conventional Commit præfikser for SPEC, RED, GREEN, REFACTOR og DONE jf. ADR 0003.
- [x] `.agents/AGENTS.md` indeholder en trinvis sparringsprocedure for etablering af brugerens intent i idéfasen før oprettelse af taskfiler.
- [x] `crates/xgauntlet-core/src/features/scaffold/templates.rs` (`render_agents_md`) afspejler 100% de nye Cockpit HUD og checkpoint retningslinjer.
- [x] `crates/xgauntlet-core/tests/scaffold_test.rs` eller tilsvarende test verificerer at den genererede `AGENTS.md` indeholder Cockpit HUD og fase-checkpoint specifikationen.
- [x] `cargo run -p xgauntlet-cli -- check-spec -t 012-dynamic-cockpit-hud-and-tdd-checkpoint-protocol` validerer med 0 fejl.
- [x] Samtlige eksisterende tests i `xgauntlet` forbliver 100% grønne (`cargo test --workspace`).

## 🚫 Must NOT
- Må IKKE tillade eller instruere i remote git publication handlinger (`git push`), `git reset --hard` eller `git clean -f` jf. ADR 0003.
- Må IKKE fjerne de eksisterende navigationslinks (Task, Spec, Glossary, ADR, Evidence) fra Response HUD specifikationen.
- Må IKKE gøre Cockpit HUD så voluminøs at den overstiger 5 linjer (skal bevare høj informationstæthed og lavt token-overhead).
- Må IKKE introducere baggrundsdæmoner eller runtime sockets jf. Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-06: Task oprettet som ACTIVE for Dynamic Cockpit HUD & TDD Checkpoint Protocol (Task 012).

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 012-dynamic-cockpit-hud-and-tdd-checkpoint-protocol`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `git status --porcelain` (verificere ren arbejdsgren efter test)
