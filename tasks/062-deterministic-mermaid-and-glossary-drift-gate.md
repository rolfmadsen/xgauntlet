---
type: Task Package
title: "Task 062: Deterministic Mermaid Class Diagram & Glossary Drift Gate"
description: "Implementere deterministisk Mermaid klasse- og informationsdiagram-eksport samt to-vejs glossary drift-validering mellem kildekodens domænemodeller og CONTEXT.md."
status: active
intent: new_feature
generated: { by: process:antigravity-task-init, at: "2026-11-08T11:32:00Z" }
tags: [topology, ast, mermaid, domain-model, glossary, check-spec, okf, readme]
scope:
  - "crates/xgauntlet-core/src/features/topology/mod.rs"
  - "crates/xgauntlet-core/src/features/topology/models.rs"
  - "crates/xgauntlet-core/src/features/topology/parser.rs"
  - "crates/xgauntlet-core/src/features/topology/mermaid.rs"
  - "crates/xgauntlet-core/src/features/tasks/validator.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/checkers.rs"
  - "README.md"
  - "README_OLD.md"
  - "tasks/062-deterministic-mermaid-and-glossary-drift-gate.md"
test_refs:
  - "crates/xgauntlet-core/tests/topology_engine_test.rs"
  - "crates/xgauntlet-core/tests/tasks_engine_test.rs"
---

# Task 062: Deterministic Mermaid Class Diagram & Glossary Drift Gate

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Etablere en deterministisk, sub-5ms AST-baseret eksport af kodebasens domænemodeller til gyldige Mermaid klasse- og informationsdiagrammer (`xgauntlet topology export --format mermaid`) samt en mekanisk to-vejs glossary drift-gate (`xgauntlet check-spec --context`), der validerer at alle domænemodeller i `src/features/**/models.rs` er defineret jf. Aristoteles' formel i `CONTEXT.md` og blokerer teknisk støj. Efter endt implementering opdateres `README.md` tilsvarende for at dokumentere de nye funktioner.

## 📋 Acceptance Criteria
- [ ] **Deterministisk AST Mermaid Eksport (`xgauntlet topology export --format mermaid`)**:
  - [ ] Traverserer og udtrækker structs, enums, traits/interfaces samt deres felter, varianter og indbyrdes relationer (`*--`, `o--`, `<|--`) fra domænemodeller i `src/features/**/models.rs` (eller typer med `/// @domain`).
  - [ ] Adskiller konceptuelle domænemodeller fra tekniske infrastrukturelementer (`engine.rs`, `git.rs`, `render.rs`, serializers som `$schema`), så diagrammet forbliver en ren forretnings- og informationsmodel.
  - [ ] Emitterer 100% syntaktisk validt Mermaid `classDiagram` output med sorterede entiteter for deterministisk reproducerbarhed.
- [ ] **To-vejs Glossary Drift Gate (`xgauntlet check-spec --context`)**:
  - [ ] **Orphan Model Detection (Code -> Context)**: Emitterer en blokerende `DiagnosticFinding`, hvis en `pub struct` eller `pub enum` i en `models.rs`-fil mangler en tilsvarende Aristoteles-definition i `CONTEXT.md`.
  - [ ] **Phantom Term Detection (Context -> Code)**: Advarer hvis et domænebegreb i `CONTEXT.md` er defineret som en datamodel-entitet, men ikke eksisterer i koden.
  - [ ] **Synonym Guard**: Validerer at felter og typer ikke anvender forbudte synonymer defineret under `_Avoid_:` i `CONTEXT.md`.
- [ ] **Dokumentations-opdatering**:
  - [ ] Opdatere `README.md` (og eventuelt `README2.md`) med CLI-eksempler, output-formater og anvendelse af `xgauntlet topology export --format mermaid` samt `xgauntlet check-spec --context`.

## 🚫 Must NOT
- Må IKKE introducere eksterne baggrundsprocesser, dæmoner eller netværkskald (Zero-Daemon Invariant).
- Må IKKE anvende LLM-kald til AST-parsing eller Mermaid-generering (skal køre 100% deterministisk i native Rust).
- Må IKKE udføre remote publication (`git push`) jf. ADR 0003.
- Må IKKE tillade tekniske serializer- eller memory-detaljer at forurene `CONTEXT.md`.

## 📝 Revisions
- 2026-11-08: Oprettet opgavepakke efter arkitektonisk sparring om deterministisk Mermaid-ekstraktion og glossary drift.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test topology_engine_test`
- `cargo test -p xgauntlet-core --test tasks_engine_test`
- `cargo run -p xgauntlet-cli -- topology export --format mermaid`
- `cargo run -p xgauntlet-cli -- check-spec --context`
- `cargo run -p xgauntlet-cli -- check-spec -t 062`
