---
type: Task Package
title: "Task 030: Autonomous RED to GREEN TDD Phase Transition"
description: "Eliminere overflødig human-in-the-loop gate mellem RED og GREEN faserne ved verificeret fejlet test og etablere en autonom indre TDD-løkke"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-19T13:45:00Z" }
tags: [task-lifecycle, tdd, checkpoint, autonomy, preflight, jit, human-in-the-loop, adr-0003]
---

# Task 030: Autonomous RED to GREEN TDD Phase Transition

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-09-19`  

## 🎯 Formål & Løsningsdesign
Eliminere den overflødige Human-in-the-Loop (HITL) gate mellem `RED` og `GREEN` faserne i xGauntlet-styrede agentforløb, så agenten automatisk og uden ophold fortsætter direkte fra en verificeret rød accepttest til grøn implementation.

### Hvorfor var der en gate, og hvorfor er den overflødig nu?
Historisk har naive agent-rammeværker påkrævet menneskelig bekræftelse ved hvert delskridt for at sikre, at en model ikke "løber løbsk" eller implementerer mod en forkert test. I xGauntlets arkitektur er denne manuelle turn-pause imidlertid blevet en ren friktionskilde:
1. **Mekanisk Pre-Flight Verifikation (`xgauntlet checkpoint --phase red`)**: Checkpoint-motoren eksekverer testlaget og beviser matematisk, at testen rent faktisk fejler med forventet fejl (exit code != 0).
2. **Atomiske Lokale Checkpoints (ADR 0003)**: Hver fase forsegles i et lokalt Git-commit (`test(<id>): ... [RED]`), hvilket giver et øjeblikkeligt tilbagerulningspunkt, hvis den grønne implementation skulle spore af.
3. **Frosne Seams & Multi-Layer Gauntlet**: Test-seams er allerede fastlagt og godkendt under `SPEC`-fasen, og den samlede verifikations-gauntlet (linter, types, tests, mutation, spec-check) venter uanset hvad i slutningen af opgaven.

### Arkitektonisk Opdeling: Grænse-Gates vs. Autonom Indre Løkke
Vi etablerer en skarp sondring mellem to typer faser:
- **Menneskelige Grænse-Gates (Beholdes 100%)**:
  1. *Gate 1: SPEC Godkendelse* – Mennesket godkender opgavens formål, acceptkriterier og test-seams før kode skrives.
  2. *Gate 2: AUDIT & HANDOFF Godkendelse* – Mennesket godkender kode-review, verifikationsrapport og endelig opgavelukning.
- **Autonom Indre Løkke (`[RED ➔ GREEN ➔ REFACTOR]`)**:
  - Når den røde test er skrevet og preflight bekræfter den forventede fejl, må agenten **IKKE** standse op eller spørge brugeren om lov. Agenten fortsætter straks med minimal kode for at gøre testen grøn.

---

## 📋 Acceptance Criteria
- [ ] **AC1 (Core Development Loop i Retningslinjer & Skabeloner)**:
  - Opdatere `features/scaffold/templates.rs` (`render_agents_md`) og roden `.agents/AGENTS.md` så procesdiagrammet opdateres:
    `SPEC / GRILL → (Human Approval) → [RED → GREEN → REFACTOR (Autonomous)] → GAUNTLET → (Human Audit) → EVIDENCE / DONE`
  - Tilføje eksplicit regel: Når en rød accepttest fejler som forventet og `checkpoint --phase red` er udført, må agenten IKKE standse eller afvente brugerbekræftelse; agenten skal umiddelbart overgå til GREEN fasen.
- [ ] **AC2 (JIT Telemetry Directive Opdatering i `jit.rs`)**:
  - Opdatere `PipelinePhase::ImplementationTdd` i `crates/xgauntlet-core/src/features/telemetry/jit.rs`.
  - Mål (`target`) og port (`gate`) opdateres til eksplicit at instruere AI-harnesses i autonom fremdrift mellem RED og GREEN uden pauser:
    - *Target*: `Execute autonomous TDD cycle: failing test (RED) ➔ minimal fix (GREEN) ➔ refactor. Auto-transition immediately from RED to GREEN once failure is preflight-verified.`
- [ ] **AC3 (Checkpoint Engine & CLI Next-Action Prompting)**:
  - I `crates/xgauntlet-core/src/features/checkpoint/models.rs`:
    - Udvide `CheckpointResult` med feltet `next_phase: Option<String>` og `autonomous_transition: bool`.
  - I `crates/xgauntlet-core/src/features/checkpoint/engine.rs`:
    - Ved succesfuld `CheckpointPhase::Red` sættes `next_phase = Some("green")` og `autonomous_transition = true`.
  - I `crates/xgauntlet-cli/src/main.rs`:
    - Ved fuldført `--phase red` udskriver terminalen et klart direktiv:
      `⚡ [AUTONOMOUS TDD] RED checkpoint established. Preflight confirmed test failure. Do NOT pause for user approval. Proceed immediately to GREEN phase.`
    - I `--json` output medtages `next_phase` og `autonomous_transition`.
- [ ] **AC4 (Skill Refactoring i `old-coder`)**:
  - Opdatere `plugins/xgauntlet/skills/old-coder/SKILL.md` (og den indlejrede template i `plugin/models.rs`):
    - Dokumentere "Autonomous TDD Inner Loop" princippet: Hvorfor overgang fra RED til GREEN sker i én sammenhængende eksekvering uden turn-pauser.
- [ ] **AC5 (ADR 0008 Formulering)**:
  - Oprette `docs/adr/0008-autonomous-tdd-inner-loop-and-boundary-human-gates.md` og registrere den i `docs/adr/README.md`.
- [ ] **AC6 (Testsuite & Verifikation)**:
  - Tilføje tests i `crates/xgauntlet-core/tests/checkpoint_engine_test.rs`, der verificerer at `CheckpointResult` for `--phase red` returnerer `autonomous_transition: true` og `next_phase: Some("green")`.
  - Verificere at `xgauntlet check-spec -t 030-autonomous-red-to-green-tdd-transition` validerer med 0 fejl.
  - 100% grøn testsuite på tværs af hele workspacet (`cargo test --workspace`).

---

## 🚫 Must NOT
- Må IKKE fjerne den indledende Human Gate ved `SPEC` (opgavespecifikation og test-seams skal godkendes af brugeren).
- Må IKKE fjerne den afsluttende Human Gate ved `AUDIT` / `DONE` (brugeren har altid det sidste ord ved gennemgang af evidens).
- Må IKKE tillade overgang til GREEN hvis RED pre-flight fejler (hvis testen uventet passerer, eller hvis der er uventet kompilationsfejl/syntaksfejl).
- Må IKKE foretage remote publication handlinger (`git push`) jf. ADR 0003.
- Må IKKE introducere baggrundsdæmoner jf. Zero-Daemon invarianten.

---

## 📝 Revisions
- 2026-09-19: Task oprettet som ACTIVE for Autonomous RED to GREEN TDD Phase Transition jf. brugerdialog.

---

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test checkpoint_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 030-autonomous-red-to-green-tdd-transition`
