---
type: Task Package
title: "Task 080: Ultralight Bugfix Workflow & Antigravity Plan-Mode Hook Support"
description: "Reducer ceremoni og ventetid ved små bugfixes (scope <= 3 filer) via ét-trins bug-task scaffolding ('xgauntlet task new --intent bug') og hurtig scoped verifikation, samt whitelist Antigravitys plan-mode værktøj ('exit_plan_mode') i adapteren."
status: draft
intent: enhancement
slice: checkpoint
spec_ref: "specs/checkpoint.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T15:16:00Z" }
tags: [bugfix, fast-track, dx, tdd, checkpoint, workflow, antigravity]
scope:
  - "specs/checkpoint.md"
  - "tasks/080-ultralight-bugfix-workflow-and-hook-ux.md"
  - "crates/xgauntlet-core/src/features/tasks/lifecycle.rs"
  - "crates/xgauntlet-core/src/features/checkpoint/engine.rs"
  - "crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
  - "crates/xgauntlet-core/tests/harness_adapters_test.rs"
  - ".agents/plugins/xgauntlet/skills/tdd/SKILL.md"
  - ".agents/plugins/xgauntlet/skills/diagnose/SKILL.md"
test_refs:
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
  - "crates/xgauntlet-core/tests/harness_adapters_test.rs"
---

# Task 080: Ultralight Bugfix Workflow & Antigravity Plan-Mode Hook Support

**Status**: `DRAFT`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Gøre det markant hurtigere at rette små, afgrænsede fejl (`intent: bug`, `scope.len() <= 3`) i hverdagen, så man ikke skal bruge unødig SDLC-specifikationsceremoni på fejlrettelser af få linjer, samt fjerne en observeret friktion i Google Antigravity-adapteren:

1. **Lyn-oprettelse af Bugfix-task (`xgauntlet task new <slug> --intent bug --scope <file> --test <test_file>`):**
   - Automatisk generering af en kompakt, klar-til-brug `ACTIVE` bugfix-task i én CLI-kommando med udfyldt standard-struktur (`Formål`, `Acceptance Criteria`, `Must NOT`, `scope`, `test_refs`), så agenten ikke skal håndskrive 40 linjers boilerplate før fejlrettelsen.
   - For afgrænsede bugfixes (`scope.len() <= 3`) kræves der ikke oprettelse af nye `specs/<slice>.md`-filer, hvis de ikke allerede findes (en bugfix genskaber eksisterende forventet adfærd frem for at udvide specifikationen).
2. **Hurtigt 2-trins `RED -> DONE` gennemløb:**
   - Tydelig, friktionsfri vejledning i `diagnose` og `tdd` skills:
     1. Scaffold bug-task i ét kald $\to$ skriv reproducerende test $\to$ `xgauntlet checkpoint -p red` (kører kun den scopede `test_refs`-test).
     2. Ret fejlen i `scope` $\to$ `xgauntlet checkpoint -p done` (verificerer og lukker tasken).
3. **Antigravity `exit_plan_mode` Hook-understøttelse:**
   - Tilføj `"exit_plan_mode"` til read/control-whitelisten i `crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs`, så Orchestrator-modeller ikke blokeres fail-closed med `ToolActionType::Other`, når de afslutter planlægningsfasen.
   - Opret den levende slice-specifikation `specs/checkpoint.md` som del af opgaven (hvis ikke allerede oprettet).

## 📋 Acceptance Criteria
- [ ] **AC1 (Kompakt CLI One-Liner til Bugfixes):**
  - `xgauntlet task new <slug> --intent bug --scope <path> --test <test_path>` opretter en gyldig, `ACTIVE` bugfix-task i ét skridt, som straks består `check-spec`.
- [ ] **AC2 (Ingen Slice-Spec Blokering på Små Bugfixes):**
  - Bugfix-tasks med `intent: bug` og `scope.len() <= 3` blokeres ikke af `MISSING_TASK_SLICE_MAPPING` / `MISSING_SLICE_SPEC_FILE`, når de blot udbedrer en defekt i eksisterende kode.
- [ ] **AC3 (Antigravity `exit_plan_mode` Adapter Support):**
  - `AntigravityAdapter::normalize_tool_call` klassificerer `"exit_plan_mode"` som sikker ikke-muterende kontrolhandling i stedet for `ToolActionType::Other` (verificeret i `harness_adapters_test.rs`).
- [ ] **AC4 (Opdateret `diagnose` & `tdd` Skill Flow):**
  - `.agents/plugins/xgauntlet/skills/diagnose/SKILL.md` og `tdd/SKILL.md` dokumenterer det hurtige bugfix-flow (`task new --intent bug` $\to$ `checkpoint -p red` $\to$ fix $\to$ `checkpoint -p done`).

## 🚫 Must NOT
- Må IKKE tillade bugfixes uden en reproduceret rød test (`checkpoint -p red`).
- Må IKKE tillade Fast-Track for opgaver med `scope.len() > 3` eller opgaver, der ændrer `docs/adr/` eller `gauntlet.toml`.
- Må IKKE lempe fail-closed politikken for ukendte muterende eller shell-eksekverende værktøjer.

## 📝 Revisions
- 2026-11-14: Oprettet opgavepakke for at reducere ceremoni-overhead ved daglig bugfixing samt fjerne `exit_plan_mode`-blokering i Antigravity-adapteren.

## 🧪 Verifikation
- `target/debug/xgauntlet check-spec -t 080`
- `cargo test -p xgauntlet-core --test fast_track_and_scoped_checkpoint_test`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `target/debug/xgauntlet verify --task 080`

