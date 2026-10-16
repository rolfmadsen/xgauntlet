---
type: Architectural Decision Record
title: 'ADR 0008: Sovereign Agentic SDLC, Task Scoping and Accessible Design Tokens'
status: accepted
tags:
- architecture
- adr
- agentic-sdlc
- task-scoping
- vertical-slices
- design-tokens
- accessibility
- wai-2.0
generated:
  by: antigravity/gemini-3.8-flash
  at: '2026-10-16T18:12:00Z'
verified: []
---

# 8. Sovereign Agentic SDLC, Task Scoping and Accessible Design Tokens

**Status**: `accepted` (Extends ADR 0001, ADR 0003, and ADR 0007)  
**Date**: `2026-10-16`  

## Context
Erfaringer fra virkelige projekter under xGauntlet-governance (herunder desktop- og GUI-systemer som Kant) har afdækket kritiske sårbarheder i snitfladen mellem AI-agenternes adfærd og xGauntlets deterministiske håndhævelse:

1. **Modstridende Eksterne Skills**: Indlejrede skills fra tredjepart (f.eks. Matt Pococks `to-spec` med dogmet »the ideal number of seams is one« og `old-coder` med Python/Gherkin artefakter) har instrueret agenter i at modarbejde xGauntlets arkitekturinvarianter. Agenterne har samlet samtlige tests i én monolitisk testfil (fx 12.000 linjer i `tests/acceptance.rs`) i stedet for modulære slice-tests.
2. **Manglende Task Scoping i Policy Engine**: `EnforcementContext` i WASM policy-motoren tillod hidtil enhver skrivning til kildekode og tests, så længe en task blot var aktiv (`has_active_task = true`). Der var ingen afgrænsning af, *hvilke* mapper eller moduler tasken havde mandat til at ændre, hvilket muliggjorde uhindret monolit-vækst (fx 11.500 linjer i `src/ui/app.rs`).
3. **Visuelt og Tilgængelighedsmæssigt Forfald (Styling & WAI 2.0 Drift)**: Ved udvikling i vertikale slices opfandt agenterne lokale ad-hoc farver, inkonsistente paddings og ufuldstændige knapper uden tastaturfokus, hvilket overtrådte WAI 2.0 / WCAG AA tilgængelighedskrav (kontrast $\ge 4.5:1$).

## Decision
Vi etablerer en suveræn Agentic SDLC-arkitektur for xGauntlet baseret på tre ufravigelige søjler:

### 1. Den 7-Fasede Kanoniske Agentic SDLC Livscyklus
Vi formaliserer xGauntlets udviklingsløkke i 7 distinkte faser, understøttet af dedikerede roller og mekaniske gates jf. `PipelinePhase` i `jit.rs`:
1. **Ideation & Context** (`discover` skill): System Architect etablerer forretningsordbog i `CONTEXT.md` og ADR'er.
2. **Specification & Task Binding** (`spec` skill): Requirements Engineer opretter scoped tasks med acceptkriterier og WAI 2.0 krav.
3. **Implementation TDD** (`tdd` skill): TDD Craftsman kører Red $\to$ Green $\to$ Refactor mod dedikerede slice-testfiler.
4. **Multi-Layer Verification**: QA Engineer eksekverer den fulde pipeline i `gauntlet.toml` (linters, types, tests, mutation gauntlet).
5. **Standards & Spec Audit** (`review` skill): Code Reviewer auditerer git diff på to ortogonale akser (Spec compliance vs. Coding standards/WAI 2.0).
6. **Evidence Integrity & Drift Check**: Evidence Auditor forsegler `verification-report.json` og lokalt atomisk checkpoint commit.
7. **Release Readiness** (`release` skill): Release Engineer validerer versionsharmoni, CHANGELOG og pakningskrav (`check-release`).

### 2. Afskaffelse af Eksterne Skills: xGauntlet Sovereign Skill Suite
Vi kasserer de 11 lånte tredjepartsskills og erstatter dem med xGauntlets egne suveræne, formålsbyggede skills baseret på anerkendte industribegreber:
- `discover`: Sokratisk domænemodellering efter Aristoteles' formel (*genus et differentiam*).
- `spec`: Vertical Slice nedbrydning med obligatorisk scoping og UI-kontrakter.
- `tdd`: Stringent Red-Green-Refactor koblet til `xgauntlet checkpoint`.
- `review`: 2-akset audit af spec-krav, Fowler smells og WAI 2.0 tilgængelighed.
- `diagnose`: Disciplineret rodårsagsanalyse (`reproduce -> minimize -> hypothesize -> fix`).
- `release`: Pakning, semantisk versionering og udgivelseskontrol.

### 3. Maskinel Task Scoping & Test-File Governance
- **OKF v0.2 Udvidelse**: Hver task i `tasks/<id>.md` SKAL deklarere `scope: ["stier/**"]` og `test_refs: ["tests/<slice>_test.rs"]`.
- **WASM Policy Håndhævelse**: `EnforcementContext` udvides med `allowed_scopes` og `allowed_test_targets`. Skrivning til produktionskode eller tests uden for taskens scope afvises med `DecisionVerdict::Deny` (fejlkode 4037).
- **Anti-Monolit Test Guard**: `xgauntlet check-spec` advarer og blokerer mod at tilføje tests til overdimensionerede testfiler (> 1.000 linjer).

### 4. Accessible Design Tokens & Sovereign UI Seam
- **The Invariant Component Layer**: For alle UI-projekter etableres et suverænt design-system (`src/design_system/`):
  - `tokens`: Semantiske farver, spacing og typografi med garanteret WAI 2.0 / WCAG AA kontrast ($\ge 4.5:1$).
  - `primitives`: Dumme UI-komponenter (Button, Input, Modal, Panel) med indbygget 2px kontraststærk fokusring og tastaturnavigation.
- **Deklarativ Arkitekturkontrol**: `gauntlet.toml` forbyder rå farver (`Color::from_rgb`, inline CSS) i feature-slices. Slices må kun sammensætte UI via design-systemets primitiver.

## Consequences
- **Positivt**: Fjerner monolit-vækst og tvinger agenter til at respektere vertikale slices og modulære testfiler.
- **Positivt**: Ensartet visuelt design og garanteret WAI 2.0 tilgængelighed på tværs af autonome slices.
- **Positivt**: Krystalklar entydighed for AI-agenter uden modstridende eksterne dogmer.
- **Trade-off**: Eksisterende tasks og skabeloner skal migreres til at deklarere `scope` og `test_refs`.
