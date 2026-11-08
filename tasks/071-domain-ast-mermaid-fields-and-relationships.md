---
type: Task Package
title: "Task 071: Domain AST Field & Relationship Extraction for Mermaid"
description: "Eksekvere deterministisk ekstraktion af felter, varianter og relationer mellem domænemodeller i topology-scanneren til berigelse af architecture.mermaid."
status: active
intent: new_feature
slice: topology
spec_ref: "specs/topology.md"
generated: { by: process:antigravity-task-init, at: "2026-11-08T19:50:00Z" }
tags: [topology, ast, mermaid, class-diagram, domain-models, relations]
scope:
  - "specs/topology.md"
  - "tasks/070-release-v0-13-2.md"
  - "tasks/071-domain-ast-mermaid-fields-and-relationships.md"
  - "crates/xgauntlet-core/src/features/topology/**"
  - "crates/xgauntlet-core/tests/**"
  - "architecture.mermaid"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
---

# Task 071: Domain AST Field & Relationship Extraction for Mermaid

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Etablere deterministisk, sub-5ms ekstraktion af struct-felter, enum-varianter og indbyrdes type-relationer mellem forretningsmodeller i `crates/xgauntlet-core/src/features/**/models.rs`, så det automatisk genererede `architecture.mermaid` viser et meningsfuldt og præcist UML-klassediagram med felter og relationer (`*--` / `-->`) frem for tomme isolerede klasser.

## 📋 Acceptance Criteria
- [x] **AC1 (Rust AST Block & Field Parsing):**
  - [x] `parse_rust_file` i `crates/xgauntlet-core/src/features/topology/parser.rs` scanner struct-kroppe `{ ... }` for felter og gemmer dem kommasepareret i nodens `fields`-metadata (`felt: Type`).
  - [x] `parse_rust_file` scanner enum-kroppe `{ ... }` for varianter og gemmer dem i nodens `variants`-metadata (`VariantA, VariantB`).
- [x] **AC2 (Inter-Model Relationship Extraction):**
  - [x] Parseren detekterer felter, der refererer til andre kendte domæneentiteter (direkte `Type`, `Option<Type>`, `Vec<Type>`), og opretter en `TopologyEdge` med kildemodel, målmodel og feltnavn i metadata.
  - [x] `export_mermaid_class_diagram` i `mermaid.rs` renderer disse kanter med det reelle feltnavn (`Source *-- Target : field_name`) og fjerner den tidligere hårde stub `: status`.
- [x] **AC3 (Conceptual Domain Filtering):**
  - [x] I `Conceptual`-niveau (`domain_only = true`) begrænses visningen til typer fra `models.rs` eller med `is_domain_model = true`, så tekniske infrastrukturelementer ikke genererer tomme urelaterede kasser.
- [x] **AC4 (Multi-Layer Verifikation & Diagram Regenerering):**
  - [x] En ny isoleret accepttest `mermaid_field_and_relation_extraction_test.rs` bekræfter ekstraktion af felter, varianter og relationer.
  - [x] `xgauntlet verify --task 071 --save` regenererer [architecture.mermaid](file:///home/rolfmadsen/Github/xgauntlet/architecture.mermaid) uden selvmutationsfejl.

## 🚫 Must NOT
- Må IKKE introducere tunge eksterne AST-dæmoner eller LLM-kald (Zero-Daemon Invariant jf. sub-5ms kravet).
- Må IKKE bryde `verify_self_mutation` eller eksisterende topology- og governance-tests.
- Må IKKE udføre remote publication (`git push`) jf. ADR 0003.

## 📝 Revisions
- 2026-11-08: Oprettet opgavepakke efter arkitektursparring om manglende relationer og felter i `architecture.mermaid`.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test mermaid_field_and_relation_extraction_test`
- `cargo test -p xgauntlet-core --test mermaid_and_glossary_drift_test`
- `cargo run -p xgauntlet-cli -- verify --task 071 --save`
