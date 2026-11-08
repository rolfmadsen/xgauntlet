---
type: Task Package
title: "Task 063: Architecture Baseline Audit Terminology"
description: "Erstatte den klodsede metafor 'røntgenbillede' med den etablerede fagterm 'Architecture Baseline Audit' i README, CLI README og regressionstest."
status: active
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-08T11:58:00Z" }
tags: [documentation, terminology, architecture, baseline-audit]
scope:
  - "README.md"
  - "packages/cli/README.md"
  - "crates/xgauntlet-core/tests/architecture_scaffold_template_test.rs"
  - "tasks/063-architecture-baseline-audit-terminology.md"
test_refs:
  - "crates/xgauntlet-core/tests/architecture_scaffold_template_test.rs"
---

# Task 063: Architecture Baseline Audit Terminology

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-08`

## 🎯 Formål
Erstatte den uhensigtsmæssige fordanskning "risikofrit røntgenbillede" med den professionelle softwarearkitektur-fagterm "Architecture Baseline Audit" i `README.md` og `packages/cli/README.md`, samt opdatere test-assertionen i `architecture_scaffold_template_test.rs` så den afspejler den opdaterede terminologi.

## 📋 Acceptance Criteria
- [x] Erstatte overskriften og omtalen af "risikofrit Røntgenbillede (Audit Baseline)" i `README.md` med "risikofri Architecture Baseline Audit (Status Quo Snapshot)".
- [x] Erstatte overskriften og omtalen i `packages/cli/README.md` tilsvarende for fuld konsistens.
- [x] Opdatere `crates/xgauntlet-core/tests/architecture_scaffold_template_test.rs` til at validere forekomsten af "Architecture Baseline Audit" eller "Baseline Audit" i stedet for strengt at kræve "Røntgenbillede".
- [x] Køre `cargo test -p xgauntlet-core --test architecture_scaffold_template_test` og verificere at testen passerer.

## 🚫 Must NOT
- Må IKKE fjerne forklaringen af trinvis brownfield-migrering eller de 4 migrationstrin jf. ADR 0012.
- Må IKKE introducere regressionsfejl i eksisterende tests.
- Må IKKE foretage remote publication (`git push`) jf. ADR 0003.

## 📝 Revisions
- 2026-11-08: Oprettet opgavepakke efter brugerfeedback om forbedring af faglig terminologi.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test architecture_scaffold_template_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 063`
