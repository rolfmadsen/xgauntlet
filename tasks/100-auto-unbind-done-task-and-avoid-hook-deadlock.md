---
type: Task Package
title: "Task 100: Auto-unbind done active_task and prevent hook deadlock on Task NONE"
description: "Ryd .xgauntlet/active_task automatisk ved checkpoint --phase done, eksponer tydelig recovery-vejledning ved terminal Worktree Task Binding, og undgå at PreToolUse-hooken blokerer task bind/unbind/status/list og telemetry ved Task: NONE."
status: active
intent: bug
feature: tasks
spec_ref: "specs/tasks.md"
generated: { by: process:antigravity-task-init, at: "2026-11-15T20:42:00Z" }
tags: [tasks, checkpoint, policy, wasm, hooks, active-task, deadlock]
scope:
  - "tasks/100-auto-unbind-done-task-and-avoid-hook-deadlock.md"
  - "crates/xgauntlet-core/src/features/checkpoint/engine.rs"
  - "crates/xgauntlet-core/src/features/tasks/parser.rs"
  - "crates/xgauntlet-core/src/features/tasks/cockpit.rs"
  - "crates/xgauntlet-core/src/features/adapters/**"
  - "crates/gauntlet-policy-engine/src/command.rs"
  - "crates/gauntlet-policy-engine/src/lib.rs"
  - "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
  - "specs/tasks.md"
  - "specs/checkpoint.md"
  - "specs/adapters.md"
test_refs:
  - "crates/xgauntlet-core/tests/checkpoint_engine_test.rs"
  - "crates/xgauntlet-core/tests/multi_agent_worktree_test.rs"
  - "crates/xgauntlet-core/tests/policy_engine_test.rs"
  - "crates/xgauntlet-core/tests/hook_policy_and_telemetry_test.rs"
---

# Task 100: Auto-unbind done active_task and prevent hook deadlock on Task NONE

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-15`

## 🎯 Formål
Løse deadlock-kæden omkring Worktree Task Binding (`.xgauntlet/active_task`) og `PreToolUse`-hooken, når en bundet Task afsluttes eller efterlades med terminal status:

1. **Årsagskæde observeret i praksis**:
   - Når flere opgaver er `ACTIVE` samtidig, bindes workspacet via `xgauntlet task bind --task <id>`, som skriver opgavens ID til `.xgauntlet/active_task`.
   - `run_checkpoint` (`xgauntlet checkpoint --phase done`) markerer opgaven som `DONE`, men fjerner ikke `.xgauntlet/active_task`.
   - `resolve_active_task_path` returnerer `None` (`Task: NONE`), når `.xgauntlet/active_task` peger på en ikke-aktiv/terminal opgave (fail-closed jf. ADR 0014).
   - `gauntlet-policy-engine` (`is_safe_xgauntlet_subcommand`) tillader ikke `xgauntlet status`, `xgauntlet telemetry` eller `xgauntlet task list|status|unbind|bind` uden aktiv Task (`has_active_task: false`), så `PreToolUse` (`xgauntlet hook antigravity`) låser agenten ude fra både inspektion og recovery.

2. **Arkitektonisk afgrænsning (ADR 0014)**:
   - `multi_agent_worktree_test.rs` verificerer eksplicit, at en ugyldig/terminal `.xgauntlet/active_task` fortsat skal fejle fail-closed i `resolve_active_task_path` for ikke at kapre en søster-worktrees aktive Task.
   - Derfor fokuseres løsningen på: (a) automatisk unbind ved `checkpoint --phase done`, (b) eksplicit diagnostik/recovery-vejledning (`xgauntlet task unbind` / `xgauntlet task bind --task <id>`) ved forældet binding, og (c) sikker tilladelse i WASM Policy Component (`gauntlet-policy-engine`) for read-only inspektion og recovery-kommandoer under `Task: NONE`.

## 📋 Acceptance Criteria
- [ ] **AC1 (Auto-unbind ved `DONE`)**: Når `xgauntlet checkpoint --phase done` gennemføres med succes (`run_checkpoint` i `crates/xgauntlet-core/src/features/checkpoint/engine.rs`), fjernes `.xgauntlet/active_task` automatisk, hvis den bundne opgave i `.xgauntlet/active_task` matcher den afsluttede `task_id` (eller kort-ID). Hvis commit/checkpoint fejler og rulles tilbage, bevares bindingen uændret.
- [ ] **AC2 (Robust diagnosticering & recovery ved terminal binding)**: Når `.xgauntlet/active_task` peger på en opgave med terminal status (`DONE`, `COMPLETED`, `DEPRECATED`, `SUPERSEDED`, `REJECTED`) eller en manglende opgave, bevares fail-closed task-resolution i `resolve_active_task_path` (ADR 0014), mens `hook` (`stderr`), `cockpit` / `PreInvocation` HUD og task-resolution eksponerer en tydelig advarsel og den præcise recovery-kommando (`xgauntlet task unbind` eller `xgauntlet task bind --task <aktiv-id>`). *(Alternativt, hvis automatisk fallback vælges, skal ADR 0014 og `test_worktree_local_active_task_binding_overrides_highest_number` eksplicit revideres først.)*
- [ ] **AC3 (Deadlock-fri `PreToolUse` & WASM Policy Component)**: I `crates/gauntlet-policy-engine/src/command.rs` (og den rekompilerede `crates/xgauntlet-core/wasm/gauntlet_policy.wasm`) tillades følgende ufarlige kommandoer via Structured Command Tokenization uden aktiv Task (`has_active_task: false`), både som `xgauntlet ...`, `npx xgauntlet ...` og `cargo run -p xgauntlet-cli -- ...` uden shell-metategn:
  - `status` og `telemetry`
  - `task list` og `task status`
  - `task unbind`
  - `task bind` (selve CLI-kommandoen `bind_active_task` afviser fortsat fail-closed mod ikke-eksisterende eller ikke-`ACTIVE` opgaver)
- [ ] **AC4 (Living Feature Specs & Verification Gauntlet)**: `specs/tasks.md`, `specs/checkpoint.md` og `specs/adapters.md` opdateres ved behov med link til Task 100, `gauntlet_policy.wasm` er rekompileret og synkroniseret, og `cargo test --workspace`, `cargo clippy --all-targets -- -D warnings` samt `xgauntlet verify` passerer.

## 🚫 Must NOT
- Må IKKE tillade ukontrolleret kodeændring (`WriteFile` til kildekode eller vilkårlige `ExecuteCommand`-kald) uden en gyldig `ACTIVE` Task (`has_active_task: false`).
- Må IKKE tillade `xgauntlet task bind` mod ikke-eksisterende eller terminale (`DONE`, `COMPLETED`, `DEPRECATED`, `SUPERSEDED`, `REJECTED`) opgaver.
- Må IKKE lade `gauntlet-policy-engine` omgå shell-metategn- eller execution-wrapper-spærrer (`&&`, `;`, `|`, `$(...)`, `env`, `bash -c` m.fl. evalueres fortsat strict per segment).
- Må IKKE bryde Worktree Task Binding-isolationen i ADR 0014 (`multi_agent_worktree_test.rs`) ved tavst at falde tilbage til en anden worktrees opgave uden eksplicit beslutning.
- Må IKKE foretage `git push` eller destruktive git resets (`git reset --hard`, `git clean -f`).

## 📝 Revisions
- 2026-11-15: Oprettet opgavepakke for auto-unbind af afsluttet `.xgauntlet/active_task` og deadlock-fri `PreToolUse`-policy ved `Task: NONE`.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 100`
- `cargo test -p gauntlet-policy-engine`
- `cargo test -p xgauntlet-core --test checkpoint_engine_test`
- `cargo test -p xgauntlet-core --test multi_agent_worktree_test`
- `cargo test -p xgauntlet-core --test policy_engine_test`
- `cargo test -p xgauntlet-core --test hook_policy_and_telemetry_test`
- `cargo run -p xgauntlet-cli -- verify`

