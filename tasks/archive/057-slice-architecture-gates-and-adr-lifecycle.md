---
type: Task Package
title: "Task 057: Deterministic Slice Architecture Gates, Living Slice Specs & ADR Lifecycle"
description: "Indfører de 4 universelle arkitekturlag (app -> features -> core -> shared), deterministiske grænse-gates (Gate A/B/C) i check-architecture, Domain -> Slice -> Task spec_ref mapning i check-spec/task new, 4-trins ADR-livscyklus i check-adr/check-release samt polyglot dokumentation i de suveræne skills."
status: done
intent: feature
generated: { by: process:antigravity-task-init, at: "2026-11-07T17:24:00Z" }
tags: [architecture, vsa, ddd, slice, layers, adr-lifecycle, check-spec, polyglot]
scope:
  - "crates/xgauntlet-core/src/features/config/**"
  - "crates/xgauntlet-core/src/features/architecture/**"
  - "crates/xgauntlet-core/src/features/topology/**"
  - "crates/xgauntlet-core/src/features/tasks/**"
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-core/src/features/checkpoint/**"
  - "crates/xgauntlet-cli/src/**"
  - "crates/xgauntlet-core/tests/**"
  - ".agents/plugins/xgauntlet/**"
  - "docs/adr/**"
  - "CODING_STANDARDS.md"
  - "CONTEXT.md"
  - "spec.md"
  - "README.md"
  - "tasks/057-slice-architecture-gates-and-adr-lifecycle.md"
test_refs:
  - "crates/xgauntlet-core/tests/slice_architecture_and_adr_lifecycle_test.rs"
---

# Task 057: Deterministic Slice Architecture Gates, Living Slice Specs & ADR Lifecycle

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-11-07`

---

## 🎯 Formål & Baggrund

På baggrund af arkitekturanalysen af et 128-tasks produktionsprojekt (`kant`) løser denne opgave fire makro-arkitektoniske udfordringer i `xgauntlet` på tværs af sprog og frameworks (Rust/Iced, TypeScript/React, Java, Python, Go):
1. **Eliminering af Falsk Vertical Slice Architecture (VSA):** Forhindrer at moduler i `features/` importerer baglæns fra `app`/`ui`, eller at slices kobler direkte på tværs af hinanden uden om en eksplicit `Action`/event/facade-kontrakt.
2. **Grænsetjek fremfor Goodhart's Law på linjetal:** Bevarer den eksisterende advarsels-/exemption-model på fil-linjetal uændret, og tilføjer deterministiske import- og lag-gates i `xgauntlet check-architecture` med 0% ændring i WASM/hooks og 0% intern rigiditet indeni en slice.
3. **Deterministisk `Bounded Context -> Domain -> Slice -> Task` kobling:** Indfører det manglende mellemniveau (`Slice`) mellem domæne og task via `domain`, `slice` og `spec_ref` (`specs/<domain>/<slice>.md`), så hver task vedligeholder sin slices levende specifikation uden at bryde bagudkompatibiliteten for eksisterende `status: done` tasks.
4. **ADR-livscyklus & afvikling af zombie-ADRs:** Udvider `check-adr` og `check-release` med `status: proposed | accepted | superseded | deprecated` (med påkrævet `superseded_by` ved `superseded`), så kun gældende `accepted` ADRs kræves refereret i `README.md` og `spec.md`.

---

## 📋 Acceptance Criteria

- [x] **AC1 — Konfiguration af Arkitektur-Gates i `gauntlet.toml` (`[architecture]`):**
  - Udvid `ArchitectureConfig` og `ArchitectureExemption` i `crates/xgauntlet-core/src/features/config/` (inkl. custom TOML parser/renderer i `loader.rs`) med:
    - `layers: Vec<String>` (f.eks. `["app", "features", "core", "shared"]`, default tom = inaktiv).
    - `slice_root: String` (f.eks. `"src/features"`, default `"src/features"`).
    - `isolate_slices: bool` (default `false`).
    - `forbid_core_imports: Vec<String>` (f.eks. `["iced", "react", "javafx"]`, default tom).
    - `allow_imports: Vec<String>` på `[[architecture.exemptions]]` til eksplicit, begrundet fritagelse ved gradvis migrering.
- [x] **AC2 — Deterministiske Grænse-Gates (Gate A, B, C) i `xgauntlet check-architecture`:**
  - Udvid `topology/parser.rs` med `SourceLanguage::Java` (`import ...;`) ved siden af Rust, TypeScript/JS, Python og Go.
  - Implementér `crates/xgauntlet-core/src/features/architecture/boundaries.rs`:
    - **Gate A (`LayerDirectionViolation`):** Fejler når et lavere lag i `layers` importerer fra et højere lag (f.eks. `features -> app` eller `core -> features/app`).
    - **Gate B (`CrossSliceImportViolation`):** Fejler når `isolate_slices = true` og en fil i `<slice_root>/<d1>/<slice_a>` importerer direkte fra `<slice_root>/<d2>/<slice_b>` (hvor `(d1, slice_a) != (d2, slice_b)`), mens interne imports indeni samme slice er 100% frie.
    - **Gate C (`ForbiddenCoreImportViolation`):** Fejler når en fil i `core`-laget importerer en pakke opført i `forbid_core_imports`.
- [x] **AC3 — Deterministisk `Domain -> Slice -> Task` Mapning i `check-spec` og `task new`:**
  - Udvid `OkfMetadata` og `TaskPackageInfo` med `domain`, `slice` og `spec_ref`.
  - For aktive tasks (`status: active` med `intent: feature | enhancement | bug`) hvor enten (a) et af felterne `domain`/`slice`/`spec_ref` er sat, (b) `specs/`-mappen findes i workspacet, eller (c) `isolate_slices = true` i `gauntlet.toml`, validerer `check-spec`:
    1. At `domain`, `slice` og `spec_ref` alle er til stede (`MISSING_TASK_SLICE_MAPPING`).
    2. At `spec_ref` matcher `specs/<domain>/<slice>.md` (`INVALID_TASK_SPEC_REF`).
    3. At `specs/<domain>/<slice>.md` eksisterer på disken og er ikke-tom (`MISSING_SLICE_SPEC_FILE`).
  - Tasks med `status: done` (samt `draft`, `deprecated`, `superseded`) er 100% fritaget (fuld bagudkompatibilitet).
  - Udvid `xgauntlet task new` med `--domain <domain>` og `--slice <slice>`, som automatisk sætter `domain`, `slice`, `spec_ref: "specs/<domain>/<slice>.md"`, tilføjer `specs/<domain>/<slice>.md` til `scope:`, og scaffolder slice-specifikationen hvis den ikke findes.
  - Tillad `specs/` og `docs/` under `validate_staged_files_scope` i `checkpoint/engine.rs`.
- [x] **AC4 — ADR-livscyklus i `check-adr` og `check-release`:**
  - `check-adr` validerer, at ADR `status` er en af `proposed | accepted | superseded | deprecated` (samt OKF-aliasserne `draft` og `stable`) og afviser ugyldige statusser med `INVALID_ADR_STATUS`.
  - Når en ADR har `status: superseded`, kræver `check-adr` et ikke-tomt `superseded_by`-felt i frontmatter (`MISSING_ADR_SUPERSEDED_BY`).
  - Opdater `docs/adr/0002-cryptographic-evidence-authority.md` til `status: superseded` med `superseded_by: '0005-two-tier-verification-and-attestation-model.md'`, og opret [ADR 0012](docs/adr/0012-vertical-slice-boundaries-and-adr-lifecycle.md).
  - `check_adr_coverage` i `release/adr.rs` kræver kun, at `accepted` (og legacy `stable`) ADRs er refereret i `README.md` og `spec.md`, og fritager `proposed`, `draft`, `superseded` og `deprecated` ADRs.
- [x] **AC5 — Pædagogisk Polyglot Dokumentation & Opdatering af Sovereign Skills:**
  - Opdater `discover`, `spec`, `tdd`, `review` og `release` i `.agents/plugins/xgauntlet/skills/` samt `ADR-FORMAT.md`, `CODING_STANDARDS.md`, `CONTEXT.md` og `spec.md` med pædagogisk forklaring (Hvad, Hvorfor, Hvordan) og eksempler for Rust/Iced, TypeScript/React og Java.

---

## 🚫 Must NOT

- **Må IKKE** ændre i `crates/gauntlet-policy-engine/` eller `gauntlet_policy.wasm` (0% WASM-ændring).
- **Må IKKE** stramme de eksisterende fil-linjetals-lofter (`max_file_lines = 800`, `max_test_file_lines = 1000`) eller bryde monotoniske ratchets i `gauntlet.toml` (ADR 0010).
- **Må IKKE** påtvinge en rigid intern mappe- eller filstruktur indeni en slice (`features/<domain>/<slice>/`).
- **Må IKKE** bryde bagudkompatibiliteten for eksisterende `status: done` tasks i `tasks/`.
- **Må IKKE** tilføje eksterne tredjeparts-crates (jf. ADR 0004 Lean Dependency Governance).
- **Må IKKE** foretage remote publication (`git push`) eller destruktive git-kommandoer (jf. ADR 0003).

---

## 📝 Revisions

- 2026-11-07: Oprettet Task 057 efter arkitekturanalyse og Catch-22 audit.

---

## 🧪 Verifikation

- `cargo test -p xgauntlet-core --test slice_architecture_and_adr_lifecycle_test`
- `cargo test -p xgauntlet-core --test adr_validation_test`
- `cargo test -p xgauntlet-core --test architecture_gate_test`
- `cargo test -p xgauntlet-core --test release_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
