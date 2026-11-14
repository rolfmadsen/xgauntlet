---
type: Task Package
title: "Task 084: Reduce Policy Hook, Done-Transition & Sandbox Checkpoint Friction"
description: "Eliminer gentagne PreToolUse-blokeringer på read-only søgekommandoer og DONE-overgangen samt dobbelte verifikationskørsler ved read-only .git/index.lock i sandbox."
status: active
intent: enhancement
feature: checkpoint
spec_ref: "specs/checkpoint.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T17:58:30Z" }
tags: [dx, policy-engine, checkpoint, sandbox, pretooluse, red-phase]
scope:
  - "specs/checkpoint.md"
  - "tasks/084-reduce-policy-and-checkpoint-sandbox-friction.md"
  - "crates/gauntlet-policy-engine/src/lib.rs"
  - "crates/gauntlet-policy-engine/src/command.rs"
  - "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
  - "crates/xgauntlet-core/src/features/policy/**"
  - "crates/xgauntlet-core/src/features/checkpoint/**"
  - "crates/xgauntlet-core/src/features/tasks/lifecycle.rs"
  - "crates/xgauntlet-core/tests/policy_engine_test.rs"
  - "crates/xgauntlet-core/tests/checkpoint_engine_test.rs"
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
test_refs:
  - "crates/xgauntlet-core/tests/policy_engine_test.rs"
  - "crates/xgauntlet-core/tests/checkpoint_engine_test.rs"
---

# Task 084: Reduce Policy Hook, Done-Transition & Sandbox Checkpoint Friction

**Status**: `ACTIVE`  
**Intent**: `🔄 ENHANCEMENT`  
**Oprettet**: `2026-11-14`

## 🎯 Formål
Fjerne de konkrete friktionspunkter og unødige gentagelser mellem `xgauntlet`-regler (`PreToolUse`, `checkpoint`, `check-spec`) og IDE-sandboxen, som blev observeret under Task 081:

1. **Read-only inspektion uden aktiv task:** `grep`, `rg`, `git show`, snævre `cargo run -p xgauntlet-cli -- <safe-subcmd>` og sikre read-only `&&`-kæder (f.eks. `git status -sb && git log -n 5 --oneline`) blokeres i dag af `4033` (`Executing modifying commands without an active task is prohibited`), bl.a. fordi rå metachar-tjek rammer `&&` og quotede mønstre.
2. **`status: done` Catch-22:** Når en opgave markeres `status: done`, falder `has_active_task` til `false`, hvorefter `PreToolUse` blokerer opfølgende `cargo run -p xgauntlet-cli -- verify`, `&&`-kæder med `xgauntlet checkpoint --phase done` og `git show`.
3. **Dyr dobbeltkørsel på sandbox `.git/index.lock`:** `xgauntlet checkpoint` kører hele verifikationspipelinen (~35s ved `--phase done`), før `git commit` opdager at `.git/` / `.git/index.lock` er read-only i standard-sandboxen.
4. **`RED` preflight vs. flere `test_refs`:** `checkpoint --phase red` afviser i dag hele `RED`-fasen, hvis bare én fil i `test_refs` allerede består — også når opgaven har flere testfiler i `test_refs` og den nye røde test ligger i én af dem.

## 📋 Acceptance Criteria
- [ ] **AC1 (Tokeniseret Read-Only Whitelist, `&&`-kæder & `cargo run -p xgauntlet-cli`):**
  - `crates/gauntlet-policy-engine/src/command.rs` og `lib.rs` (med genopbygget `crates/xgauntlet-core/wasm/gauntlet_policy.wasm`) tillader uden aktiv task (`!has_active_task`):
    - Read-only `grep`, `rg` (ekskl. `--pre` / `--pre-glob`) og `git show`.
    - Snævert `cargo run -p xgauntlet-cli -- <safe-subcmd>` (og `--package xgauntlet-cli`), hvor `<safe-subcmd>` matcher samme sikre `xgauntlet`-whitelist (`verify`, `doctor`, `check-evidence`, `check-spec`, `check-release`, `check-adr`, `check-architecture`, `handoff`, `lineage`, `checkpoint`).
    - `&&`-kæder hvor **samtlige** tokeniserede segmenter består `is_safe_command_segment`, uden at quotede tegn (`|`, `>`, `<`, `&`) i f.eks. `rg`/`grep`-mønstre falsk-positivt udløser metachar-blokering.
- [ ] **AC2 (Friktionsfri `DONE`-Overgang i `checkpoint --phase done`):**
  - `xgauntlet checkpoint --phase done` opdaterer automatisk den aktive task-fils frontmatter (`status: done`) og body (`**Status**: \`DONE\``) **efter** bestået `run_preflight_check` og **før** `stage_workspace_changes` + `execute_git_commit` (med rollback af task-filen, hvis commit fejler), så commit `chore(<id>): seal evidence and mark task DONE` efterlader worktreet rent og uden `4033`-stranding.
- [ ] **AC3 (Fail-Fast `.git/` Skrivecheck før Verifikation):**
  - `xgauntlet checkpoint` resolver den reelle git-mappe via `git rev-parse --git-dir` (worktree-kompatibel) og tester skriveadgang med en unik `create_new(true)`-probe-fil (uden at røre eksisterende `.git/index.lock`), **før** `run_preflight_check` / `verify` startes, og fejler øjeblikkeligt med en præcis fejlbesked ved read-only `.git/`.
- [ ] **AC4 (`RED` Preflight for Flere `test_refs` & Skarp Diagnostik):**
  - `checkpoint --phase red` godkender `RED`-fasen, når **mindst ét** scoped `test_refs`-target fejler med gyldig testfejl (og uden produktions-syntaks/manifestfejl), mens `GREEN` og `REFACTOR` fortsat kræver at **samtlige** `test_refs` består; ved `RedPhaseUnmet` rapporteres præcist hvilke targets der bestod/fejlede.

## 🚫 Must NOT
- Må IKKE tillade muterende shell-konstruktioner (`>`, `>>` uden for anførselstegn, `;`, `||`, `|`, `$(...)`, backticks, `rm`, `git push`, `git reset --hard`), `rg --pre` / `--pre-glob` eller vilkårlig `cargo run` (andre packages end `xgauntlet-cli` eller usikre subcommands) uden aktiv task (fail-closed policy bevares).
- Må IKKE omgå fuld verifikation eller evidensforsegling i `checkpoint --phase done`.
- Må IKKE bryde lagarkitekturen (`app -> features -> core -> shared`) eller 800-linjers loftet i `gauntlet.toml` (`crates/gauntlet-policy-engine/src/lib.rs` er allerede 797 linjer; ny logik lægges i `command.rs` eller tests udtrækkes).

## 📝 Revisions
- 2026-11-14: Oprettet draft task baseret på session-analyse af Task 081-forløbet (`4033`-blokeringer, `.git/index.lock` sandbox-retries og `RED phase unmet`).
- 2026-11-14: Skærpet scope (`command.rs`, `gauntlet_policy.wasm`, `tasks/lifecycle.rs`), præciseret AC1–AC4 (tokeniserede `&&`-kæder, `cargo run -p xgauntlet-cli`, `rg --pre`-blokering, worktree-sikker `.git/`-probe, mindst-ét-rødt `RED`-kriterium og `lib.rs` 800-linjers grænse) samt oprettet `specs/checkpoint.md`.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 084`
- `cargo test -p xgauntlet-core --test policy_engine_test`
- `cargo test -p xgauntlet-core --test checkpoint_engine_test`
- `cargo test --workspace`

