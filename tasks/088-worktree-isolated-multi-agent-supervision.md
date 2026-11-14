---
type: Task Package
title: "Task 088: Worktree-Isolated Multi-Agent & Subagent Supervision"
description: "Muliggør xGauntlet-supervision af parallelle sessioner, subagenter og swarms på tværs af Antigravity, Claude Code, Codex og Mistral Vibe via git worktree-isolation, lokal task-binding, delt worktree-CWD-opløsning og sessions-scopet telemetri."
status: active
intent: feature
generated: { by: process:antigravity-task-init, at: "2026-11-14T22:05:00Z" }
tags: [multi-agent, worktree, adapters, policy, telemetry, antigravity, claude_code, codex, mistral, adr-0014]
feature: adapters
spec_ref: "specs/adapters.md"
scope:
  - "wit/gauntlet_policy.wit"
  - "crates/gauntlet-policy-engine/src/**"
  - "crates/xgauntlet-core/src/features/adapters/**"
  - "crates/xgauntlet-core/src/features/policy/**"
  - "crates/xgauntlet-core/src/features/tasks/**"
  - "crates/xgauntlet-core/src/features/telemetry/**"
  - "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
  - "crates/xgauntlet-cli/src/**"
  - "specs/adapters.md"
  - "specs/tasks.md"
  - "specs/telemetry.md"
  - "docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md"
  - "docs/adr/README.md"
  - "CONTEXT.md"
test_refs:
  - "crates/xgauntlet-core/tests/multi_agent_worktree_test.rs"
  - "crates/gauntlet-policy-engine/tests/delegation_policy_test.rs"
---

# Task 088: Worktree-Isolated Multi-Agent & Subagent Supervision

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-11-14`

---

## 🎯 Formål

Implementere [ADR 0014](../docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md) så `xGauntlet` deterministisk og fail-closed kan supervisere **både**:
1. **Parallelle uafhængige opgaver/sessioner** i hver sin `git worktree` (f.eks. Session A på `task 088` og Session B på `task 089`), og
2. **Koordinator + delegerede subagenter / swarms** på tværs af **Google Antigravity** (`invoke_subagent`, `swarm`, `send_message`, `define_subagent`, `manage_subagents`, `run_workflow`), **Claude Code** (`Agent`/`Task` med `isolation: "worktree"`), **OpenAI Codex** og **Mistral Vibe**, uden at gå på kompromis med Zero-Daemon ([ADR 0007](../docs/adr/0007-local-transparent-supervisor-and-wasm-verifier.md)), Self-Mutation Invariant eller atomiske TDD-checkpoints ([ADR 0003](../docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md)).

*Note:* Opgaven afventer menneskelig godkendelse af `ADR 0014` og `SPEC`-gate (ADR 0009 Gate 1) før aktivering (`status: active`) og oprettelse af den levende specifikation `specs/adapters.md`.

---

## 📋 Acceptance Criteria

- [ ] **AC1 — Worktree-Lokal Task-Binding (`.xgauntlet/active_task`):**
  - `resolve_active_task_path` / `resolve_active_task_id` i `crates/xgauntlet-core/src/features/tasks/parser.rs` tjekker først en gitignoreret worktree-lokal binding (`.xgauntlet/active_task`), før den falder tilbage til den højeste nummererede `ACTIVE` task i `tasks/**`.
  - Hvis `.xgauntlet/active_task` peger på en ikke-eksisterende eller ikke-aktiv (`DONE`/`DRAFT`) task, fejler opløsningen sikkert (ingen stille forkert opgavebinding) eller falder deterministisk tilbage jf. specifikation.
  - CLI (`xgauntlet task bind <id>` / `xgauntlet task unbind`) tilføjes i `crates/xgauntlet-cli`, så bruger eller koordinator kan binde hver worktree eksplicit til dens respektive task.
  - To samtidige `git worktree`s på samme repository kan dermed aktivere, redigere, verificere (`xgauntlet verify`) og checkpointe to forskellige aktive tasks uafhængigt uden `4037`-kollisioner.

- [ ] **AC2 — Fælles Payload-Bevidst Worktree-Opløsning for Alle 4 Harnesses:**
  - `resolve_effective_workspace(workspace, payload)` løftes til fælles adapter-funktionalitet i `crates/xgauntlet-core/src/features/adapters/` og anvendes i `handle_hook` for **alle fire** adapters (`AntigravityAdapter`, `ClaudeCodeAdapter`, `CodexAdapter`, `MistralAdapter`).
  - Når en harness eksekverer et hook fra parent-repoets CWD med et gyldigt worktree i payloadens `cwd` eller `workspacePaths`, opløses `EnforcementContext::from_workspace` mod det pågældende worktree — stadig strengt afgrænset af `is_valid_workspace` og sti-sanitering ([ADR 0006](../docs/adr/0006-multi-harness-policy-adapter-contract.md)).

- [ ] **AC3 — Kanonisk Klassifikation & WASM-Policy for Agent Delegation:**
  - `wit/gauntlet_policy.wit` og `crates/gauntlet-policy-engine` opdateres (og `crates/xgauntlet-core/wasm/gauntlet_policy.wasm` rekompileres) til at håndhæve sikker agent-delegation (via udvidet `ToolActionType` eller kanonisk delegation-evaluering med fuld reference/WASM-paritet):
    - **Read-only delegation** (f.eks. Antigravity `invoke_subagent` med `TypeName: "research"` eller `Workspace: "inherit"` for ikke-muterende subagenter, `manage_subagents` `list`, Claude Code read-only `Agent`/`Task`) tillades — også uden aktiv task (`SPEC`/`QUERY`).
    - **Muterende / skrivende worker-delegation** (`swarm` med `workspace: "share"` / `"branch"`, skrivende subagenter i isolerede worktrees, `run_workflow`) kræver aktiv task og **afvises fail-closed**, hvis en muterende worker forsøges startet med `workspace: "inherit"` (deler parent working tree med anden aktiv skriver).
    - Ukendte værktøjer forbliver blokeret fail-closed med `reason_code: 4038`.
  - `AntigravityAdapter::normalize_tool_call` (og tilsvarende normalisering i `ClaudeCodeAdapter`, `CodexAdapter`, `MistralAdapter` samt valgfrie hook-matchers) dækker subagent-/orkestreringsværktøjer ensartet.

- [ ] **AC4 — Workspace- & Sessions-Scopet Telemetri (Zero-Daemon):**
  - Antigravitys `PreInvocation`-dedup-cache i `crates/xgauntlet-core/src/features/adapters/antigravity/telemetry.rs` nøgles pr. `(canonical_workspace, session_id)`-hash i stedet for `<USER>`, så parallelle sessioner eller worktrees aldrig undertrykker hinandens telemetri.
  - `.xgauntlet/session_state.json` i `crates/xgauntlet-core/src/features/telemetry/session.rs` understøtter sessions-keyed state (når `session_id`/`conversation_id` indgår i payloaden), uden at bryde eksisterende enkelt-sessions-adfærd eller `SessionHealth`.

- [ ] **AC5 — Levende Specifikationer & Multi-Harness Kontrakt-Tests:**
  - `specs/adapters.md` oprettes, og `specs/tasks.md` samt `specs/telemetry.md` opdateres med to-vejs sporbarhed til Task 088.
  - Dedikerede, fokuserede tests i `crates/xgauntlet-core/tests/multi_agent_worktree_test.rs` og `crates/gauntlet-policy-engine/tests/delegation_policy_test.rs` (begge under 1.000 linjer) verificerer worktree-task-binding, payload-worktree-opløsning på tværs af alle 4 harnesses, delegation-policy (read-only vs. `share`/`branch` vs. afvist `inherit`), og isoleret telemetri-dedup.

---

## 🚫 Must NOT

- **Må IKKE** tillade flere agenter eller sessioner at skrive samtidigt i *samme* working tree (`workspace: "inherit"` for parallelle skrivende workers skal afvises fail-closed).
- **Må IKKE** introducere baggrundsdæmoner, sockets, PID-lock-filer eller eksterne runtime-afhængigheder ([ADR 0007](../docs/adr/0007-local-transparent-supervisor-and-wasm-verifier.md)).
- **Må IKKE** lade hook-payloads overskrive `has_active_task`, `read_only` eller `allowed_scopes` i `EnforcementContext` ([ADR 0006](../docs/adr/0006-multi-harness-policy-adapter-contract.md)).
- **Må IKKE** svække fail-closed `ToolActionType::Other` (`4038`) blokeringen for ukendte eller uautoriserede værktøjer.
- **Må IKKE** ændre `.xgauntlet/active_task` eller `.xgauntlet/session_state.json` fra at være gitignorerede (runtime-state må aldrig skabe falsk `dirty` status eller `ScopeViolation`).
- **Må IKKE** skabe monolitiske testfiler (>1.000 linjer) eller udføre `git push` / `git reset --hard` ([ADR 0003](../docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md)).

---

## 📝 Revisions

- 2026-11-14: Oprettet udkast (`DRAFT`) sammen med [ADR 0014](../docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md) og begrebsafklaring i `CONTEXT.md`.
- 2026-11-14: Bruger godkendte Gate 1 (`SPEC`): [ADR 0014](../docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md) sat til `accepted`, Task 088 aktiveret (`ACTIVE`) og `specs/adapters.md` oprettet. Afventer menneskelig `SPEC` & `ADR 0014` godkendelse før aktivering (`ACTIVE`) og oprettelse af `specs/adapters.md`.

---

## 🧪 Verifikation

- `cargo run -p xgauntlet-cli -- check-adr --all`
- `cargo run -p xgauntlet-cli -- check-spec -t 088`
- `cargo run -p xgauntlet-cli -- check-spec --context`
- `cargo test -p gauntlet-policy-engine`
- `cargo test -p xgauntlet-core --test multi_agent_worktree_test`
- `cargo run -p xgauntlet-cli -- verify --task 088`

