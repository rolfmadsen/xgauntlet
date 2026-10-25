---
type: Task Package
title: "Task 029: Detect Composite TypeScript Project References in Stack Profile"
description: "Enable stack profiler and doctor to automatically detect composite tsconfig setups (files: [], references) and configure tsc -b instead of tsc --noEmit to prevent silent typecheck bypasses."
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-09-15T16:40:34Z" }
tags: [stack-profiler, typescript, vite, project-references, gauntlet]
scope: ["crates/xgauntlet-core/src/features/config/**", "crates/xgauntlet-core/src/features/doctor/**"]
test_refs: ["crates/xgauntlet-core/tests/stack_profiles_test.rs", "crates/xgauntlet-core/tests/doctor_test.rs"]
---

# Task 029: Detect Composite TypeScript Project References in Stack Profile

**Status**: `TODO`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-15`

## 🎯 Formål
Gøre xgauntlets Node/TypeScript stack profiler og `doctor` bevidst om "composite project references" (typisk for moderne Vite, Turborepo og Monorepo setups med `tsconfig.json` -> `files: []` og `references: [...]`). Når dette mønster opdages, skal xgauntlet automatisk konfigurere `tsc -b` (eller `tsc -p tsconfig.app.json --noEmit`) frem for `tsc --noEmit`, for at forhindre at typecheck-laget passerer i stilhed uden at tjekke nogen filer overhovedet.

Da konfigurationen deklareres centralt i `gauntlet.toml [layers.types]` og inspiceres via `xgauntlet doctor`, fungerer løsningen **100% ensartet på tværs af samtlige understøttede AI harnesses** (Google Antigravity, Claude Code, OpenAI Codex og Mistral Vibe) jf. ADR 0004.

## 📋 Acceptance Criteria
- [ ] **AC1 (Detection Engine)**: Stack profiler scanner rodens `tsconfig.json` for tilstedeværelse af `"references"` og tomme/manglende kilde-inclusions i roden.
- [ ] **AC2 (Profile Generation)**: `xgauntlet init` genererer automatisk `command = ["npx", "tsc", "-b"]` i `gauntlet.toml` [layers.types], når composite references detekteres.
- [ ] **AC3 (Doctor Warning)**: `xgauntlet doctor` advarer, hvis `gauntlet.toml` bruger `tsc --noEmit` i et repository med composite project references.
- [ ] **AC4 (Test Suite)**: Unit tests for composite tsconfig detection og profiler output.

## 🚫 Must NOT
- Må IKKE bryde eksisterende standard Node/TS projekter (hvor flad `tsconfig.json` anvendes).
- Må IKKE antage fast navngivning af under-konfigurationer (f.eks. `tsconfig.app.json`), men foretrække standarden `tsc -b`.
- Må IKKE foretage remote publication handlinger (`git push`).

## 📝 Revisions
- 2026-09-15: Task oprettet med detaljerede kriterier baseret på erfaringer fra knowledgegraphstudio.
- 2026-10-25: Tilføjet OKF v0.2 scope og test_refs (ADR 0008), samt ekspliciteret tværgående harness-paritet.

## 🧪 Verifikation
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-spec -t 029-detect-composite-typescript-references`

