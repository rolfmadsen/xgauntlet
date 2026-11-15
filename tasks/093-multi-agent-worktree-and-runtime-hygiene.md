---
type: Task Package
title: "Task 093: Multi-Agent Worktree Workflow, Coexistence Policy & Runtime/Evidence Hygiene"
description: "Løser observationerne fra Kant-repoet efter Task 088 (v0.15.1): ekskluderer .xgauntlet/ og .worktrees/ fra verify-manifest og checkpoint-staging, sikrer clean working tree efter checkpoint --phase done, gør per-session telemetri race-fri (.xgauntlet/sessions/), udvider read-only/branch-klon-opløsning og tilføjer opt-in coordinated inherit + shared CARGO_TARGET_DIR samt xgauntlet worktree create."
status: active
intent: feature
generated: { by: process:antigravity-task-init, at: "2026-11-15T00:26:00Z" }
tags: [multi-agent, worktree, adapters, evidence, checkpoint, telemetry, cargo-target-dir, adr-0014]
feature: adapters
spec_ref: "specs/adapters.md"
scope:
  - ".gitignore"
  - "tasks/093-multi-agent-worktree-and-runtime-hygiene.md"
  - "crates/xgauntlet-core/src/features/evidence/**"
  - "crates/xgauntlet-core/src/features/checkpoint/**"
  - "crates/xgauntlet-core/src/features/adapters/**"
  - "crates/xgauntlet-core/src/features/telemetry/**"
  - "crates/xgauntlet-core/src/features/config/**"
  - "crates/xgauntlet-core/src/features/tasks/**"
  - "crates/xgauntlet-core/src/features/gauntlet/**"
  - "crates/gauntlet-policy-engine/src/**"
  - "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
  - "crates/xgauntlet-cli/src/**"
  - "docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md"
  - "specs/adapters.md"
  - "specs/checkpoint.md"
  - "specs/telemetry.md"
  - "specs/tasks.md"
  - "spec.md"
  - "CONTEXT.md"
test_refs:
  - "crates/xgauntlet-core/tests/task_093_hygiene_and_coexistence_test.rs"
  - "crates/xgauntlet-core/tests/multi_agent_worktree_test.rs"
  - "crates/xgauntlet-cli/tests/worktree_cli_test.rs"
---

# Task 093: Multi-Agent Worktree Workflow, Coexistence Policy & Runtime/Evidence Hygiene

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-11-15`

---

## 🎯 Formål

At bringe [Task 088](archive/088-worktree-isolated-multi-agent-supervision.md) og [ADR 0014](../docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md) i fuld praktisk samklang med Antigravitys multi-agent orkestratorer (`invoke_subagent`, `/swarm`, `/boost`) og de konkrete erfaringer fra Kant-repoet:

1. **`.xgauntlet/` & `.worktrees/` Runtime-Isolation (`verify` + `checkpoint`):**
   - Udeluk `.xgauntlet` og `.worktrees` i `EXCLUDED_DIRS` (`crates/xgauntlet-core/src/features/evidence/manifest.rs`), så runtime-state aldrig udløser `[xgauntlet-self-mutation]`.
   - Ekskludér `.xgauntlet/` og `.worktrees/` i `stage_workspace_changes` (`crates/xgauntlet-core/src/features/checkpoint/git.rs`), så tidligere trackede runtime-filer aldrig auto-committes af `checkpoint`.
2. **Clean Working Tree efter `xgauntlet checkpoint --phase done`:**
   - Bevar `SessionLineageEntry` (med `commit_oid`) i `verification-report.json` og `evidence.md`, og hvis disse filer er tracked af Git og ændres post-commit, forsegles de automatisk i et rent lokalt opfølgningscommit (`chore(<id>): seal post-commit evidence [EVIDENCE]`), så `git status` er 100% clean efter `DONE`.
3. **Race-Free Per-Session Telemetri (`.xgauntlet/sessions/`):**
   - Eksplicitte `session_id`-nøgler gemmes atomisk (temp-fil + rename) i deres egen fil `.xgauntlet/sessions/<hash>.json` (med fuld bagudkompatibilitet for `.xgauntlet/session_state.json` ved `"default"`/single-session), og `reset_session_health` nulstiller ikke længere andre samtidige sessioners filer.
4. **Antigravity Subagent-, Swarm- & Worktree-Coexistence:**
   - **Read-Only Subagenter:** `browser` og `image-generator` samt custom `.agents/agents/<name>.md` (eller `define_subagent`-payloads) med udelukkende read-only `tools` klassificeres som `"read_only"` i stedet for `4040`.
   - **`Workspace: "branch"`-kloner:** `resolve_effective_workspace` / `is_same_git_repo_worktree` i `adapters/workspace.rs` genkender sikkert lokale `git clone`-workspaces (via `.git/objects/info/alternates` eller matching `remote.origin.url`), mens fremmede repos fortsat afvises fail-closed.
   - **Opt-In Coordinated `inherit` (`[multi_agent]` i `gauntlet.toml`):** `MultiAgentConfig` (`allow_coordinated_inherit`, `share_cargo_target_dir`) tilføjes til `GauntletConfig`. Standard `allow_coordinated_inherit = false` bevarer streng ADR 0014 `4040`-spærre som default; `allow_coordinated_inherit = true` tillader koordineret `/swarm`/`/boost` fan-out i `inherit` under en aktiv task.
   - **Shared `CARGO_TARGET_DIR` & `xgauntlet worktree create <task-id>`:** Når `share_cargo_target_dir = true` i en linked `git worktree` uden eksplicit `CARGO_TARGET_DIR` i miljøet, peger `xgauntlet` automatisk `CARGO_TARGET_DIR` på `<primary_repo_root>/target` for at undgå ~40 GB `target/`-duplikering per worktree. CLI udvides med `xgauntlet worktree create <task-id>` (opretter `.worktrees/<task-id>/` og binder `.xgauntlet/active_task` automatisk).

---

## 📋 Acceptance Criteria

- [ ] **AC1 — `.xgauntlet/` & `.worktrees/` ekskluderet fra Manifest & Checkpoint Staging:**
  - `EXCLUDED_DIRS` i `crates/xgauntlet-core/src/features/evidence/manifest.rs` indeholder `".xgauntlet"` og `".worktrees"`; ændringer i `.xgauntlet/session_state.json`, `.xgauntlet/sessions/*` eller `.worktrees/**` udløser aldrig `[xgauntlet-self-mutation]`.
  - `stage_workspace_changes` i `crates/xgauntlet-core/src/features/checkpoint/git.rs` ekskluderer `.xgauntlet/` og `.worktrees/` fra `git add -A`, selv hvis filer tidligere har været tracked i Git.
- [ ] **AC2 — Clean Working Tree efter `xgauntlet checkpoint --phase done`:**
  - `run_checkpoint` bevarer `SessionLineageEntry` med `DONE`-commit-OID i `verification-report.json` og `evidence.md`, og forsegler automatisk tracked `verification-report.json`/`evidence.md` hvis de er modified efter `DONE`-committet, så `git status --porcelain` er tom bagefter.
- [ ] **AC3 — Race-Free Per-Session Telemetri:**
  - Eksplicitte `session_id`-kald til `record_session_turn_for_session` / `record_session_snapshot_for_session` skriver atomisk til `.xgauntlet/sessions/<hash>.json` uden at overskrive samtidige sessioner.
  - `reset_session_health` fjerner kun legacy `.xgauntlet/session_state.json` (eller den relevante sessions data) og sletter ikke andre aktive sessioners tilstand.
- [ ] **AC4 — Antigravity Read-Only Subagenter, `branch`-kloner & Coordinated `inherit` Opt-In:**
  - `classify_delegation_tool` / adapter-normalisering tillader `invoke_subagent` med `TypeName` = `"research"`, `"browser"`, `"image-generator"` samt custom `.agents/agents/<name>.md` uden muterende værktøjer som `read_only`.
  - `resolve_effective_workspace` genkender lokale `Workspace: "branch"`-kloner (delte Git-alternates eller `origin`-sti mod parent repo), og afviser fortsat fremmede repos fail-closed.
  - `GauntletConfig.multi_agent.allow_coordinated_inherit` (default `false`) tillader ved `true` koordineret `inherit`-delegation under en aktiv task (mens `false` bevarer `4040`).
- [ ] **AC5 — Shared `CARGO_TARGET_DIR` & `xgauntlet worktree create`:**
  - `GauntletConfig.multi_agent.share_cargo_target_dir` peger automatisk `CARGO_TARGET_DIR` mod primary repoets `target/` ved `cargo`-kørsler i linked worktrees, når `CARGO_TARGET_DIR` ikke allerede er sat.
  - `xgauntlet worktree create <task-id>` opretter en linked git worktree under `.worktrees/<task-id>/` og binder `.xgauntlet/active_task` til `<task-id>`.
  - `check-spec -t 093`, `check-spec --context`, `check-adr --all`, `cargo test --workspace` og `xgauntlet verify` passerer.

---

## 🚫 Must NOT

- **Må IKKE** bryde Zero-Daemon-princippet ([ADR 0007](../docs/adr/0007-local-transparent-supervisor-and-wasm-verifier.md)) eller introducere baggrundstjenester/sockets/permanente lock-filer.
- **Må IKKE** svække default `4040`-spærren i [ADR 0014](../docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md) uden eksplicit opt-in (`allow_coordinated_inherit = true` i `gauntlet.toml`).
- **Må IKKE** tillade fremmede repositories i hook-payloadens `cwd`/`workspacePaths` at hijacke `EnforcementContext` ([ADR 0006](../docs/adr/0006-multi-harness-policy-adapter-contract.md)).
- **Må IKKE** udføre `git push`, `git reset --hard` eller `git clean -f` ([ADR 0003](../docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md)).
- **Må IKKE** kompromittere `Self-Mutation Invariant` for faktiske kilde-, test- eller konfigurationsfiler eller oprette monolitiske testfiler (>1.000 linjer).

---

## 📝 Revisions

- 2026-11-15: Oprettet `DRAFT`-opgavepakke efter indledende `discover`-gennemgang og empirisk kodeanalyse af observationer fra Kant-repoet.
- 2026-11-15: Indarbejdet indsigter fra Antigravity-dokumentationen (`Subagents`, `Orchestrator`, `/swarm`, `/boost`, `/teamwork`) samt race-condition-fund i `telemetry/session.rs`.
- 2026-11-15: Bruger godkendte sammenkobling; design låst i `SPEC` og opgave aktiveret (`ACTIVE`) med opdatering af `ADR 0014` Addendum og `MultiAgentConfig`.

---

## 🧪 Verifikation

- `cargo run -p xgauntlet-cli -- check-spec -t 093`
- `cargo run -p xgauntlet-cli -- check-spec --context`
- `cargo run -p xgauntlet-cli -- check-adr --all`
- `cargo test -p xgauntlet-core --test task_093_hygiene_and_coexistence_test`
- `cargo test -p xgauntlet-core --test multi_agent_worktree_test`
- `cargo run -p xgauntlet-cli -- verify --task 093`
