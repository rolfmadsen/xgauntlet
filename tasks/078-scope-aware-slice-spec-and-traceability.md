---
type: Task Package
title: "Task 078: Scope-Aware Slice Spec Mapping & Two-Way Spec-Feature-Task Traceability"
description: "Gør check-spec slice-validering scope-bevidst (kræv kun specs/<slice>.md når taskens scope faktisk berører slice_root) og etablér synlig to-vejs sporbarhed mellem spec.md, specs/<slice>.md og tasks/<id>.md."
status: done
intent: enhancement
slice: tasks
spec_ref: "specs/tasks.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T15:16:00Z" }
tags: [tasks, check-spec, slice, traceability, brownfield, vsa]
scope:
  - "specs/tasks.md"
  - "specs/architecture.md"
  - "specs/release.md"
  - "specs/topology.md"
  - "spec.md"
  - "tasks/078-scope-aware-slice-spec-and-traceability.md"
  - "crates/xgauntlet-core/src/features/tasks/validator.rs"
  - "crates/xgauntlet-core/src/features/tasks/lifecycle.rs"
  - "crates/xgauntlet-core/tests/slice_architecture_and_adr_lifecycle_test.rs"
  - ".agents/plugins/xgauntlet/skills/spec/SKILL.md"
  - "evidence.md"
  - "verification-report.json"
test_refs:
  - "crates/xgauntlet-core/tests/slice_architecture_and_adr_lifecycle_test.rs"
---

# Task 078: Scope-Aware Slice Spec Mapping & Two-Way Spec-Feature-Task Traceability

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Sikre en reel, meningsfuld sammenhæng mellem **Applikation (`spec.md`) $\leftrightarrow$ Feature/Slice nu-tilstand (`specs/<slice>.md`) $\leftrightarrow$ Ændringsopgave (`tasks/<id>.md`)** uden at tvinge udviklere eller AI-agenter til at oprette kunstige `specs/*.md`-filer for tværgående opgaver eller blokere brownfield-migrering:

1. **Scope-bevidst `validate_task_slice_mapping` i `validator.rs`:**
   - Hvis en aktiv task **ikke** eksplicit angiver `domain`, `slice` eller `spec_ref`, kræves `slice` + `spec_ref` **kun** når:
     1. `intent` er `feature`, `enhancement` eller `bug`, **og**
     2. workspacet bruger slices (`specs/` eksisterer eller `isolate_slices = true`), **og**
     3. mindst én sti i taskens `scope` faktisk berører `architecture.slice_root` (enten direkte under `slice_root`, f.eks. `src/features/canvas/**`, eller via et bredt parent-glob som `src/**`).
   - Opgaver, hvis `scope` udelukkende berører filer uden for `slice_root` (f.eks. `.agents/**`, `AGENTS.md`, `docs/**`, `README.md`, `packages/cli/**`, `crates/xgauntlet-cli/**`), fritages automatisk for `MISSING_TASK_SLICE_MAPPING`.
   - Hvis en task derimod *eksplicit* angiver `domain`, `slice` eller `spec_ref`, valideres stien og filens eksistens fortsat strengt.
2. **To-vejs menneskelig sporbarhed (`spec.md` $\leftrightarrow$ `specs/<slice>.md` $\leftrightarrow$ `tasks/`):**
   - `spec.md` linker direkte til de levende feature-specifikationer i `specs/<slice>.md`.
   - Hver `specs/<slice>.md` (samt templates oprettet af `xgauntlet task new --slice <slice>`) indeholder en `## 📋 Relaterede Tasks`-sektion med klikbare links til de tasks, der har formet slicen, så udvikleren kan navigere begge veje mellem featurens nu-tilstand og dens ændringshistorik.

## 📋 Acceptance Criteria
- [x] **AC1 (Scope-Aware Slice Mapping i `validator.rs`):**
  - En aktiv `feature`/`enhancement`/`bug`-task uden `slice`/`spec_ref`, hvis `scope` *ikke* rammer `architecture.slice_root` (f.eks. `scope: [".agents/plugins/xgauntlet/skills/**"]`), består `check_task_specification` med 0 `MISSING_TASK_SLICE_MAPPING`-fejl.
  - En aktiv `feature`/`enhancement`/`bug`-task uden `slice`/`spec_ref`, hvis `scope` *rammer* `architecture.slice_root` (f.eks. `scope: ["src/features/diagram/canvas/**"]`), fejler fortsat med `MISSING_TASK_SLICE_MAPPING`.
- [x] **AC2 (To-vejs navigations- og sporbarhedsstruktur):**
  - `TaskScaffolder` (`lifecycle.rs`) inkluderer en `## 📋 Relaterede Tasks`-sektion ved oprettelse af nye `specs/<slice>.md`-filer og tilføjer link til den nye task.
  - `spec.md` og eksisterende `specs/*.md` (`architecture.md`, `release.md`, `topology.md`, samt ny `specs/tasks.md`) opdateres med to-vejs links mellem system-spec, slice-spec og relaterede tasks.
- [x] **AC3 (Skill-dokumentation & Regressionstests):**
  - `.agents/plugins/xgauntlet/skills/spec/SKILL.md` dokumenterer samspillet mellem `spec.md` (Applikation), `specs/<slice>.md` (Feature nu-tilstand) og `tasks/<id>.md` (Delta), samt at kun tasks der rører `slice_root` kræver `spec_ref`.
  - Nye og eksisterende tests i `crates/xgauntlet-core/tests/slice_architecture_and_adr_lifecycle_test.rs` passerer 100%.

## 🚫 Must NOT
- Må IKKE svække Gate A, Gate B eller Gate C i `xgauntlet check-architecture`.
- Må IKKE kræve `specs/<slice>.md` for uafhængige slices i et brownfield-projekt, som den aktive task slet ikke rører ved.
- Må IKKE bryde bagudkompatibilitet for eksisterende `status: done` tasks.

## 📝 Revisions
- 2026-11-14: Oprettet opgavepakke efter arkitektonisk sparring om sammenhængen mellem `spec.md`, `specs/<slice>.md`, `tasks/` og brownfield-projekter.

## 🧪 Verifikation
- `target/debug/xgauntlet check-spec -t 078`
- `cargo test -p xgauntlet-core --test slice_architecture_and_adr_lifecycle_test`
- `target/debug/xgauntlet verify --task 078`

