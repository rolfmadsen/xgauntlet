---
type: Task Package
title: "Task 098: Fix Windows Verbatim Path (\\\\?\\) in git worktree add"
description: "Stripper Windows extended-length verbatim path-præfikser (\\\\?\\ og \\\\?\\UNC\\) fra canonicalize() i worktree.rs, så xgauntlet worktree create og git worktree add/remove virker fejlfrit på Windows CI."
status: done
intent: bug
feature: tasks
spec_ref: "specs/tasks.md"
generated: { by: process:antigravity-task-init, at: "2026-11-15T16:04:00Z" }
tags: [bugfix, windows, worktree, git, ci]
scope:
  - "tasks/098-fix-windows-worktree-verbatim-path.md"
  - "crates/xgauntlet-core/src/features/tasks/worktree.rs"
  - "crates/xgauntlet-core/src/features/tasks/mod.rs"
test_refs:
  - "crates/xgauntlet-cli/tests/worktree_cli_test.rs"
---

# Task 098: Fix Windows Verbatim Path (`\\?\`) in `git worktree add`

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-15`

## 🎯 Formål
Rette Windows CI-fejlen i `test_ac5_cli_worktree_create_creates_in_repo_worktree_and_binds_task`:
- På Windows returnerer `std::fs::canonicalize()` (via `resolve_primary_repo_root`) et extended-length verbatim path med præfikset `\\?\C:\...`.
- Når `create_task_worktree` og `cleanup_failed_worktree` i [worktree.rs](../crates/xgauntlet-core/src/features/tasks/worktree.rs) sender den sti til `git.exe worktree add` / `remove`, oversætter Git for Windows den til `//?/C:/.../.git` og fejler med `Invalid argument`.

## 📋 Acceptance Criteria
- [x] **AC1 (RED-test for Windows Verbatim Path Normalization)**: `crates/xgauntlet-cli/tests/worktree_cli_test.rs` verificerer via `strip_windows_verbatim_prefix`, at både drive-paths (`\\?\C:\...` → `C:\...`), UNC-paths (`\\?\UNC\server\share\...` → `\\server\share\...`) og POSIX-stier (`/tmp/...` → `/tmp/...`) normaliseres uden at `git worktree`-argumenter indeholder `\\?\`.
- [x] **AC2 (Fix i `worktree.rs`)**: `create_task_worktree` og `cleanup_failed_worktree` normaliserer `primary_root` og `worktree_path` via `strip_windows_verbatim_prefix` før `git worktree add` / `remove` og i returværdien `WorktreeCreateResult`.
- [x] **AC3 (Verification)**: `cargo test -p xgauntlet-cli --test worktree_cli_test` og `cargo run -p xgauntlet-cli -- verify --task 098 --save` passerer alle obligatoriske lag.

## 🚫 Must NOT
- Ændr IKKE `resolve_git_common_dir`s interne canonicalize-sammenligninger i `adapters/workspace.rs` (bevarer eksisterende worktree/symlink-lighed).
- Foretag IKKE `git push`, `git reset --hard` eller `git clean -f` (ADR 0003).

## 📝 Revisions
- 2026-11-15: Oprettet bugfix-opgavepakke efter Windows Server 2025 CI-fejl i `v0.16.0` (`worktree_cli_test.rs:76`).
- 2026-11-15: Implementeret `strip_windows_verbatim_prefix` og `normalize_git_path` i `worktree.rs` + cross-platform test i `worktree_cli_test.rs` (3/3 bestået).

## 🧪 Verifikation
- `cargo test -p xgauntlet-cli --test worktree_cli_test`
- `cargo run -p xgauntlet-cli -- verify --task 098 --save`

