---
type: Task Package
title: "Task 038: Antigravity Hook Schema Compliance and Non-Destructive Scaffolding"
description: "Tilpas PreInvocation til officielt Antigravity hook-schema og implementer fejlsikker, ikke-destruktiv merge i scaffold_hooks"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:07:00Z" }
tags: [antigravity, hooks, adapter, schema, non-destructive, scaffolding]
scope:
  - "crates/xgauntlet-core/src/features/adapters/antigravity/**"
  - ".agents/hooks.json"
test_refs:
  - "crates/xgauntlet-core/tests/harness_adapters_test.rs"
---

# Task 038: Antigravity Hook Schema Compliance and Non-Destructive Scaffolding

**Status**: `TODO`  
**Intent**: `🐛 BUG FIX`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Sikre at `PreInvocation` konfigurationen genereret af Antigravity-adapteren følger det officielle Google Antigravity hook-schema (handlers direkte i eventets liste uden unødig `matcher`), og garantere at scaffolding under ingen omstændigheder overskriver, korrumperer eller clobberer eksisterende brugerkonfigurationer i `.agents/hooks.json`.

## 📋 Acceptance Criteria
- [ ] **AC1 (PreInvocation Schema Compliance)**: `generate_hooks_json` genererer `PreInvocation` i overensstemmelse med det officielle schema (direkte array af hook handlers uden `matcher`). `matcher` reserveres strengt til `PreToolUse` og `PostToolUse`.
- [ ] **AC2 (Fejlsikker Scaffolding mod Korrupt JSON)**: Hvis eksisterende `.agents/hooks.json` indeholder syntaktisk ugyldig JSON, må `scaffold_hooks()` ALDRIG overskrive filen med defaults via `.ok()`. Den skal afbryde med en struktureret fejl (`ErrorKind::InvalidData`) og efterlade den oprindelige fil 100% intakt.
- [ ] **AC3 (Sikker Group Key Isolering)**: `generate_hooks_json` må ikke overtage eller kapre en urelateret fremmed nøgle (`first_key`) i roden af `hooks.json`. Hvis `xgauntlet` ikke findes, oprettes den eksplicit som sit eget dedikerede objekt uden at modificere andre grupper.
- [ ] **AC4 (Regressionstest Suite)**: Testsuiten beviser merge-adfærd i tre distinkte scenarier: (1) ikke-eksisterende fil, (2) gyldig eksisterende fil med fremmede hooks, (3) korrupt/ugyldig eksisterende fil (skal forblive uændret).

## 🚫 Must NOT
- Må IKKE overskrive eller slette eksisterende brugerkonfiguration ved fejlfortolkning af JSON.
- Må IKKE injecte `matcher` i events der ikke understøtter matcher i Antigravity-specifikationen.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P2 fund).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 038`
