---
type: Task Package
title: "Task 069: Correct DDD/VSA Taxonomy, 1- and 2-Level Slice Isolation, and Dogfooding"
description: "Align Domain -> Subdomain -> Bounded Context -> Shared Kernel / Vertical Slice hierarchy, support 1- and 2-level slices in Gate B and check-spec, and dogfood isolate_slices and check-spec --context in xgauntlet."
status: done
intent: enhancement
slice: architecture
spec_ref: "specs/architecture.md"
generated: { by: process:antigravity-task-init, at: "2026-11-08T14:50:00Z" }
tags: [architecture, ddd, vsa, slice-isolation, glossary, dogfooding]
scope:
  - "crates/xgauntlet-core/src/features/architecture/**"
  - "crates/xgauntlet-core/src/features/tasks/**"
  - "crates/xgauntlet-core/src/features/scaffold/**"
  - "crates/xgauntlet-cli/src/**"
  - "crates/xgauntlet-core/tests/**"
  - ".agents/plugins/xgauntlet/skills/**"
  - "specs/**"
  - "docs/adr/**"
  - "gauntlet.toml"
  - "CONTEXT.md"
  - "spec.md"
  - "CODING_STANDARDS.md"
  - "README.md"
  - "packages/cli/README.md"
test_refs:
  - "crates/xgauntlet-core/tests/architecture_gate_test.rs"
  - "crates/xgauntlet-core/tests/mermaid_and_glossary_drift_test.rs"
---

# Task 069: Correct DDD/VSA Taxonomy, 1- and 2-Level Slice Isolation, and Dogfooding

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Ret de konceptuelle og sekventielle inversioner mellem Domain, Subdomain, Bounded Context (`CONTEXT.md`), Shared Kernel (`core/`), Vertical Slice Architecture (`features/`) og Tasks (`tasks/` & `specs/`), og aktivér fuld dogfooding i `xgauntlet`s egen `gauntlet.toml`:
1. **DDD & VSA Taksonomi i `CONTEXT.md`, `ADR 0012` og Bundled Skills (`discover`, `spec`, `tdd`):**
   - Etablér den korrekte begrebsrækkefølge: `Domain (Problem Space) -> Subdomain (Optional Grouping) -> Bounded Context (CONTEXT.md) -> Shared Kernel (core/) & Vertical Slice (features/<slice>/ eller features/<subdomain>/<slice>/)`.
   - Præcisér livscyklus for `CONTEXT.md`: Primære forretningsbegreber fastlægges i `DISCOVER`/`SPEC`, mens nyintroducerede kildemodeller fra `GREEN`/`REFACTOR` verificeres deterministisk i `GAUNTLET` (`xgauntlet check-spec --context`).
2. **Auto-detektion af 1-niveau og 2-niveau Slices i Gate B (`boundaries.rs`) og Glob Exemptions (`checker.rs`):**
   - I `extract_slice_identity`: Skeln mellem mappeniveauer og filnavn under `slice_root`, så 1-niveau mapper (`features/<slice>/<file>.rs`) udtrækkes som `("", "<slice>")` og 2-niveau mapper (`features/<subdomain>/<slice>/<file>.rs`) udtrækkes som `("<subdomain>", "<slice>")`. Interne filer i samme slice (`models.rs`, `validator.rs` osv.) må aldrig behandles som separate slices.
   - I `checker.rs`: Understøt glob-matching (`/**`) på `[[architecture.exemptions]]` `path` og evaluér `allow_imports` på tværs af alle matchende exemptions for en fil.
3. **Valgfri `subdomain` (med bagudkompatibelt alias `domain`) i Tasks (`check-spec` & `task new`):**
   - Understøt både 1-niveau slice-binding (`slice: <slice>` + `spec_ref: "specs/<slice>.md"`) og 2-niveau slice-binding (`subdomain: <sd>` / `domain: <sd>`, `slice: <s>` + `spec_ref: "specs/<sd>/<s>.md"`).
4. **Parentetiske Model-Aliaser i `validate_glossary_drift` (`glossary.rs`) & Dogfooding i `gauntlet.toml`:**
   - Understøt parentetiske og skråstregs-separerede model-aliaser i `CONTEXT.md` (f.eks. `**Architecture Gate (ArchitectureConfig / ArchitectureReport)**:`), så `xgauntlet check-spec --context` består med 0 orphan models og 0 avoid-violations på `xgauntlet`s eget workspace, samtidig med at uregistrerede modeller stadig afvises.
   - Aktivér `isolate_slices = true` (`slice_root = "crates/xgauntlet-core/src/features"`) og `check-spec --context` i `xgauntlet`s egen `gauntlet.toml`.

## 📋 Acceptance Criteria
- [x] **AC1 (1-Level & 2-Level Slice Identity in Gate B):** `run_architecture_check` med `isolate_slices = true` tillader interne imports mellem søskendefiler i en 1-niveau slice (`src/features/tasks/validator.rs` der importerer `crate::features::tasks::models::TaskPackage`), men afviser uautoriserede tværgående slice-imports (`src/features/tasks/validator.rs` der importerer `crate::features::release::models::ReleaseReport`) med `CrossSliceImportViolation`, og bevarer fuld 2-niveau slice-isolation (`src/features/<subdomain>/<slice>/...`).
- [x] **AC2 (Glob Path Exemptions for `allow_imports`):** `[[architecture.exemptions]]` med `path = "src/features/**"` eller `path = "crates/xgauntlet-core/src/features/**"` og `allow_imports = [...]` matcher kildefiler under præfikset og kombineres korrekt med fil-specifikke linjeloft-exemptions.
- [x] **AC3 (1-Level & 2-Level `spec_ref` in `check-spec` and `task new`):** `check_task_specification` og `TaskScaffolder` accepterer både 1-niveau tasks (`slice: architecture`, `spec_ref: "specs/architecture.md"` uden `domain`/`subdomain`) og 2-niveau tasks (`subdomain: billing` eller `domain: billing`, `slice: checkout`, `spec_ref: "specs/billing/checkout.md"`).
- [x] **AC4 (Glossary Parenthetical Aliases & Dogfooding):** `validate_glossary_drift` udtrækker både hovedtermen og parentetiske `/`-separerede model-aliaser fra `CONTEXT.md`, afviser fortsat uregistrerede modeller (`UnregisteredBountyEvidence`), og består 100% på `xgauntlet`s eget repository via `xgauntlet check-spec --context` og `xgauntlet check-architecture`.

## 🚫 Must NOT
- Må IKKE ændre i `crates/gauntlet-policy-engine` eller `wit/gauntlet_policy.wit`.
- Må IKKE bryde bagudkompatibilitet for eksisterende `status: done` tasks eller projekter med `domain:` i task-frontmatter.
- Må IKKE svække Orphan Model detektionen for uregistrerede `pub struct`/`pub enum` typer i `models.rs`.
- Må IKKE lade `README.md` og `packages/cli/README.md` divergere.

## 📝 Revisions
- 2026-11-08: Oprettet Task 069 efter arkitektonisk gennemgang og brugerafklaring af DDD/VSA-taksonomi og dogfooding.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test architecture_gate_test`
- `cargo test -p xgauntlet-core --test mermaid_and_glossary_drift_test`
- `cargo run -p xgauntlet-cli -- check-architecture`
- `cargo run -p xgauntlet-cli -- check-spec --context`
- `cargo run -p xgauntlet-cli -- verify --task 069`
