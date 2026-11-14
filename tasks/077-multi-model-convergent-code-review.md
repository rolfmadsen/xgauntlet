---
type: Task Package
title: "Task 077: Multi-Model Convergent Code Review Protocol"
description: "Styrke xGauntlets review-skill med cross-model separation, struktureret JSON-fundskema (inkl. boundary_owner), hårdt round cap (maks. 3 runder) og diff-chunking-grænse (>500 linjer)."
status: draft
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-14T11:48:00Z" }
tags: [review, multi-model, convergence, skill, anti-agreeableness, okf]
scope:
  - "tasks/077-multi-model-convergent-code-review.md"
  - ".agents/plugins/xgauntlet/skills/review/SKILL.md"
  - ".agents/plugins/xgauntlet/skills/spec/SKILL.md"
  - ".agents/AGENTS.md"
  - "CONTEXT.md"
  - "crates/xgauntlet-core/tests/plugin_review_skill_test.rs"
test_refs:
  - "crates/xgauntlet-core/tests/plugin_review_skill_test.rs"
---

# Task 077: Multi-Model Convergent Code Review Protocol

**Status**: `DRAFT`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Opgradere xGauntlets kode- og arkitekturreview (Fase 5: `review`) med de mest effektive konvergensmønstre fra [Multi-Model AI Code Review: Convergence Loops and Automated Quality Assurance](https://zylos.ai/research/2026-03-01-multi-model-ai-code-review-convergence/), så vi eliminerer *agreeableness bias*, undgår fix-oscillation på tværs af slice-grænser og bevarer høj signal-to-noise uden at introducere dyre multi-pass ensembles:

1. **Cross-Model Separation (Modvirke Echo Chamber / Agreeableness Bias):** Eksplicit anvisning af, at `review`-rollen ikke blot kører i en frisk session uden Builder-historik, men så vidt muligt eksekveres af en anden model/provider end den, der genererede koden.
2. **Struktureret JSON-Kontrakt for Review Findings:** Definere et kanonisk JSON-skema for review-fund (`finding_id`, `severity`, `category`, `file`, `line_range`, `description`, `in_scope`, `boundary_owner`, `round_number`), så fund kan filtreres, aggregeres og valideres maskinelt frem for kun via fritekst.
3. **Cross-Boundary Klassifikation (`boundary_owner`):** Fund, der skyldes kontrakt- eller arkitekturdrift i et andet slice/lag (`current | core | adapter | external`), logges eksplicit, men ekskluderes fra den aktive opgaves konvergenskrav for at forhindre lokale workarounds og oscillation.
4. **Hårdt Round Cap & Eksplicit Afslutning:** Indføre et hårdt loft på maks. **3 runder** (Runde 1: Discovery + op til 2 lukkede verifikationsrunder med frossen målstreg). Ved manglende konvergens efter Runde 3 eskaleres obligatorisk til menneskelig afgrelse.
5. **Diff-størrelsesgrænse (>500 linjer):** Tilføje en operationel regel i `spec` og `review` om, at diffs over ~500 linjer enten skal opdeles i mindre tasks eller gennemgås slice-for-slice.

## 📋 Acceptance Criteria
- [ ] **AC1 (`review`-skill opdatering):**
  - `.agents/plugins/xgauntlet/skills/review/SKILL.md` dokumenterer:
    - Cross-model separation som anbefalet standard ved `🏁 SESSION HANDOFF` til review.
    - Det strukturerede JSON-skema for review-fund inkl. `in_scope` og `boundary_owner`.
    - Hårdt round cap på maks. 3 runder før obligatorisk human escalation.
    - Diff-chunking-grænse ved >500 linjer.
- [ ] **AC2 (`spec` & `AGENTS.md` alignering):**
  - `.agents/plugins/xgauntlet/skills/spec/SKILL.md` og `.agents/AGENTS.md` afspejler diff-størrelsesvejledning og cross-model handoff-anbefaling i `🏁 SESSION HANDOFF`.
- [ ] **AC3 (Embedded Plugin Verifikation):**
  - Da `crates/xgauntlet-core/src/features/plugin/bundle.rs` embedder `.agents/plugins/xgauntlet/skills/review/SKILL.md` direkte via `include_str!`, verificeres det opdaterede indhold via en dedikeret test i `crates/xgauntlet-core/tests/plugin_review_skill_test.rs`.
- [ ] **AC4 (Spec & Gauntlet Verifikation):**
  - `xgauntlet check-spec -t 077` og `cargo test --workspace` passerer uden fejl.

## 🚫 Must NOT
- Må IKKE introducere 8-pass parallelle ensembles eller tunge multi-LLM daemoner i den lokale TDD-loop (skal bevare Zero-Daemon og lav latenstid).
- Må IKKE svække eller erstatte "Mekanisk Først-Princip" (`xgauntlet verify`, `xgauntlet check-architecture`, `cargo-mutants`).
- Må IKKE ændre `review`-skillens *Definition of Enough* 3-kriteriers tærskel (Reelle Nedbrud, Datatab & Git-forurening, Kontraktbrud) til at tillade subjektiv stilkritik.
- Må IKKE flytte målstregen i Runde 2+ (lukket verifikation forbliver ufravigelig).

## 📝 Revisions
- 2026-11-14: Oprettet opgavepakke baseret på Zylos-artiklen (2026-03-01) om multi-model AI code review convergence; sat til `DRAFT` og renset for kunstig `specs/plugin.md`-binding afventende Spor 2 (Task 078).

## 🧪 Verifikation
- `target/debug/xgauntlet check-spec -t 077`
- `cargo test -p xgauntlet-core --test plugin_review_skill_test`
- `target/debug/xgauntlet verify --task 077`
