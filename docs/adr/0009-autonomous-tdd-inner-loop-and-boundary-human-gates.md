---
type: Architectural Decision Record
title: 'ADR 0009: Autonomous TDD Inner Loop and Boundary Human-in-the-Loop Gates'
status: accepted
tags:
- architecture
- adr
- tdd
- autonomy
- hitl
- checkpoint
- preflight
generated:
  by: process:antigravity-task-init
  at: '2026-10-25T16:30:00Z'
verified: []
---

# 9. Autonomous TDD Inner Loop and Boundary Human-in-the-Loop Gates

**Status**: `accepted` (Extends ADR 0003 and ADR 0008)  
**Date**: `2026-10-25`  

## Context
Tidlige agent-rammeværker påkrævede manuel Human-in-the-Loop (HITL) godkendelse ved hvert eneste skridt i en TDD-cyklus for at forhindre, at en model "løber løbsk" eller implementerer mod en forkert test. I xGauntlets arkitektur er en turn-pause mellem `RED` og `GREEN` imidlertid blevet en ren friktionskilde uden sikkerhedsmæssig merværdi:

1. **Mekanisk Pre-Flight Verifikation (`xgauntlet checkpoint --phase red`)**: Checkpoint-motoren eksekverer testlaget og beviser matematisk og empirisk, at den nye accepttest rent faktisk fejler med den forventede fejl (exit code != 0).
2. **Atomiske Lokale Checkpoints (ADR 0003)**: Hver fase forsegles i et lokalt Git-commit (`test(<id>): ... [RED]`), hvilket giver et øjeblikkeligt tilbagerulningspunkt, hvis den grønne implementation skulle spore af.
3. **Frosne Seams & Multi-Layer Gauntlet**: Test-seams er allerede fastlagt og godkendt under `SPEC`-fasen, og den samlede verifikations-gauntlet (linter, types, tests, mutation, spec-check) venter uanset hvad i slutningen af opgaven.

At tvinge agenten til at pause og spørge brugeren om lov til at skrive produktionskode for at få en verificeret rød test til at passere, fragmenterer flowet og forøger risikoen for context rot.

## Decision
Vi etablerer en skarp arkitektonisk sondring mellem **Menneskelige Grænse-Gates** og den **Autonome Indre Løkke**:

### 1. Menneskelige Grænse-Gates (Beholdes 100%)
Mennesket bevarer suveræn kontrol over de to afgørende ydre grænser:
- **Gate 1: SPEC Godkendelse**: Mennesket godkender opgavens formål, acceptkriterier, scope og test-seams før kode skrives.
- **Gate 2: AUDIT & HANDOFF Godkendelse**: Mennesket godkender to-akset code-review, verifikationsrapport og endelig opgavelukning.

### 2. Autonom Indre Løkke (`[RED ➔ GREEN ➔ REFACTOR]`)
Når den røde accepttest er skrevet og preflight-verificeret af `xgauntlet checkpoint --phase red`:
- Agenten må **IKKE** standse op eller afvente brugerbekræftelse.
- Agenten skal umiddelbart overgå til `GREEN` fasen og implementere minimal kode for at få testen til at passere.
- Overgangen fra `RED` til `GREEN` understøttes maskinelt af checkpoint-motoren via:
  - `CheckpointResult.autonomous_transition = true`
  - `CheckpointResult.next_phase = Some("green")`
  - Eksplicit CLI- og harness-prompting der instruerer agenten i at fortsætte uden ophold.

### 3. Sikkerhedsinvarianter
- Hvis en rød test uventet passerer, afviser `checkpoint --phase red` med `RedPhaseUnmet` og overgang til GREEN blokeres fail-closed.
- Hvis nye tests sniges ind under GREEN uden forudgående bevis for rød fejl, blokerer `BornGreenTestDetected` (Task 032).
- Zero remote push (ADR 0003) og Zero-Daemon invarianten forbliver ubrydelige.

## Consequences
- **Positive**:
  - Højere gennemstrømning og flydende udvikling for agent og udvikler.
  - Færre overflødige turn-pauser og lavere token-forbrug.
  - Bevarelse af 100% deterministisk tilbagerulning og proveniens via lokale Git commits.
- **Negative / Trade-offs**:
  - Udvikleren ser ikke den isolerede røde test før GREEN implementationen er fuldført, men kan altid inspicere det lokale `test(<id>): ... [RED]` commit i Git-loggen.
