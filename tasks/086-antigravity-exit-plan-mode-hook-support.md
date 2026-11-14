---
type: Task Package
title: "Task 086: Antigravity exit_plan_mode Hook Whitelist Support"
description: "Klassificer Antigravitys plan-mode kontrolværktøj ('exit_plan_mode') som sikker ikke-muterende handling i AntigravityAdapter::normalize_tool_call i stedet for ToolActionType::Other."
status: draft
intent: bug
generated: { by: process:antigravity-task-init, at: "2026-11-14T18:44:00Z" }
tags: [antigravity, adapter, hooks, bugfix, fast-track]
scope:
  - "crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs"
test_refs:
  - "crates/xgauntlet-core/tests/antigravity_adapter_test.rs"
---

# Task 086: Antigravity `exit_plan_mode` Hook Whitelist Support

**Status**: `DRAFT`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-14`

---

## 🎯 Formål

Udskilt fra det tidligere udkast Task 080 (hvor Fast-Track DX-delene blev samlet i [Task 056](056-simplify-fast-track-and-retire-worktree-inverse-proof.md)).

Når Orchestrator- eller Plan-modeller i Google Antigravity kalder `exit_plan_mode`, klassificerer `AntigravityAdapter::normalize_tool_call` aktuelt værktøjet som `ToolActionType::Other` i stedet for en sikker ikke-muterende kontrolhandling (`ToolActionType::ReadFile`), hvilket blokerer overgangen ud af plan-mode fail-closed i `PreInvocation`.

*Bemærk:* Med kun 1 kildefil i `scope` og `intent: bug` er denne opgave egnet som den første praktiske verifikation af det nye 2-trins Fast-Track bugfix-flow efter Task 056.

---

## 📋 Acceptance Criteria

- [ ] **AC1 — Whitelist `exit_plan_mode` i AntigravityAdapter:**
  - `AntigravityAdapter::normalize_tool_call` i `crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs` klassificerer `"exit_plan_mode"` som `ToolActionType::ReadFile` (sikker ikke-muterende kontrolhandling) i stedet for `ToolActionType::Other`.
- [ ] **AC2 — Regressionstest i `antigravity_adapter_test.rs`:**
  - `crates/xgauntlet-core/tests/antigravity_adapter_test.rs` verificerer, at `"exit_plan_mode"` normaliseres til `ToolActionType::ReadFile` og tillades af hook-politikken uden at lempe ukendte `Other`-værktøjer.

---

## 🚫 Must NOT

- **Må IKKE** lempe fail-closed politikken for ukendte, muterende eller shell-eksekverende værktøjer (`ToolActionType::Other` skal fortsat blokeres).
- **Må IKKE** ændre adapter-kontrakter for øvrige harnesses (Claude Code, Codex, Mistral Vibe).
- **Må IKKE** udføre remote publication (`git push`) jf. ADR 0003.

---

## 📝 Revisions

- 2026-11-14: Oprettet som selvstændig bugfix-task udskilt fra udkastet til Task 080 (`exit_plan_mode`).

---

## 🧪 Verifikation

- `cargo test -p xgauntlet-core --test antigravity_adapter_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

