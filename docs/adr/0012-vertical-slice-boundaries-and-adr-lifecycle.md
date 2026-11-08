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
1. **DDD & Vertical Slice Taksonomi og 4 Universelle Lag (`app -> features -> core -> shared`):**
   - **Domain (Problem Space):** Virksomhedens eller produktets overordnede faglige problemrum.
   - **Subdomain (`features/<subdomain>/` — valgfrit grupperingsniveau):** Fagligt delområde (*Core*, *Supporting* eller *Generic*), når et system har mange slices.
   - **Bounded Context (`CONTEXT.md`):** Den sproglige og arkitektoniske systemgrænse, hvori enhver domænemodel har én entydig Aristotelisk definition.
   - **Shared Kernel (`core/`) & Vertical Slice (`features/<slice>/` eller `features/<subdomain>/<slice>/`):** Den konkrete funktionelle kapabilitet (*Stateful UI-Slice* eller *Stateless Pipeline-Slice*) med Black-Box indkapsling (egen offentlig facade, egen tilstand/input og eksplicit `Action`/event/result output) og 0% intern mappestruktur-rigiditet. Leaf-filer inde i en slice-mappe (`models.rs`, `validator.rs` osv.) er altid interne i deres slice.
   - **De 4 lag (`app -> features -> core -> shared`):** Streng envejs-import oppefra og ned.
2. **Deterministiske Grænse-Gates i `xgauntlet check-architecture` (`[architecture]` i `gauntlet.toml`):**
   - **Gate A (`LayerDirectionViolation`):** Afviser imports fra et lavere lag til et højere lag (f.eks. `features -> app` eller `core -> features/app`).
   - **Gate B (`CrossSliceImportViolation`):** Auto-detekterer 1-niveau (`features/<slice>/<file>`) og 2-niveau (`features/<subdomain>/<slice>/<file>`) slice-dybde ud fra mappestrukturen under `slice_root`, og afviser direkte imports på tværs af forskellige slices når `isolate_slices = true`.
   - **Gate C (`ForbiddenCoreImportViolation`):** Afviser UI-/framework-imports i `core`-laget jf. `forbid_core_imports`.
   - Eksisterende projekter kan migrere gradvist via `allow_imports` (inkl. glob-stier som `path = "src/features/**"`) under `[[architecture.exemptions]]`.
3. **Levende Slice-Specifikationer (`specs/<slice>.md` eller `specs/<subdomain>/<slice>.md`):**
   - Tasks knyttes deterministisk til deres slice via `slice` (og valgfrit `subdomain`, med bagudkompatibelt alias `domain`) samt `spec_ref` i OKF-frontmatter (`check-spec` og `xgauntlet task new --slice <s> [--subdomain <sd>]`), mens eksisterende `status: done` tasks bevarer fuld bagudkompatibilitet.
   - `xgauntlet check-spec --context` verificerer både aktive task-specifikationer og kildemodellers synkronisering mod `CONTEXT.md` som del af `GAUNTLET`-pipelinen efter `GREEN`.
4. **4-Trins ADR-Livscyklus (`proposed | accepted | superseded | deprecated`):**
   - `check-adr` håndhæver gyldige statusser og kræver `superseded_by` når `status: superseded`.
   - `check-release` kræver udelukkende, at gældende `accepted` ADRs er refereret i `README.md` og `spec.md`.

## Consequences
- Arkitektonisk erosion og falsk VSA fanges deterministisk af `xgauntlet check-architecture` på tværs af både 1-niveau (`features/<slice>/`) og 2-niveau (`features/<subdomain>/<slice>/`) projekter i Rust, TypeScript/React, Java, Python og Go — herunder via dogfooding i `xgauntlet`s eget repository.
- Funktionelle kapabiliteter dokumenteres i levende slice-specifikationer (`specs/<slice>.md` eller `specs/<subdomain>/<slice>.md`), mens `docs/adr/` forbliver slankt og forbeholdt ægte tværgående arkitekturbeslutninger.
