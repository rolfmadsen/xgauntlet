# Living Feature Specification: `checkpoint`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Feature Path**: `crates/xgauntlet-core/src/features/checkpoint/`  
**Type**: `Stateless Phase-Bound TDD Checkpoint & Provenance Feature`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0003](../docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md), [ADR 0009](../docs/adr/0009-autonomous-tdd-inner-loop-and-boundary-human-gates.md), [ADR 0011](../docs/adr/0011-fast-track-worktree-inverse-proof-and-scoped-preflight.md), [ADR 0013](../docs/adr/0013-feature-architecture-boundaries-and-living-specs.md)

---

## 🎯 Purpose & Capability Boundary
The `checkpoint` feature enforces phase-bound TDD invariant verification (`SPEC`, `RED`, `GREEN`, `REFACTOR`, `DONE`) and creates atomic, strictly local conventional Git commits:
- **Orchestrator (`engine.rs`)**: `run_checkpoint(options)` resolves the target task, enforces `scope` / `test_refs` commit boundaries via `validate_staged_files_scope`, executes local commits, generates `SessionHandoff` cards on `DONE`, and appends `SessionLineageEntry` and test provenance into `verification-report.json`.
- **Pre-Flight Engine (`preflight.rs`)**: `run_preflight_check(options)` validates phase-specific invariants before commit creation:
  - `SPEC`: Validates OKF v0.2 task specification readiness (`check_task_specification`).
  - `RED`: Runs scoped `test_refs` targets (`resolve_preflight_test_layers`) and verifies that expected test assertions fail rather than production syntax/manifest errors.
  - `GREEN` / `REFACTOR`: Verifies born-green test provenance (`detect_born_green_tests` + `verify_inverse_test_failure`), confirms all scoped `test_refs` pass, and enforces the Self-Mutation Invariant in `REFACTOR`.
  - `DONE`: Confirms preceding `RED` commit (for both standard TDD and 2-step Fast-Track `RED -> DONE` bugfixes with `intent: bug`, `scope.len() <= 3`, no wildcards) and runs the full `execute_gauntlet_pipeline` verification.
- **Git & Provenance (`git.rs` & `provenance.rs`)**: Formats conventional commit messages (`compose_commit_message`), stages and commits locally without remote publication, and extracts/verifies test provenance.

## 🚫 Invariants
- Never executes `git push`, `git reset --hard`, `git clean -f`, or any remote publication action (ADR 0003).
- Never bypasses full Gauntlet pipeline verification or evidence sealing in `DONE` unless `--skip-verify` is explicitly requested.
- Never commits files outside the active task's declared `scope` / `test_refs` (apart from canonical governance/evidence files).

## 📋 Relaterede Tasks
- [Task 014: Phase Checkpoint Engine](../tasks/archive/014-phase-checkpoint-engine.md)
- [Task 032: Enforce RED-First Test Provenance and Born-Green Detection](../tasks/archive/032-enforce-red-first-test-provenance-and-born-green-detection.md)
- [Task 052: DX Fast-Track and Hook Hardening](../tasks/archive/052-dx-fast-track-and-hook-hardening.md)
- [Task 056: Simplify Fast-Track and Retire Worktree Inverse Proof](../tasks/archive/056-simplify-fast-track-and-retire-worktree-inverse-proof.md)
- [Task 064: Static Checkpoint Commands and Harness Auto-Allow](../tasks/archive/064-static-checkpoint-commands-and-harness-auto-allow.md)
- [Task 084: Reduce Policy Hook, Done-Transition & Sandbox Checkpoint Friction](../tasks/archive/084-reduce-policy-and-checkpoint-sandbox-friction.md)

