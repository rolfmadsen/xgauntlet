---
type: Task Package
title: "Task 041: Workspace Agnosticism & Production Readiness Triage"
description: "Sikre universel workspace-understøttelse i manifest-scanneren, fail-closed git checkpoint scoping, Claude Code og Codex gatekeeper-paritet samt integration af Definition of Enough i sovereign review skill"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-18T14:42:00Z" }
tags: [manifest, checkpoint, scoping, claude_code, codex, adapters, review, skills, production-readiness]
scope:
  - "crates/xgauntlet-core/src/features/evidence/manifest.rs"
  - "crates/xgauntlet-core/src/features/checkpoint/**"
  - "crates/xgauntlet-core/src/features/adapters/claude_code/**"
  - "crates/xgauntlet-core/src/features/adapters/codex/**"
  - ".agents/plugins/xgauntlet/skills/review/**"
test_refs:
  - "crates/xgauntlet-core/tests/production_triage_test.rs"
---

# Task 041: Workspace Agnosticism & Production Readiness Triage

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-18`  

## 🎯 Formål
Etablere de nødvendige pragmatiske forudsætninger for at xGauntlet kan afvikles som autoritativ verifikationsmotor og gatekeeper på tværs af fremmede kodebaser (herunder Kant, TypeScript monorepos og Go-projekter):

1. **Generisk Workspace Manifest-Scanning**: Sikre at `compute_workspace_manifest` scanner hele workspace-roden (fratrukket `EXCLUDED_DIRS` og `EXCLUDED_FILES`), således at mapper som `apps/`, `libs/`, `services/`, `cmd/`, `frontend/` osv. automatisk inkluderes i canonical multi-digests uden at kræve hardcodede whitelist-stier.
2. **Fail-Closed Git Checkpoint Scoping**: Beskytte Git-historikken mod forurening ved at verificere alle staged filer mod den aktive opgaves erklærede `scope` (`allowed_scopes`, `allowed_test_targets`) og tilladte governance-filer (`tasks/<id>.md`, `evidence.md`, `verification-report.json`, `.agents/**`) før `git commit` udføres under et TDD-checkpoint.
3. **Scaffolding Gatekeeper Paritet for Claude Code & Codex**: Udvide `ClaudeCodeAdapter::generate_settings_json` og `CodexAdapter::generate_hooks_json` til at scaffold'e både `PreToolUse` gatekeeper-hooks (`xgauntlet hook ...`) og `PostToolUse` telemetri-hooks (`xgauntlet telemetry ...`).
4. **Sovereign Review Skill Hardening**: Indarbejde en operationel "Definition of Enough"-sektion i `.agents/plugins/xgauntlet/skills/review/SKILL.md` med eksplicit trusselsmodel, afvisning af teoretiske fund, lukket verifikation (anti-moving-goalposts) og obligatorisk `STATUS: SHIP IT` stopkriterie.
5. **Repo Rengøring**: Fjerne overflødig Iced GUI-PDF fra xGauntlet-repoet og synkronisere versionsdeklarationer.

---

## 📋 Acceptance Criteria
- [ ] **AC1 (Generisk Workspace Manifest-Scanning)**: `compute_workspace_manifest` scanner automatisk hele workspace-roden rekursivt for filer og mapper (respekterer `EXCLUDED_DIRS` og `EXCLUDED_FILES`), således at ikke-standard mapper som `apps/` eller `libs/` altid registreres i manifestets `items` og `files`.
- [ ] **AC2 (Fail-Closed Checkpoint Scoping)**: `run_checkpoint` validerer `staged_files` fra `git diff --cached --name-only` mod den aktive opgaves `scope`. Hvis filer uden for det tilladte scope er blevet staget, afvises checkpointet øjeblikkeligt med `CheckpointError::ScopeViolation` (eller tilsvarende fail-closed fejl) og ingen commit udføres.
- [ ] **AC3 (Scaffolding Gatekeeper Paritet)**: `ClaudeCodeAdapter::generate_settings_json` og `CodexAdapter::generate_hooks_json` genererer konfigurationer med både `PreToolUse` (`xgauntlet hook claude_code` / `xgauntlet hook codex`) og `PostToolUse` (`xgauntlet telemetry --format ...`).
- [ ] **AC4 (Sovereign Review Skill Hardening)**: `.agents/plugins/xgauntlet/skills/review/SKILL.md` indeholder en dedikeret sektion med "Definition of Enough", trusselsmodel (stokastisk AI-assistent vs. ondsindet hacker), afvisningskriterier for teoretiske edge-cases og instruks om lukket verifikation med `STATUS: SHIP IT`.
- [ ] **AC5 (Repo Rengøring & Paritet)**: `docs/rust_iced/iced — A Cross-Platform GUI Library for Rust.pdf` er fjernet fra repoet, og versionsangivelser i `.agents/plugins/xgauntlet/plugin.json` og `policy.json` er opdateret.

---

## 🚫 Must NOT
- Må IKKE udføre remote publication handlinger (`git push`) jf. ADR 0003.
- Må IKKE udføre destruktive git handlinger (`git reset --hard` eller `git clean -f`).
- Må IKKE bryde eksisterende 286 enhedstests eller reducere verifikationsstabiliteten.
- Må IKKE tillade, at filer uden for den aktive opgaves scope forsegles i Git-historikken via `xgauntlet checkpoint`.

---

## 📝 Revisions
- 2026-10-18: Opgave initialiseret på baggrund af produktionsreview og triage af reelle stopklodser for generisk drift.

---

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 041`
- `cargo test -p xgauntlet-core --test production_triage_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
