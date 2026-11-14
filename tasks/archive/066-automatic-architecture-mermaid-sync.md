---
type: Task Package
title: "Task 066: Automatic Architecture Mermaid Diagram Synchronization"
description: "Automatisk generering og synkronisering af architecture.mermaid ved verify --save samt deklarativ konfiguration i gauntlet.toml og linking i spec.md."
status: done
intent: new_feature
generated: { by: process:antigravity-task-init, at: "2026-11-08T12:45:00Z" }
tags: [topology, mermaid, architecture, evidence, gauntlet, config, spec]
scope:
  - "crates/xgauntlet-core/src/features/config/models.rs"
  - "crates/xgauntlet-core/src/features/config/loader.rs"
  - "crates/xgauntlet-core/src/features/topology/parser.rs"
  - "crates/xgauntlet-core/src/features/evidence/manifest.rs"
  - "crates/xgauntlet-core/src/features/evidence/report.rs"
  - "crates/xgauntlet-core/src/features/gauntlet/pipeline.rs"
  - "crates/xgauntlet-core/src/features/checkpoint/engine.rs"
  - "crates/gauntlet-policy-engine/src/lib.rs"
  - "crates/xgauntlet-core/tests/mermaid_auto_sync_test.rs"
  - "gauntlet.toml"
  - "spec.md"
  - "architecture.mermaid"
  - "tasks/066-automatic-architecture-mermaid-sync.md"
test_refs:
  - "crates/xgauntlet-core/tests/mermaid_auto_sync_test.rs"
---

# Task 066: Automatic Architecture Mermaid Diagram Synchronization

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Etablere automatisk generering og persistens af et deterministisk Mermaid-klassediagram (`architecture.mermaid`) ved hver fuldført verifikation med gemt evidens (`xgauntlet verify --save`), så projektets visuelle og maskinlæsbare arkitektur- og domænemodel altid er opdateret i git-arbejdstræet uden manuelle omdirigeringer. Filen konfigureres deklarativt under `[paths] diagram_file` i `gauntlet.toml` og refereres med et direkte link fra `spec.md`.

## 📋 Acceptance Criteria
- [x] **Deklarativ Konfiguration (`gauntlet.toml`)**:
  - [x] `PathsConfig` udvides med `diagram_file: String` (default `"architecture.mermaid"`).
  - [x] Hvis `diagram_file` er sat til en tom streng, springes automatisk diagram-generering over.
- [x] **Automatisk Persistens under `verify --save`**:
  - [x] Ved kørsel af verifikationspipelinen med `save_evidence = true` genereres det deterministiske Mermaid `classDiagram` via AST-traverseringen fra Task 062 og skrives til den konfigurerede `diagram_file`.
  - [x] Filen genereres uden at udløse selvmutation (`verify_self_mutation`), og tilføjes til `EXCLUDED_FILES` i workspace-manifestet.
- [x] **Scope- og Policy-Godkendelse**:
  - [x] Checkpoint-motorens scope-validering (`validate_staged_files_scope`) tillader `architecture.mermaid` som en gyldig governance-artefakt.
  - [x] Policy-engine tillader skrivning til `architecture.mermaid` analogt med `spec.md` og `evidence.md`.
- [x] **Specifikation & Dokumentation**:
  - [x] `spec.md` indeholder et direkte, klikbart link til `architecture.mermaid`.
  - [x] Repositoryets egen `architecture.mermaid` genereres og forsegles.

## 🚫 Must NOT
- Må IKKE udføre remote publication (`git push`) jf. ADR 0003.
- Må IKKE bryde selvmutations-invarianterne (ADR 0005).
- Må IKKE fejle, hvis en workspace mangler domænemodeller (skal generere et gyldigt tomt eller minimalt classDiagram).

## 📝 Revisions
- 2026-11-08: Oprettet opgavepakke efter sparring om automatisk diagram-persistens.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test mermaid_auto_sync_test`
- `cargo run -p xgauntlet-cli -- verify --task 066 --save`
