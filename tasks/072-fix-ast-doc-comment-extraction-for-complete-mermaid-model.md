---
type: Task Package
title: "Task 072: Fix AST Doc-Comment Parsing for Complete Mermaid Data Model"
description: "Rette linjeskifts-håndtering i extract_brace_body så doc-kommentarer ikke kasserer struct-felter og enum-varianter, og genskabe den fulde datamodel i architecture.mermaid."
status: done
intent: bug_fix
slice: topology
spec_ref: "specs/topology.md"
generated: { by: process:antigravity-task-init, at: "2026-11-08T21:02:00Z" }
tags: [topology, ast, bugfix, doc-comments, mermaid, data-model]
scope:
  - "specs/topology.md"
  - "tasks/072-fix-ast-doc-comment-extraction-for-complete-mermaid-model.md"
  - "crates/xgauntlet-core/src/features/topology/rust_ast.rs"
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
  - "architecture.mermaid"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
---

# Task 072: Fix AST Doc-Comment Parsing for Complete Mermaid Data Model

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Løse defekten i `extract_brace_body` i `crates/xgauntlet-core/src/features/topology/rust_ast.rs`, hvor krop-linjer mellem tuborg-klammer `{ ... }` blev samlet med mellemrum i stedet for linjeskift `\n`. Denne fejl medførte, at `parse_struct_fields` og `parse_enum_variants` ved mødet med den første `///`-doc-kommentar klippede resten af struct- eller enum-kroppen væk, hvilket resulterede i tomme klasser `{}` og manglende relationer for doc-kommenterede domænemodeller (såsom `DoctorCheckItem`, `DoctorCategory`, `ArchitectureFindingCategory` m.fl.) i `architecture.mermaid`.

Efter rettelsen skal `architecture.mermaid` regenereres, så samtlige domænemodeller fremstår med deres reelle felter, enum-varianter og indbyrdes kompositionsrelationer.

## 📋 Acceptance Criteria
- [x] **AC1 (Bevaring af linjeskift i `extract_brace_body`):**
  - [x] `extract_brace_body` i `rust_ast.rs` sammenkæder udtrukne linjer med `\n` i stedet for `' '`.
- [x] **AC2 (Robust linje-for-linje kommentar-stripping):**
  - [x] `parse_struct_fields` og `parse_enum_variants` fjerner kommentarer linje for linje uden at klippe efterfølgende felter/varianter af.
- [x] **AC3 (Fuld ekstraktion af felter, varianter og relationer):**
  - [x] `DoctorCategory` udtrækker alle 6 varianter (`Host`, `Git`, `Governance`, `Toolchains`, `Engine`, `Harnesses`).
  - [x] `DoctorCheckItem` udtrækker samtlige felter, herunder `category: DoctorCategory` og `status: DoctorCheckStatus`.
  - [x] `DoctorCheckItem *-- DoctorCategory : category` og `DoctorCheckItem *-- DoctorCheckStatus : status` genereres som relationer i `architecture.mermaid`.
- [x] **AC4 (Regressionstest & Multi-Layer Verifikation):**
  - [x] En regressionstest i `crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs` beviser at doc-kommenterede modeller udtrækkes komplet med felter og varianter.
  - [x] `target/debug/xgauntlet verify --task 072 --save` kører rent uden selvmutation, og `architecture.mermaid` opdateres med den fulde datamodel.

## 🚫 Must NOT
- Må IKKE ændre eksisterende relationstyper eller indføre eksterne parsere/dæmoner (Zero-Daemon Invariant).
- Må IKKE bryde `cargo fmt`, `clippy` eller eksisterende unit- og integrationstests.
- Må IKKE foretage remote git-handlinger (`git push`) jf. ADR 0003.

## 📝 Revisions
- 2026-11-08: Initialiseret opgavepakke efter afklaring af manglende relationer i `architecture.mermaid`.

## 🧪 Verifikation
```bash
cargo test -p xgauntlet-core --test mermaid_field_and_relation_extraction_test
target/debug/xgauntlet verify --task 072 --save
```
