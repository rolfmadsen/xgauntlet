---
type: Architectural Decision Record
title: 'ADR 0013: Feature Architecture Boundaries, Living Feature Specifications, and ADR Lifecycle'
status: accepted
tags:
- architecture
- package-by-feature
- ddd
- feature
- adr-lifecycle
generated:
  by: antigravity/1.0
  at: '2026-11-14T17:30:00Z'
verified:
- by: human:maintainer
  at: '2026-11-14T17:30:00Z'
---

# 13. Feature Architecture Boundaries, Living Feature Specifications, and ADR Lifecycle

**Status**: `accepted` (supersedes [ADR 0012](0012-vertical-slice-boundaries-and-adr-lifecycle.md))  
**Date**: `2026-11-14`  

## Context
`ADR 0012` indførte fire kritiske makro-arkitektoniske mekanismer — de 4 universelle lag (`app -> features -> core -> shared`), deterministiske arkitektur-gates i `check-architecture`, levende kapabilitetskontrakter under `specs/` og den 4-fasede ADR-livscyklus.

Imidlertid anvendte `ADR 0012` og de efterfølgende specifikationer betegnelsen *"Vertical Slice"* / `slice` for funktionelle moduler under `features/<feature>/`. I Event Modeling og CQRS-praksis betegner en *slice* derimod en konkret hændelse (*event*) eller ét enkelt kommando/read-model-flow, hvor en samlet brugsfunktion (`Feature`) typisk omfatter flere events og slices. Den sproglige kollision skabte forvirring i `CONTEXT.md`, OKF-frontmatter, CLI-flag og konfigurationsnøgler.

## Decision
Vi erstatter `slice`- og `Vertical Slice`-terminologien fra `ADR 0012` med **`Feature`** (`Feature Module` og `Living Feature Specification`), mens alle arkitektoniske garantier, grænse-gates og livscyklusregler fra `ADR 0012` videreføres uændret:

1. **Ubiquitous Domain Hierarchy (`Domain -> Subdomain -> Bounded Context -> Feature -> Task`):**
   - Et `Feature` (`features/<feature>/` eller `features/<subdomain>/<feature>/`) er den autonome, black-box-indkapslede kapabilitet (*Stateful UI-Feature* eller *Stateless Pipeline-Feature*), der ejer sine egne modeller og eksponerer en smal offentlig facade uden direkte imports på tværs af søskende-features.
   - `slice` udgår som aktivt governance-begreb i `xGauntlet`.

2. **Det Universelle 4-Lags Dependency Guard (`app -> features -> core -> shared`):**
   - **Gate A (`LayerDirectionViolation`)**: Håndhæver streng top-down importretning.
   - **Gate B (`CrossFeatureImportViolation`)**: `isolate_features = true` og `feature_root` håndhæver black-box-isolation mellem både 1-niveau (`features/<feature>/`) og 2-niveau (`features/<subdomain>/<feature>/`) feature-mapper.
   - **Gate C (`ForbiddenCoreImportViolation`)**: Beskytter `core` (`Shared Kernel`) mod lækage af framework- og UI-afhængigheder.

3. **Living Feature Specifications (`specs/<feature>.md` / `specs/<subdomain>/<feature>.md`) & Scope-Aware Task Binding (`spec_ref`):**
   - Aktive feature/enhancement/bug tasks, hvis `scope` berører `architecture.feature_root`, deklarerer `feature:` (og valgfrit `domain:`/`subdomain:`) samt `spec_ref` mod deres levende feature-specifikation (stilladseret via `xgauntlet task new <slug> --feature <feature>`).
   - Tværgående opgaver uden for `feature_root` samt historiske tasks med `status: done` kræver ikke kunstig `specs/<feature>.md`-tilknytning.

4. **ADR Lifecycle Governance (`draft | accepted | superseded | deprecated`):**
   - Kun `accepted` (og `stable`) ADRs gælder for nye tasks, release-audits og `[[architecture.exemptions]]`. Historiske `superseded` ADRs (herunder `ADR 0012`) bevares udelukkende som arkivsporbarhed.

5. **Fuld Bagudkompatibilitet (Brownfield & Historiske Tasks):**
   - Konfigurationsparseren (`[architecture]`), OKF-frontmatter-parseren og CLI `task new` accepterer fortsat legacy-nøglerne `slice_root`, `isolate_slices`, `slice:` og `--slice` som bagudkompatible aliaser for henholdsvis `feature_root`, `isolate_features`, `feature:` og `--feature`.

## Consequences
- **Positiv**: Entydigt fælles sprog mellem kode (`src/features/`), opgavepakker (`tasks/`), specifikationer (`specs/`) og Event Modeling / CQRS-begrebsapparat uden begrebskollision på `slice`.
- **Positiv**: Alle eksisterende projekter, exemptions, historiske `DONE`-tasks (`001`–`080`) og legacy konfigurationer fungerer uændret uden brud.
- **Negativ**: Kræver midlertidige bagudkompatible aliaser i `config/loader.rs`, `tasks/okf.rs` og `cli`, indtil eventuelle legacy-konfigurationer migreres.
