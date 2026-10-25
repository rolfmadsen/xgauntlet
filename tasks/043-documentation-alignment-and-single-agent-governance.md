---
type: Task Package
title: "Task 043: Documentation Alignment & Single-Agent Governance"
description: "Fjerne forældede referencer til parallelle subagenter og formalisere den transparente single-agent pair-programming model i ROADMAP.md og AGENTS.md"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-25T11:25:00Z" }
tags: [documentation, governance, single-agent, review-skill, transparency, clean-craftsmanship]
scope: ["ROADMAP.md", ".agents/AGENTS.md", ".agents/plugins/xgauntlet/skills/review/SKILL.md", "README.md", "packages/cli/README.md"]
test_refs: ["crates/xgauntlet-core/tests/single_agent_governance_test.rs"]
---

# Task 043: Documentation Alignment & Single-Agent Governance

**Status**: `DONE`  
**Intent**: `🔄 REFACTOR`  
**Oprettet**: `2026-10-25`  

## 🎯 Formål
Formålet med denne opgave er at bringe repositoryets autoritative governance- og roadmap-dokumentation i 100% overensstemmelse med xGauntlets faktiske arkitektur og samarbejdsmodel.

Historiske sektioner i `ROADMAP.md` og `.agents/AGENTS.md` beskriver stadig en hypotetisk model med "uafhængige parallelle subagenter" til to-akset review (`review`-skillen). Dette skaber forvirring om systemets virkemåde og strider imod xGauntlets kerneprincipper: **Clean Craftsmanship, absolut transparens og deterministisk enkelt-agent parløb**.

Opgaven afskaffer disse referencer og formaliserer, at to-akset review og andre opgaver udføres som sekventielle, fokuserede roller i rene sessioner med eksplicitte *Human-in-the-Loop* session handoffs.

## 📋 Acceptance Criteria
- [x] **AC1 (Opdatering af `ROADMAP.md`)**:
  - Gennemgå `ROADMAP.md` og fjerne alle formuleringer om kørsel af "parallelle subagenter" i relation til `review` og SDLC-pipelinen (herunder linje 167, 201 og 260).
  - Erstatte med en klar beskrivelse af, at Akse 1 (Specifikation) og Akse 2 (Standarder) evalueres af `Code Reviewer`-rollen i en dedikeret, isoleret session mod opgavens Git diff.
- [x] **AC2 (Opdatering af `.agents/AGENTS.md`)**:
  - Opdatere beskrivelsen af `review`-skillen i `.agents/AGENTS.md` (linje 33) så "running independent parallel sub-agents" erstattes af "running as an independent code reviewer role in a fresh session with Fowler code smells baseline".
- [x] **AC3 (Verifikation af `review/SKILL.md`)**:
  - Verificere at `.agents/plugins/xgauntlet/skills/review/SKILL.md` er fri for subagent-dogmer og udelukkende foreskriver den sekventielle to-aksede model baseret på Definition of Enough.
- [x] **AC4 (Dokumentation af Single-Agent & HITL Princippet)**:
  - Indsætte en kort, præcis arkitekturnote i `README.md` / `packages/cli/README.md` under "De 4 AI-roller", der ekspliciterer:
    - xGauntlet anvender en transparent enkelt-agent model under direkte menneskeligt opsyn.
    - Kontekst-renhed opnås via mekaniske Session Handoffs, ikke via uigennemskuelige baggrunds-subagenter.
- [x] **AC5 (Spec & Invariant Verifikation)**:
  - Verificere at `cargo run -p xgauntlet-cli -- check-spec -t 043` validerer med 0 fejl.

## 🚫 Must NOT
- Må IKKE fjerne den to-aksede review-metodik (Akse 1 vs. Akse 2) eller Definition of Enough – kun subagent-abstraktionen fjernes.
- Må IKKE introducere modstridende retningslinjer mellem `README.md`, `ROADMAP.md` og `.agents/AGENTS.md`.

## 📝 Revisions
- 2026-10-25: Task oprettet for at rydde op i forældede subagent-referencer og fastholde transparent enkelt-agent parløb.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 043`
- `git diff --check`
