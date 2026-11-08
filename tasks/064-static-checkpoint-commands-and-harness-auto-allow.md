---
type: Task Package
title: "Task 064: Static Checkpoint Commands & Harness Auto-Allow"
description: "Gøre checkpoint-kommandoer i templates, skills og dokumentation 100% statiske (uden dynamiske -t flags) så AI-harnesses kan auto-godkende checkpoints via præfiks-matching."
status: active
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-08T12:08:00Z" }
tags: [scaffold, templates, skills, checkpoint, harness, auto-approval, antigravity]
scope:
  - "crates/xgauntlet-core/src/features/scaffold/templates.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - ".agents/AGENTS.md"
  - ".agents/plugins/xgauntlet/skills/release/SKILL.md"
  - ".agents/plugins/xgauntlet/skills/tdd/SKILL.md"
  - "tasks/064-static-checkpoint-commands-and-harness-auto-allow.md"
test_refs:
  - "crates/xgauntlet-core/tests/scaffold_engine_test.rs"
---

# Task 064: Static Checkpoint Commands & Harness Auto-Allow

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Fjerne dynamiske opgaveflag (`-t <id>` / `--task <id>`) fra alle checkpoint-instruktioner i scaffoldede templates, agent-guidelines og sovereign skills. Da `xgauntlet checkpoint` automatisk udleder den aktive opgave fra `tasks/`, skal kommandoen altid være 100% statisk (`xgauntlet checkpoint --phase <fase>`). Dette sikrer, at terminal-sandkasser i AI-harnesses (såsom Google Antigravity IDE og Claude Code) kan auto-godkende kommandoen permanent via præfiks-matching, så brugeren slipper for manuelle godkendelser ved hvert task-skift.

## 📋 Acceptance Criteria
- [ ] **Scaffold Templates (`templates.rs`)**: Opdatere `render_agents_md` og relaterede sektioner, så `xgauntlet checkpoint --phase <spec|red|green|refactor|done>` instrueres som standard uden variable task-numre.
- [ ] **Sovereign Skills (`tdd/SKILL.md` og `release/SKILL.md`)**: Erstatte rå `git commit`-eksempler og `-t <id>` i skills med de kanoniske, statiske `xgauntlet checkpoint --phase <fase>` kommandoer.
- [ ] **Agent Guidelines (`.agents/AGENTS.md`)**: Opdatere fase-oversigten med de statiske kommandoer.
- [ ] **CLI Help Docs (`main.rs`)**: Præcisere i `checkpoint` argument-hjælpeteksten, at `--task` er valgfri og automatisk udledes fra aktiv opgave.
- [ ] **Verifikation**: Køre `cargo test -p xgauntlet-core --test scaffold_engine_test` samt regressionstests og sikre, at alt passerer.

## 🚫 Must NOT
- Må IKKE fjerne muligheden for explicit `--task <id>` i `xgauntlet checkpoint` (skal bevares for bagudkompatibilitet og scripting).
- Må IKKE bryde eksisterende assertions i `scaffold_engine_test.rs`.
- Må IKKE foretage remote publication (`git push`) jf. ADR 0003.

## 📝 Revisions
- 2026-11-08: Oprettet opgavepakke for statiske checkpoint-kommandoer og præfiks-venlig auto-godkendelse.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test scaffold_engine_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 064`
