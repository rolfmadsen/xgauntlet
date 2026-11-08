---
type: Task Package
title: "Task 074: Domain Relationship Completeness & AST Trait Realization"
description: "Eliminere forældreløse domæne-typer i Mermaid-diagrammet ved at tilføje AST trait-realization (<|..), udrydde Primitive Obsession med stærke enums, og konsolidere TaskContract-modellen."
status: active
intent: enhancement
slice: topology
spec_ref: "specs/topology.md"
generated: { by: process:antigravity-task-init, at: "2026-11-08T21:40:00Z" }
tags: [topology, mermaid, ast, trait-realization, strong-types, domain-model]
scope:
  - "specs/topology.md"
  - "tasks/074-domain-relationship-completeness-and-trait-realization.md"
  - "crates/xgauntlet-core/src/features/topology/parser.rs"
  - "crates/xgauntlet-core/src/features/topology/mermaid.rs"
  - "crates/xgauntlet-core/src/features/adapters/models.rs"
  - "crates/xgauntlet-core/src/features/evidence/models.rs"
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
  - "architecture.mermaid"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
---

# Task 074: Domain Relationship Completeness & AST Trait Realization

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Fuldende domænemodellens relationelle integritet i `architecture.mermaid` ved at eliminere forældreløse typer (orphans) gennem tre mekaniske forbedringer:
1. **AST Trait Realization (`<|..`):** Udvid AST-scanneren i `parser.rs` til at registrere `impl <Trait> for <Struct>` og rendere realiseringspile (`Concrete ..|> Interface` / `Concrete <|.. Interface`) i Mermaid-diagrammet, så `HarnessAdapter` forbindes til alle fire adaptere.
2. **Udryddelse af Primitive Obsession:** Erstat rå `String`-felter med stærke domæne-enums i `AdapterHookVerdict` (`decision: DecisionVerdict`), `CheckSummary` (`status: CheckStatus`), og `VerificationReport` (`verdict: VerificationVerdict`, `execution_origin: ExecutionOrigin`), så kompositionspile automatisk etableres.
3. **Konsolidering af `TaskContract`:** Sikre at `TaskContract` bindes entydigt ind i `VerificationReport`, så redundante, uforbundne tvillinge-modeller elimineres.

## 📋 Acceptance Criteria
- [ ] **AC1 (AST Trait Realization):**
  - [ ] `parser.rs` scanner `impl <Trait> for <Struct>` blokke i Rust source og etablerer `EdgeType::Implements` kanter.
  - [ ] `mermaid.rs` renderer `EdgeType::Implements` som `Concrete ..|> Interface`.
  - [ ] `architecture.mermaid` viser relationer fra `AntigravityAdapter`, `ClaudeCodeAdapter`, `CodexAdapter`, `MistralAdapter` til `HarnessAdapter`.
- [ ] **AC2 (Primitive Obsession Elimination / Strong Enums):**
  - [ ] `AdapterHookVerdict.decision` typesættes stærkt som `DecisionVerdict`.
  - [ ] `CheckSummary.status` typesættes stærkt som `CheckStatus`.
  - [ ] `VerificationReport.verdict` typesættes stærkt som `VerificationVerdict`.
  - [ ] `VerificationReport.execution_origin` typesættes stærkt som `ExecutionOrigin`.
  - [ ] AST-scanneren etablerer automatisk `*--` relationer for samtlige disse felter i `architecture.mermaid`.
- [ ] **AC3 (Konsolideret TaskContract):**
  - [ ] `TaskContract` har entydig tilknytning i domænemodellen uden at efterlade isolerede modeller.
- [ ] **AC4 (Multi-Layer Verifikation & Regression):**
  - [ ] Eksisterende tests opdateres og udvides til at verificere `..|>` og de nye stærke kompositionsrelationer.
  - [ ] `architecture.mermaid` regenereres og demonstrerer en fuldt sammenhængende model.
  - [ ] `xgauntlet verify --task 074 --save` passerer med `Self-mutation status: CLEAN`.

## 🚫 Must NOT
- Må IKKE bryde JSON-serialisering / API-kompatibilitet for eksisterende rapporter (`verdict`, `status` og `decision` skal serialiseres uændret).
- Må IKKE introducere baggrunds-daemons eller runtime ambient authority.
- Må IKKE bryde `cargo check`, `cargo test`, eller `check-spec`.
