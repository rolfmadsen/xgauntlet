---
type: Architectural Decision Record
title: 'ADR 0010: ADR-Bound Architectural Exemptions and Monotonic Ratchet Governance'
status: accepted
tags:
- architecture
- adr
- ratchet
- governance
- exemptions
- fail-closed
generated:
  by: process:antigravity-task-init
  at: '2026-10-25T17:16:00Z'
verified: []
---

# 10. ADR-Bound Architectural Exemptions and Monotonic Ratchet Governance

**Status**: `accepted` (Extends ADR 0001, ADR 0003, and ADR 0008)  
**Date**: `2026-10-25`  

## Context
Deklarative arkitekturgrænser (`max_file_lines = 800` og `max_test_file_lines = 1000`) forhindrer uoverskuelige god-objects og monolitiske testfiler. For at rumme legitime arkitektoniske undtagelser understøtter `gauntlet.toml` sektionen `[[architecture.exemptions]]` med en `max_lines` ratchet-grænse.

Imidlertid opstår der et markant governance-problem, hvis ratchets blot hæves mekanisk:
1. **Stille Ratchet-Inflation**: Når en AI-agent eller udvikler tilføjer kode til en undtaget fil og rammer loftet, opstår fristelsen til blot at forøge `max_lines` (f.eks. bump fra 2.130 til 2.160 til 2.190 i `main.rs`) uden at undersøge, om filen burde dekomponeres, eller om arkitekturen er overtrådt.
2. **Ubegrundede Undtagelser**: Uden formel kobling til arkitekturbeslutninger bliver undtagelser en bekvemmelighedsventil, der gradvist udhuler kodesundheden og Package-by-Feature princippet (ADR 0001).
3. **Flad Numerisk Dogmatik**: Rå linjetal tager ikke højde for kontekst. En stor fil (f.eks. en central CLI-dispatcher eller en multi-adapter testmatrix) kan være arkitektonisk forsvarlig, hvis den er begrundet og accepteret, mens en 400-linjers fil kan være et arkitektonisk makværk.

## Decision
Vi etablerer et krav om **ADR-forankring** og **monoton ratchet-styring**:

### 1. Obligatorisk ADR-Forankring (`adr`-felt)
Enhver deklaration i `[[architecture.exemptions]]` SKAL indeholde et eksplicit `adr`-felt, der refererer til en gyldig Architecture Decision Record i `docs/adr/`:
- `xgauntlet check-architecture` og `validate_config` validerer fail-closed:
  - Exemption mangler `adr` eller er tom ➔ `ExemptionMissingAdr` fejl.
  - Refereret ADR-fil findes ikke på disken ➔ `ExemptionAdrNotFound` fejl.
  - Refereret ADR har ikke status `accepted` eller `stable` i sin OKF-frontmatter (f.eks. `draft`, `deprecated`, `superseded`) ➔ `ExemptionAdrNotAccepted` fejl.

### 2. Monoton Ratchet-Invariants (Non-Increasing Ceilings)
- `max_lines` ratchets er **monotont faldende** under autonom udvikling.
- Autonome agenter må **ALDRIG** hæve et ratchet-loft på egen hånd for at få en opgave igennem gatekeeperen. Agenter må kun sænke loftet (stramme skruen), når kode refaktoreres eller flyttes ud i delmoduler.
- At hæve en ratchet-grænse er en eksklusiv menneskelig arkitekturhandling (Human-in-the-Loop), som forudsætter:
  1. En eksplicit, accepteret ADR der redegør for, hvorfor dekomponering fravælges.
  2. Eksplicit menneskelig godkendelse i `SPEC`-fasen.

### 3. Eksisterende Undtagelser i `gauntlet.toml`
Repositoryets eksisterende undtagelser forankres i godkendte ADR'er:
- `crates/xgauntlet-cli/src/main.rs`: Forankres i `docs/adr/0010-adr-bound-architectural-exemptions-and-monotonic-ratchets.md` (CLI root dispatcher).
- `crates/xgauntlet-core/tests/harness_contracts_test.rs`: Forankres i `docs/adr/0004-harness-adapter-slices.md`.
- `crates/xgauntlet-core/tests/harness_adapters_test.rs`: Forankres i `docs/adr/0004-harness-adapter-slices.md`.

## Consequences
- **Positive**:
  - Fjerner muligheden for, at agenter i det skjulte udvider ratchet-lofter.
  - Tvinger arkitektonisk refleksion og dokumentation før linjegrænser dispenserer.
  - Skaber 100% sporbarhed mellem konfiguration og arkitektur-rationale.
- **Negative / Trade-offs**:
  - Hver ny arkitektonisk undtagelse kræver oprettelse af en godkendt ADR i `docs/adr/`.
