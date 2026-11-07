---
type: Task Package
title: "Task 059: Architecture Governance Documentation & Scaffold Template Hardening"
description: "Dokumenterer arkitektur-gates (Gate A/B/C) og Vertical Slice Architecture pædagogisk i README.md og packages/cli/README.md, samt udvider scaffoldet gauntlet.toml template med veldokumenterede eksempler på lag og slice-isolation."
status: active
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-07T21:06:00Z" }
tags: [architecture, templates, documentation, vsa, layers, brownfield, greenfield]
scope:
  - "crates/xgauntlet-core/src/features/scaffold/**"
  - "crates/xgauntlet-core/src/features/config/**"
  - "crates/xgauntlet-core/tests/**"
  - "README.md"
  - "packages/cli/README.md"
  - "tasks/059-architecture-governance-docs-and-scaffold-template.md"
test_refs:
  - "crates/xgauntlet-core/tests/architecture_scaffold_template_test.rs"
---

# Task 059: Architecture Governance Documentation & Scaffold Template Hardening

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-07`

---

## 🎯 Formål & Baggrund

Med introduktionen af de deterministiske arkitektur-gates i Task 057 / ADR 0012 (`LayerDirectionViolation`, `CrossSliceImportViolation`, `ForbiddenCoreImportViolation`) har xGauntlet fået fuld støtte for Vertical Slice Architecture på tværs af Rust, TypeScript/React, Java, Python og Go.

For at brugere og AI-agenter kan udnytte dette optimalt, løser denne opgave to centrale behov:
1. **Pædagogisk Dokumentation i README:**
   Forklare hvordan xGauntlet by default kører i skånsom "Safe Mode" for brownfield-projekter (hvor lag og slice-isolation er inaktive for at undgå brud i eksisterende kode), hvordan greenfield-projekter kan aktivere den fulde Vertical Slice-model, og hvordan AI-agenten automatisk håndterer oprettelse af `domain -> slice -> task` uden manuel brugerindgriben.
2. **Scaffold Template Hardening (`gauntlet.toml`):**
   Opdatere `xgauntlet init` template-genereringen, så den nyskabte `gauntlet.toml` indeholder veldokumenterede, udkommenterede eksempler på `layers`, `slice_root`, `isolate_slices` og `forbid_core_imports`, så konfigurationen er selvforklarende og let at slå til i greenfield-projekter.

---

## 📋 Acceptance Criteria

- [x] **AC1 — Template Hardening i `xgauntlet init`:**
  - `render_gauntlet_toml` i `crates/xgauntlet-core/src/features/scaffold/templates.rs` (og/eller `render_toml`) skal inkludere udkommenterede, pædagogiske instruktioner og eksempler for `layers`, `slice_root`, `isolate_slices` og `forbid_core_imports`.
- [x] **AC2 — Pædagogisk Arkitektur-Dokumentation i README:**
  - Opdater både `README.md` (GitHub) og `packages/cli/README.md` (npm) med et dedikeret afsnit under `check-architecture`:
    - Skånsom Safe-Mode som default for brownfield.
    - Fuld Vertical Slice Architecture for greenfield (Gate A, B og C).
    - AI-drevet slice-lifecycle (hvordan LLM'en opretter `specs/<domain>/<slice>.md` og `tasks/` ud fra almindeligt menneskesprog).
    - Trinvis brownfield-migrering via `[[architecture.exemptions]]` med `allow_imports`.
- [x] **AC3 — Regressions- og Template-Test:**
  - Opret `crates/xgauntlet-core/tests/architecture_scaffold_template_test.rs`, der beviser:
    - At `render_gauntlet_toml` genererer de pædagogiske eksempler for VSA og lag.
    - At den genererede template er gyldig syntax, som kan parses rent af `parse_toml` uden fejl.
    - At README-filerne dokumenterer de 3 gates og safe-mode.

---

## 🚫 Must NOT

- Må IKKE ændre standardadfærd for eksisterende `gauntlet.toml`-filer (bagudkompatibilitet for brownfield bevares 100%).
- Må IKKE tvangsaktivere `layers` eller `isolate_slices` automatisk uden eksplicit konfiguration.
- Må IKKE udføre remote publication handlinger (`git push`) jf. ADR 0003.

---

## 📝 Revisions

- 2026-11-07: Oprettet Task 059 for dokumentation af arkitektur-gates og template-forbedring.

---

## 🧪 Verifikation

- `cargo run -p xgauntlet-cli -- check-spec -t 059`
- `cargo test --test architecture_scaffold_template_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
