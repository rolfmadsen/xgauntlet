---
type: Task Package
title: "Task 102: Refresh verification-report.json digests after mark_task_done in DONE checkpoint"
description: "Opdater verification-report.json workspace_state og task_digest efter mark_task_done og commit i checkpoint --phase done, så xgauntlet check-release --strict passerer uden drift i CI."
status: active
intent: bug
generated: { by: process:antigravity-task-init, at: "2026-11-16T08:10:00Z" }
tags: [checkpoint, evidence, release, drift, bugfix]
scope:
  - "crates/xgauntlet-core/src/features/checkpoint/engine.rs"
test_refs:
  - "crates/xgauntlet-core/tests/checkpoint_done_drift_test.rs"
---

# Task 102: Refresh verification-report.json digests after mark_task_done in DONE checkpoint

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-16`

## 🎯 Formål
Eliminere falsk `Workspace state drift detected` (`source_manifest_digest_post` og `task_digest`) i `xgauntlet check-release --strict` efter `xgauntlet checkpoint --phase done`, hvor `mark_task_done` ændrer `tasks/<id>.md` efter preflight-verifikationen blev kørt.

## 📋 Acceptance Criteria
- [ ] **AC1 — Zero-drift post-DONE `verification-report.json`**: Når `run_checkpoint` gennemfører `CheckpointPhase::Done` med preflight (`!skip_verify`), opdateres `verification-report.json` (og `evidence.md`) før `seal_tracked_evidence_files`, så `verify_workspace_state_match(&report, &compute_workspace_manifest(...))` passerer uden `source_drift` eller `task_drift`.
- [ ] **AC2 — Strict Release Gatekeeper Grøn**: `cargo run -p xgauntlet-cli -- check-release --strict` og `cargo test --workspace` passerer med exit code `0`.

## 🚫 Must NOT
- Må IKKE svække `verify_self_mutation` under `execute_gauntlet_pipeline` eller omgå `verify_workspace_state_match` i `check-release --strict`.
- Må IKKE ændre versionsnumre væk fra `0.16.2`.

## 📝 Revisions
- 2026-11-16: Oprettet fast-track bugfix efter CI `gate-check-release` fejlede på post-`DONE` task/manifest drift.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test checkpoint_done_drift_test`
- `cargo run -p xgauntlet-cli -- check-release --strict`
