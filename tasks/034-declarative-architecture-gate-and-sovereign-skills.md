---
type: Task Package
title: "Task 034: Declarative Architecture Gate & Sovereign Skills"
description: "Etablering af deklarativ [architecture] gatekeeper i gauntlet.toml og udrulning af xGauntlets 6 suveræne skills jf. ADR 0008"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-10-16T20:55:00Z" }
tags: [architecture, design-tokens, skills, gauntlet-toml, linter, ratchet, adr-0008]
scope:
  - "crates/xgauntlet-core/src/features/config/**"
  - "crates/xgauntlet-core/src/features/architecture/**"
  - "crates/xgauntlet-core/src/features/plugin/**"
  - "crates/xgauntlet-cli/src/**"
  - ".agents/plugins/xgauntlet/**"
test_refs:
  - "crates/xgauntlet-core/tests/architecture_gate_test.rs"
  - "crates/xgauntlet-core/tests/plugin_distribution_test.rs"
---

# Task 034: Declarative Architecture Gate & Sovereign Skills

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-16`  

## 🎯 Formål
Etablere xGauntlets deklarative arkitektur-gatekeeper og suveræne Agentic SDLC skills jf. ADR 0008 ved at:
1. Udvide `gauntlet.toml` med `[architecture]` konfiguration, der deklarerer `max_file_lines`, `max_test_file_lines`, faglige `exemptions`, `forbidden_tokens` (f.eks. rå RGB farver uden om design tokens) og `forbidden_imports`.
2. Implementere `run_architecture_check` i `xgauntlet-core` og CLI-subcommandet `xgauntlet check-architecture`.
3. Erstatte de 11 eksterne/Matt Pocock skills med xGauntlets 6 suveræne skills (`discover`, `spec`, `tdd`, `review`, `diagnose`, `release`), som håndhæver xGauntlets metodik uden fremmede dogmer.

## 📋 Acceptance Criteria
- [ ] **AC1 (Architecture Config i `gauntlet.toml`)**: `GauntletConfig` i `crates/xgauntlet-core/src/features/config/models.rs` udvides med `#[serde(default)] pub architecture: ArchitectureConfig` med understøttelse af `max_file_lines`, `max_test_file_lines`, `exemptions`, `forbidden_tokens` og `forbidden_imports`.
- [ ] **AC2 (Architecture Checker Engine)**: `crates/xgauntlet-core/src/features/architecture/` implementerer `run_architecture_check`, der scanner workspace-kildetræet, validerer linjetal mod grænser (med respekt for explicit begrundede undtagelser jf. Ratchet-modellen), og fanger forbudte tokens og imports.
- [ ] **AC3 (CLI Subcommand `check-architecture`)**: `xgauntlet check-architecture` implementeres i `xgauntlet-cli` med formateret terminal-output, der ved overtrædelser returnerer exit code 1 og handlingsanvisende hints.
- [ ] **AC4 (Gauntlet Layer Integration)**: Arkitektur-tjekket kan konfigureres og eksekveres som et verifikationslag i `xgauntlet verify`.
- [ ] **AC5 (Udrulning af 6 Suveræne Skills)**: De 11 eksterne skills i `.agents/plugins/xgauntlet/skills/` erstattes af de 6 suveræne skills: `discover`, `spec`, `tdd`, `review`, `diagnose`, `release`. `plugin.json` og `ALL_EMBEDDED_SKILLS` i `bundle.rs` opdateres tilsvarende.
- [ ] **AC6 (Regression & Verification Gauntlet)**: `architecture_gate_test.rs` og `plugin_distribution_test.rs` samt hele workspace passerer med 0 fejl og clippy clean.

## 🚫 Must NOT
- Må IKKE bryde bagudkompatibilitet for eksisterende `gauntlet.toml` filer uden `[architecture]` sektion (skal falde tilbage på sikre defaults).
- Må IKKE være en blind "brick wall" uden mulighed for velbegrundede, eksplicitte undtagelser (`exemptions`).
- Må IKKE tillade en undtaget fil at vokse ukontrolleret (Ratchet-invariaten).
- Må IKKE introducere baggrunds-dæmoner eller runtime socket-afhængigheder (Zero-Daemon).
- Må IKKE foretage remote publication handlinger (`git push`).

## 📝 Revisions
- 2026-10-16: Task 034 initialiseret jf. ADR 0008 for Arkitektur-Gate og Suveræne Skills.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test architecture_gate_test`
- `cargo test -p xgauntlet-core --test plugin_distribution_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 034-declarative-architecture-gate-and-sovereign-skills`
- `cargo run -p xgauntlet-cli -- check-architecture`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
