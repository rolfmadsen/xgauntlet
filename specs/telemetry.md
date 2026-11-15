# Living Feature Specification: `telemetry`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Feature Path**: `crates/xgauntlet-core/src/features/telemetry/`  
**Type**: `Stateless JIT Governance & Session Health Telemetry Feature`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0007](../docs/adr/0007-local-transparent-supervisor-and-wasm-verifier.md), [ADR 0008](../docs/adr/0008-sovereign-agentic-sdlc-task-scoping-and-design-tokens.md), [ADR 0013](../docs/adr/0013-feature-architecture-boundaries-and-living-specs.md)

---

## 🎯 Purpose & Capability Boundary
The `telemetry` feature delivers zero-daemon, deterministic Just-In-Time (JIT) phase directives and advisory session health tracking across isolated AI harness invocations:
- **JIT Governance Directives (`jit.rs`)**: Maps TDD pipeline phases (`PipelinePhase`) to compact role, target, gate, and skill pointer blocks (`JitPhaseDirective`) injected via `PreInvocation` telemetry without permanent system-prompt bloat.
- **Session Health & Token Telemetry (`session.rs`)**: Tracks interaction turn counts, payload bytes, and deterministic BPE character-heuristic token estimates (`~1 token ≈ 3.8 chars`), categorizing session health (`Optimal`, `Aging`, `Saturated`) from both turns and estimated tokens (`from_usage`) per `session_id`/`conversation_id` inside `.xgauntlet/sessions/<hash>.json` (with `.xgauntlet/session_state.json` legacy fallback), isolating Antigravity `PreInvocation` dedup caches by `(canonical_workspace, session_id)` hash, and resetting cleanly on `handoff`.
- **Snapshot-Safe Payload & Transcript Enrichment**: Optionally derives monotonic snapshots (`turns_count`, `estimated_tokens`) from explicit hook token fields or a local JSONL `transcript_path` without network calls, daemons, or quadratic double-counting across repeated `PreInvocation` hooks.
- **Empirical Cockpit Truthfulness (`tasks/telemetry.rs`)**: Enriches `TaskTelemetry` (including `session_id`-aware lookup) and `IdleWorkspaceTelemetry` (`Task: NONE`) with fast (`<5ms`) real signals: `VerificationFreshness` (`NONE`, `PASS`, `FAIL`, `STALE` vs. task ID, dirty tree, and `HEAD`), task `scope` + *Scope Hygiene* (out-of-scope dirty file counts), worktree/task binding, and `commits_ahead`, without fabricated defaults (`14/14 PASS`, `Mutation: 100%`, `Drift: 0%`, `crates/*`).

## 🚫 Invariants
- Never spawns background daemons, sockets, or network calls for token estimation or telemetry persistence.
- Never runs heavy builds or full-repo digest hashing inside `PreInvocation` / `telemetry` inspection.
- Never fabricates `PASS`, invariant counts, or mutation percentages when `verification-report.json` is missing or stale.
- Never blocks modifying tool calls (`PreToolUse`) based on `SessionHealth` (strictly advisory).
- Never attaches turn-counting I/O to `PreToolUse` (`Commands::Hook`), preserving `<5ms` cold-start policy evaluation.
- Keeps `.xgauntlet/session_state.json` and `.xgauntlet/sessions/` ignored by Git and exempt from `checkpoint` scope staging so runtime telemetry never causes false `dirty` status or `ScopeViolation`.

## 📋 Relaterede Tasks
- [Task 023: Pipeline JIT Governance & Global Plugin Distribution](../tasks/archive/023-pipeline-jit-governance-and-global-plugin-distribution.md)
- [Task 044: Session Health, Turn & Token Telemetry](../tasks/archive/044-session-health-turn-and-token-telemetry.md)
- [Task 085: HUD Start/End Protocol & Active Session Telemetry Wiring](../tasks/archive/085-hud-start-end-protocol-and-session-telemetry-wiring.md)
- [Task 088: Worktree-Isolated Multi-Agent & Subagent Supervision](../tasks/archive/088-worktree-isolated-multi-agent-supervision.md)
- [Task 093: Multi-Agent Worktree Workflow, Coexistence Policy & Runtime/Evidence Hygiene](../tasks/archive/093-multi-agent-worktree-and-runtime-hygiene.md)
- [Task 094: Cockpit HUD Truthfulness, Session Resolution & Antigravity Start-HUD Cadence](../tasks/archive/094-cockpit-hud-truthfulness-and-cadence.md)

