---
type: Architectural Decision Record
title: 'ADR 0012: Deterministic Vertical Slice Boundary Gates, Living Slice Specifications, and ADR Lifecycle'
status: accepted
tags:
- architecture
- vsa
- ddd
- slice
- adr-lifecycle
generated:
  by: human:maintainer
  at: '2026-11-07T17:25:00Z'
verified:
- by: human:maintainer
  at: '2026-11-07T17:25:00Z'
---

# 12. Deterministic Vertical Slice Boundary Gates, Living Slice Specifications, and ADR Lifecycle

**Status**: `accepted`  
**Date**: `2026-11-07`  

## Context
En dybdegående arkitekturanalyse af et større produktionssystem udviklet gennem 128 tasks (`kant`) afslørede fire makro-arkitektoniske svagheder, som opstår over tid, når governance kun fokuserer på mikro-determinisme (én task ad gangen) og rå fil-linjetal:
1. **Falsk Vertical Slice Architecture (VSA):** Kildekode blev flyttet ud i `src/features/`, men importerede fortsat `crate::ui::app::Message` baglæns og modtog op til 28 løse state-argumenter via `#[allow(clippy::too_many_arguments)]`, mens al tilstand og forretningslogik forblev centraliseret i en monolitisk `app.rs`.
2. **Goodhart's Law på linjetal:** Rå linjetals-lofter alene forhindrer ikke cirkulær kobling eller lag-brud; tværtimod opmuntres kunstig opsplitning af sammenhængende logik uden reel indkapsling.
3. **Manglende mellemniveau (`Slice`) mellem Domain og Task:** Med få brede domænemapper og over 100 mikro-tasks forsvandt den levende specifikation af hver enkelt funktionel kapabilitet (`Slice`).
4. **ADR-inflation og zombie-ADRs:** Almindelige feature-specifikationer blev skrevet som ADRs, og `check-release` tvang selv `deprecated` og `superseded` ADRs til at forblive refereret i `README.md` og `spec.md`.

## Decision
Vi indfører en stack-agnostisk makro-arkitektonisk governance-model i `xgauntlet` uden ændringer i WASM-policy-motoren og uden at stramme eksisterende fil-linjelofter:
1. **4-Niveaus DDD & Slice Taksonomi og 4 Universelle Lag (`app -> features -> core -> shared`):**
   - **Bounded Context (`CONTEXT.md`):** Den sproglige og arkitektoniske systemgrænse.
   - **Domain (`features/<domain>/`):** Fagligt delområde inden for konteksten.
   - **Slice (`features/<domain>/<slice>/`):** Den konkrete funktionelle kapabilitet (*Stateful UI-Slice* eller *Stateless Pipeline-Slice*) med Black-Box indkapsling (egen offentlig facade, egen tilstand/input og eksplicit `Action`/event/result output) og 0% intern mappestruktur-rigiditet.
   - **De 4 lag (`app -> features -> core -> shared`):** Streng envejs-import oppefra og ned.
2. **Deterministiske Grænse-Gates i `xgauntlet check-architecture` (`[architecture]` i `gauntlet.toml`):**
   - **Gate A (`LayerDirectionViolation`):** Afviser imports fra et lavere lag til et højere lag (f.eks. `features -> app` eller `core -> features/app`).
   - **Gate B (`CrossSliceImportViolation`):** Afviser direkte imports på tværs af slices (`features/<d1>/<slice_a> -> features/<d2>/<slice_b>`) når `isolate_slices = true`.
   - **Gate C (`ForbiddenCoreImportViolation`):** Afviser UI-/framework-imports i `core`-laget jf. `forbid_core_imports`.
   - Eksisterende projekter kan migrere gradvist via `allow_imports` under `[[architecture.exemptions]]`.
3. **Levende Slice-Specifikationer (`specs/<domain>/<slice>.md`):**
   - Tasks knyttes deterministisk til deres slice via `domain`, `slice` og `spec_ref: "specs/<domain>/<slice>.md"` i OKF-frontmatter (`check-spec` og `xgauntlet task new --domain <d> --slice <s>`), mens eksisterende `status: done` tasks bevarer fuld bagudkompatibilitet.
4. **4-Trins ADR-Livscyklus (`proposed | accepted | superseded | deprecated`):**
   - `check-adr` håndhæver gyldige statusser og kræver `superseded_by` når `status: superseded`.
   - `check-release` kræver udelukkende, at gældende `accepted` ADRs er refereret i `README.md` og `spec.md`.

## Consequences
- Arkitektonisk erosion og falsk VSA fanges deterministisk af `xgauntlet check-architecture` på tværs af Rust, TypeScript/React, Java, Python og Go.
- Funktionelle kapabiliteter dokumenteres i levende slice-specifikationer (`specs/<domain>/<slice>.md`), mens `docs/adr/` forbliver slankt og forbeholdt ægte tværgående arkitekturbeslutninger.
