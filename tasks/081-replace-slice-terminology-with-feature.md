---
type: Task Package
title: "Task 081: Replace Slice Terminology with Feature Architecture"
description: "Afvikl det tvetydige 'slice'-begreb i xGauntlet og erstat det konsekvent med 'Feature' på tværs af CONTEXT.md, ADR, OKF-frontmatter, CLI, specs/ og check-architecture med 100% bagudkompatibilitet."
status: active
intent: refactor
feature: architecture
spec_ref: "specs/architecture.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T16:52:00Z" }
tags: [architecture, feature, slice, ddd, terminology, refactor, adr, cli]
scope:
  - "CONTEXT.md"
  - "spec.md"
  - "gauntlet.toml"
  - "README.md"
  - "packages/cli/README.md"
  - "CODING_STANDARDS.md"
  - "docs/adr/**"
  - "specs/**"
  - "tasks/081-replace-slice-terminology-with-feature.md"
  - "crates/xgauntlet-core/src/features/architecture/**"
  - "crates/xgauntlet-core/src/features/config/models.rs"
  - "crates/xgauntlet-core/src/features/config/loader.rs"
  - "crates/xgauntlet-core/src/features/scaffold/templates.rs"
  - "crates/xgauntlet-core/src/features/tasks/models.rs"
  - "crates/xgauntlet-core/src/features/tasks/okf.rs"
  - "crates/xgauntlet-core/src/features/tasks/parser.rs"
  - "crates/xgauntlet-core/src/features/tasks/validator.rs"
  - "crates/xgauntlet-core/src/features/tasks/lifecycle.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/checkers.rs"
  - "crates/xgauntlet-core/tests/slice_architecture_and_adr_lifecycle_test.rs"
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
  - "crates/xgauntlet-core/tests/architecture_gate_test.rs"
  - "crates/xgauntlet-core/tests/architecture_scaffold_template_test.rs"
  - "crates/xgauntlet-core/tests/mermaid_and_glossary_drift_test.rs"
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
  - ".agents/plugins/xgauntlet/skills/**"
test_refs:
  - "crates/xgauntlet-core/tests/slice_architecture_and_adr_lifecycle_test.rs"
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
---

# Task 081: Replace Slice Terminology with Feature Architecture

**Status**: `ACTIVE`
**Intent**: `🔄 REFACTOR`
**Oprettet**: `2026-11-14`

## 🎯 Formål
Fjerne sproglig og arkitektonisk forvirring omkring begrebet **"slice"** i `xGauntlet` ved at erstatte det med **`Feature`** på tværs af domæneordbog, ADR-governance, CLI, konfigurationsskema og specifikationer:

1. **Baggrund & Arkitektonisk Konklusion:**
   - I dag bruger `xGauntlet` (`ADR 0012`, `CONTEXT.md`, `gauntlet.toml`, `check-spec`, `check-architecture`) betegnelsen *"Vertical Slice"* / `slice` til det, der reelt er **brugervendte eller funktionelle Features** (f.eks. `export`, `glossary`, `tasks`, `checkpoint` under `features/<feature>/`, jf. `ADR 0001`).
   - I Event Modeling / CQRS-praksis hænger en *slice* derimod sammen med et konkret **event / én hændelse** i applikationen — hvor der typisk trigges flere events inden for én og samme `Feature`.
   - At kalde en samlet `Feature` for en `slice` skaber derfor en unødig begrebskollision og tvinger udviklere til at bruge et teknisk begreb forkert i `specs/<navn>.md`, CLI-flag og task-frontmatter.
2. **Målretning af Begrebsmodellen (`Feature` > ingen `slice`-overlæsning):**
   - **Ubiquitous Language & ADR (`CONTEXT.md` + `docs/adr/`):** Opret en ny ADR (eller opdatér/supersede `ADR 0012` via eksplicit `superseded_by`-linje), der fastlægger `Feature` (`features/<feature>/` eller `features/<subdomain>/<feature>/`) og `Living Feature Specification` (`specs/<feature>.md`) som den officielle model mellem `Domain/Subdomain` og `Task`, mens `slice` fjernes fra xGauntlets governance-begreber (evt. tilføjes `slice` under `_Avoid_` for `Feature` i `CONTEXT.md`).
   - **OKF Task Frontmatter & CLI (`tasks/`, `lifecycle.rs`, `validator.rs`, `cli`):**
     - Indfør `feature:` (og CLI-flag `--feature`) som primært felt i OKF-frontmatter og `xgauntlet task new`.
     - Bevar bagudkompatibel læsning af `slice:` / `--slice` i eksisterende `DONE`-tasks og ældre repos.
     - Tasks med `scope` uden for `feature_root` kræver fortsat ikke kunstig `specs/<feature>.md`-tilknytning.
   - **Architecture Gatekeeper & `gauntlet.toml` (`architecture/`, `config/`):**
     - Understøt `feature_root` og `isolate_features` som primære nøgler i `[architecture]` med fuld bagudkompatibel fallback fra `slice_root` og `isolate_slices`.
     - Opdatér Gate B-diagnostik og eksisterende `specs/*.md` samt `.agents/plugins/xgauntlet/skills/**` til konsekvent brug af `Feature`-terminologi.

## 📋 Acceptance Criteria
- [ ] **AC1 (ADR & Ubiquitous Language i `CONTEXT.md`):**
  - Ny `accepted` ADR erstatter (`superseded_by`) `ADR 0012`'s `Vertical Slice`-terminologi med `Feature` (`Feature Module` & `Living Feature Specification`), og `docs/adr/README.md` + `spec.md` opdateres tilsvarende.
  - `CONTEXT.md` definerer `Feature` / `Living Feature Specification` efter Aristotelisk formel og fjerner `slice` som aktivt governance-begreb (`check-spec --context` passerer uden orphan models eller avoid-brud).
- [ ] **AC2 (Task Frontmatter, Validator & CLI med Bagudkompatibilitet):**
  - OKF-parseren, `validator.rs` og `xgauntlet task new` bruger `feature` og `--feature` som primære navne ved scaffolding af nye tasks og `specs/<feature>.md`.
  - Eksisterende tasks (inkl. alle `status: done` tasks i `tasks/` og `tasks/archive/`) med `slice:` i YAML frontmatter fortsætter med at parse og validere uden fejl.
- [ ] **AC3 (`gauntlet.toml`, Architecture Gates, Specs & Skills):**
  - `[architecture]` understøtter `feature_root` og `isolate_features` (med fallback til legacy `slice_root` og `isolate_slices`).
  - `specs/*.md` (herunder `specs/architecture.md`, `specs/tasks.md`, `specs/release.md`, `specs/topology.md`) og sovereign skills i `.agents/plugins/xgauntlet/skills/` er migreret til `Feature`-terminologi.
- [ ] **AC4 (Isolerede Tests):**
  - Integrationstests i `slice_architecture_and_adr_lifecycle_test.rs` og `task_lifecycle_test.rs` verificerer både nye `feature` / `feature_root` / `--feature`-kontrakter og fuld bagudkompatibilitet med legacy `slice` / `slice_root`.

## 🚫 Must NOT
- Må IKKE bryde eksisterende projekter eller historiske tasks (`001`–`080`), der stadig bruger `slice:`, `slice_root` eller `isolate_slices`.
- Må IKKE genindføre krav om `specs/<feature>.md` for tværgående tasks uden for `feature_root` (bevar scope-bevidst reglen fra Task 078).
- Må IKKE koble `tasks/`-mappestrukturen til feature-mapper (bevar afkoblingen fra Task 079).
- Må IKKE ændre `gauntlet_policy.wasm` eller slække `ADR 0010`/`ADR 0003`-invarianter.

## 📝 Revisions
- 2026-11-14: Oprettet `DRAFT`-opgavepakke efter arkitektur-sparring om forskellen på `Feature` (problem-/kapabilitetsniveau) og `slice` (event-/hændelsesniveau) samt konklusion om at fjerne `slice`-begrebet fra xGauntlet.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec --context -t 081`
- `cargo run -p xgauntlet-cli -- check-architecture`
- `cargo test -p xgauntlet-core --test slice_architecture_and_adr_lifecycle_test`
- `cargo test -p xgauntlet-core --test task_lifecycle_test`
- `cargo run -p xgauntlet-cli -- verify --task 081`

