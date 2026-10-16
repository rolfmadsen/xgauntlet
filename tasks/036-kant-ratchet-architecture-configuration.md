---
type: Task Package
title: "Task 036: Kant Declarative Ratchet Architecture Configuration"
description: "Etablering af deklarativ [architecture] konfiguration i Kants gauntlet.toml med forbud mod rå styling og ratchet spærrehage på app.rs og acceptance.rs"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-16T21:48:00Z" }
tags: [kant, architecture, ratchet, gauntlet-toml, linter, limits]
scope:
  - "gauntlet.toml"
test_refs:
  - "tests/architecture_test.rs"
---

# Task 036: Kant Declarative Ratchet Architecture Configuration

**Status**: `TODO`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-16`  

## 🎯 Formål
Etablere xGauntlets deklarative arkitektur-gatekeeper i Kant-repositoryet (`/home/rolfmadsen/Github/kant`), så forfaldet stoppes øjeblikkeligt:
1. Forbyde rå styling (`Color::from_rgb`, `Color::from_rgba`) uden om det kommende design tokens system.
2. Fastsætte en absolut spærrehage (`[[architecture.exemptions]]`) på `src/ui/app.rs` (11.520 linjer) og `tests/acceptance.rs` (12.181 linjer), så de fremadrettet KUN må falde i størrelse jf. Ratchet-modellen.
3. Integrere arkitektur-kontrollen som et obligatorisk lag i Kants verifikations-gauntlet.

## 📋 Acceptance Criteria
- [ ] **AC1 (Konfigurer [architecture] i gauntlet.toml)**: `gauntlet.toml` i Kant konfigureres med `max_file_lines = 800` og `max_test_file_lines = 1000`.
- [ ] **AC2 (Forbyd rå farvetokens)**: `forbidden_tokens = ["Color::from_rgb", "Color::from_rgba"]` deklareres i `gauntlet.toml`, så fremtidig UI-kode tvinges til at anvende type-sikre design tokens.
- [ ] **AC3 (Ratchet spærrehage på monolitterne)**: `[[architecture.exemptions]]` etableres for `src/ui/app.rs` (loft: 11.550 linjer) og `tests/acceptance.rs` (loft: 12.200 linjer) med eksplicitte faglige begrundelser.
- [ ] **AC4 (Architecture Verification Layer)**: Laget `architecture` (`command = ["xgauntlet", "check-architecture"]`) tilføjes til `[[layers]]` i `gauntlet.toml` som non-optional.
- [ ] **AC5 (Verifikation)**: Kørsel af `xgauntlet check-architecture` i Kant passerer fejlfrit med de etablerede spærrehager.

## 🚫 Must NOT
- Må IKKE hæve lofterne på `app.rs` eller `acceptance.rs` (spærrehagen må kun strammes, aldrig lempes).
- Må IKKE tillade nye filer over 800 linjer eller testfiler over 1.000 linjer.
- Må IKKE introducere eksterne netværks- eller dæmonafhængigheder.

## 📝 Revisions
- 2026-10-16: Task 036 oprettet til udrulning af Ratchet-arkitektur i Kant.

## 🧪 Verifikation
- `xgauntlet check-architecture` (i Kant workspace)
- `xgauntlet check-spec -t 036-kant-ratchet-architecture-configuration`
