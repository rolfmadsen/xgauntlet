---
type: Task Package
title: "Task 073: Conceptual Domain Model Filtering and Topology Cleanup"
description: "Rense det konceptuelle Mermaid-diagram for tekniske hjælpeklasser (fejl-enums, CLI-options og runtime-parametre) så architecture.mermaid udelukkende fremviser den forbundne forretningsdomænemodel."
status: active
intent: enhancement
slice: topology
spec_ref: "specs/topology.md"
generated: { by: process:antigravity-task-init, at: "2026-11-08T21:25:00Z" }
tags: [topology, mermaid, conceptual-model, domain-model, filtering, cleanup]
scope:
  - "specs/topology.md"
  - "tasks/073-conceptual-domain-model-filtering-and-cleanup.md"
  - "crates/xgauntlet-core/src/features/topology/mermaid.rs"
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
  - "architecture.mermaid"
  - "verification-report.json"
  - "evidence.md"
test_refs:
  - "crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs"
---

# Task 073: Conceptual Domain Model Filtering and Topology Cleanup

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Etablere en præcis skillelinje i `crates/xgauntlet-core/src/features/topology/mermaid.rs` mellem den konceptuelle forretningsdomænemodel (`MermaidLevel::Conceptual`) og den fulde tekniske AST (`MermaidLevel::Detailed`). 

I `Conceptual`-niveauet skal teknisk stillads (fejl-enums som `*Error`/`*Exception`, CLI-argumenter som `*Options`/`*Args` samt runtime-parametre som `Direction` og `SemverBump`) automatisk filtreres fra. Derved elimineres de 19 isolerede, uforbundne kasser, så `architecture.mermaid` udelukkende fremviser de reelle forretningsaggregater og entiteter forankret i `CONTEXT.md` med deres indbyrdes kompositionsrelationer.

Samtidig ryddes overflødig, redundant label-inferens op i `mermaid.rs`, og diagrammet forsynes med en eksplicit konceptuel domæne-header.

## 📋 Acceptance Criteria
- [ ] **AC1 (Konceptuel Domæne-støjfiltrering i `mermaid.rs`):**
  - [ ] Når `level == MermaidLevel::Conceptual`, udelades typer, hvis navn ender på `Error`, `Exception`, `Options`, `Args` eller er runtime-flagene `Direction`, `SemverBump`, `PlatformTarget`.
  - [ ] Når `level == MermaidLevel::Detailed`, medtages samtlige deklarerede typer fortsat ufiltreret.
- [ ] **AC2 (Oprydning af Redundant Label-inferens i `mermaid.rs`):**
  - [ ] Fjern redundant fallback-kodesøgning efter feltnavne i `mermaid.rs`, da 3. pass i `parser.rs` allerede beriger alle `DependsOn`-kanter med deterministisk `field_name`-metadata.
- [ ] **AC3 (Eksplicit Konceptuel Header i Mermaid Eksport):**
  - [ ] `export_mermaid_class_diagram` indsætter en indledende kommentarheader, der deklarerer diagrammet som en konceptuel domænemodel forankret i forretningssproget.
- [ ] **AC4 (Regressionstest & Multi-Layer Verifikation):**
  - [ ] Ny unit-test i `crates/xgauntlet-core/tests/mermaid_field_and_relation_extraction_test.rs` beviser at `DoctorError` og `DoctorOptions` filtreres fra i `Conceptual`, men bevares i `Detailed`.
  - [ ] `architecture.mermaid` regenereres uden de 19 tekniske kasser, og `target/debug/xgauntlet verify --task 073 --save` passerer med `Self-mutation status: CLEAN`.

## 🚫 Must NOT
- Må IKKE fjerne adgangen til at inspicere den fulde rå AST via `MermaidLevel::Detailed`.
- Må IKKE bryde eksisterende topology traversal, blast-radius eller CLI eksport.
- Må IKKE udføre remote git-handlinger (`git push`) jf. ADR 0003.

## 📝 Revisions
- 2026-11-08: Initialiseret opgavepakke til oprydning og konceptuel domænefiltrering.

## 🧪 Verifikation
```bash
cargo test -p xgauntlet-core --test mermaid_field_and_relation_extraction_test
target/debug/xgauntlet verify --task 073 --save
```
