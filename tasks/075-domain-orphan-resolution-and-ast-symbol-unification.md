---
type: Task Package
title: "Task 075: Domain Orphan Resolution & AST Symbol Unification"
description: "Eliminere de 13 resterende forældreløse domænemodeller i architecture.mermaid ved at forhindre test-mock skygning i AST-scanneren, opdele policy-domænemodeller i models.rs, integrere AttestationReport, og binde analyse-DTOer."
status: active
intent: bugfix
slice: topology
spec_ref: "specs/topology.md"
generated: { by: process:antigravity-task-init, at: "2026-11-08T22:15:00Z" }
tags: [topology, ast, mermaid, symbol-unification, domain-model, policy, evidence]
scope:
  - "specs/topology.md"
  - "tasks/075-domain-orphan-resolution-and-ast-symbol-unification.md"
  - "CONTEXT.md"
  - "crates/xgauntlet-core/src/features/topology/parser.rs"
  - "crates/xgauntlet-core/src/features/topology/mermaid.rs"
  - "crates/xgauntlet-core/src/features/policy/mod.rs"
  - "crates/xgauntlet-core/src/features/policy/models.rs"
  - "crates/xgauntlet-core/src/features/evidence/models.rs"
  - "crates/xgauntlet-core/src/features/evidence/attestation.rs"
  - "crates/xgauntlet-core/src/features/gauntlet/pipeline.rs"
  - "crates/xgauntlet-core/src/features/plugin/models.rs"
  - "crates/xgauntlet-core/src/features/plugin/installer.rs"
  - "crates/xgauntlet-core/src/features/topology/models.rs"
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
  - "architecture.mermaid"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
---

# Task 075: Domain Orphan Resolution & AST Symbol Unification

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Eliminere de 13 forældreløse typer (`TrustDecision`, `NormalizedToolCall`, `DiscoveredHarness`, `CapabilityRequest`, `BlastRadiusReport`, `AttestationStatus`, `EnforcementContext`, `TaskContract`, `PolicyDecision`, `DecisionVerdict`, `ToolActionType`, `AdapterHookVerdict`, `DoctorCategory`) i `architecture.mermaid` gennem 4 afgrænsede udbedringer:
1. **Symbol Table Shadowing Fix**: Beskytte AST-scannerens type-opslag mod at blive overskrevet af test-mock structs/enums (fx mock-`TaskContract` og mock-`DoctorCategory` i integrationstests).
2. **Policy Slice Domænestruktur**: Flytte policy-domænemodeller (`PolicyDecision`, `DecisionVerdict`, `CapabilityRequest`, `ToolActionType`, `EnforcementContext`) til en eksplicit `features/policy/models.rs`, så de genkendes af domænefilteret og etablerer kompositioner til `AdapterHookVerdict` og `NormalizedToolCall`.
3. **Attestation Report Domæne-Integration**: Integrere `AttestationReport` som domænemodel i `evidence/models.rs`, så `AttestationStatus` og `TrustDecision` forbindes til deres overordnede rapportaggregat.
4. **Analyse DTO Forbindelser**: Binde `BlastRadiusReport` og `DiscoveredHarness` til deres naturlige operationelle ejere (`TopologyGraph` og `PluginInstallReport`).

## 📋 Acceptance Criteria
- [x] **AC1 (Symbol Table Shadowing Elimination):**
  - [x] AST-scanneren i `parser.rs` prioriterer produktionsfiler (`src/`) og domænemodeller (`is_domain_model = true`) frem for test-filer (`tests/`), så mock-modeller aldrig skygger for ægte typer.
  - [x] `VerificationReport *-- TaskContract : task_contract` og `DoctorCheckItem *-- DoctorCategory : category` renderes automatisk i `architecture.mermaid`.
- [x] **AC2 (Policy Modeller & Hook/Adapter Forbindelse):**
  - [x] `features/policy/models.rs` definerer eller re-eksporterer de kanoniske policy-modeller.
  - [x] `PolicyDecision *-- DecisionVerdict : verdict` og `CapabilityRequest *-- ToolActionType : action_type` renderes.
  - [x] `AdapterHookVerdict *-- DecisionVerdict : decision` og `NormalizedToolCall *-- ToolActionType : action_type` renderes.
- [x] **AC3 (Attestation Forældre/Børn Relationer):**
  - [x] `AttestationReport` renderes i konceptuelt diagram med relationer til `AttestationStatus` og `TrustDecision`.
- [x] **AC4 (Analyse DTO Integration):**
  - [x] `BlastRadiusReport` og `DiscoveredHarness` er ikke længere isolerede øer i det konceptuelle klassediagram.
- [x] **AC5 (Multi-Layer Verifikation & Invariant Kontrol):**
  - [x] `cargo test` og `xgauntlet verify --task 075 --save` passerer uden fejl eller regressionsbrud.

## 🚫 Must NOT
- Må IKKE bryde bagudkompatibilitet for WASM policy runtime eller eksterne API-kald.
- Må IKKE ændre eksisterende JSON-serialisering i rapporter.
- Må IKKE indføre manuelle hardkodede diagramlinjer; diagrammet skal udledes 100% deterministisk fra AST.

## 📝 Revisions
- 2026-11-08: Oprettet opgavepakke med rodårsagsdiagnose af de 13 forældreløse typer.

## 🧪 Verifikation
- `cargo test --test mermaid_field_and_relation_extraction_test`
- `target/debug/xgauntlet verify --task 075 --save`
