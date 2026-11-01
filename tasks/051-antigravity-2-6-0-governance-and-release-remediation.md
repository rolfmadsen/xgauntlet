---
type: Task Package
title: "Task 051: Antigravity 2.6.0 Governance & Strict Release Remediation"
description: "Udbedring af de 4 code review-fund fra v0.9.0..07c6cd9 (ADR 0006 workspace-containment, Single-Agent tool-gating, strict in-flight release gatekeeper og genforsegling af v0.10.0)"
status: active
generated: { by: process:antigravity-task-init, at: "2026-11-01T14:35:00Z" }
tags: [antigravity, security, governance, adr-0006, release, semver, bugfix]
scope:
  - "crates/xgauntlet-core/src/features/adapters/antigravity/**"
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-core/src/lib.rs"
  - "crates/xgauntlet-cli/src/**"
  - "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
  - "Cargo.toml"
  - "Cargo.lock"
  - "package.json"
  - "packages/cli/package.json"
  - "CHANGELOG.md"
  - "verification-report.json"
  - "evidence.md"
  - "tasks/050-antigravity-ide-2-6-0-support-and-release.md"
  - "tasks/051-antigravity-2-6-0-governance-and-release-remediation.md"
test_refs:
  - "crates/xgauntlet-core/tests/antigravity_adapter_test.rs"
  - "crates/xgauntlet-core/tests/release_in_flight_test.rs"
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
---

# Task 051: Antigravity 2.6.0 Governance & Strict Release Remediation

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-01`

## 🎯 Formål
Two-Axis Code Review af `v0.9.0..07c6cd9` identificerede 4 kontrakt- og formålsbrud i forbindelse med opdateringen til Antigravity IDE 2.6.0 og forberedelsen af `v0.10.0` (som endnu kun ligger lokalt og ikke er pushet):
1. `resolve_effective_workspace` udleder fallback-workspace fra det uprivilegerede tool-argument (`toolCall.args.Path`), hvilket bryder ADR 0006 og ophæver workspace-containment på tværs af lokale Git-repositories.
2. `AntigravityAdapter::normalize_tool_call` whitelister subagent-, besked- og cron/dæmon-værktøjer samt muterende `manage_task`-handlinger som ubegrænset `ReadFile` uden aktiv task, i modstrid med Single-Agent HITL-modellen (Task 043) og Zero-Daemon princippet (`spec.md`).
3. `execute_release_workflow` i `render.rs` ændrede `strict: true` til `strict: false`, hvilket slog valideringen af `verification-report.json` og 0% workspace-drift fra under `release prepare`/`bump` (brud på Task 042 AC3) samt ignorerede fejl fra `cargo check`.
4. `Commands::Telemetry` læser fra `stdin` før formatet tjekkes, og `Task 050` står fortsat som `ACTIVE` med ufuldstændig `scope`-deklaration inde i det lokale `v0.10.0`-tag.

Formålet med Task 051 er at udbedre disse 4 punkter via streng TDD (RED → GREEN → REFACTOR) og genforsegle det lokale `v0.10.0`-tag med 0% workspace-drift klar til `git push`.

## 📋 Acceptance Criteria
- [x] **AC1 (ADR 0006 Trusted Workspace Containment)**: Fjern Trin 3 (`toolCall.args.Path` forældretraversering) fuldstændigt fra `crates/xgauntlet-core/src/features/adapters/antigravity/workspace.rs`. Bevar det indkommende `workspace` uændret, når det allerede er et gyldigt workspace (`gauntlet.toml`, `tasks/` eller `.agents/`), og brug kun konvoluttens `workspacePaths` / `cwd` som fallback, når proces-`workspace` ikke selv er et gyldigt workspace. En accepttest i `antigravity_adapter_test.rs` beviser, at et `PreToolUse`-kald (`write_file` / `edit_file`) mod en absolut sti i et andet Git-repo på disken afvises fail-closed (`decision: "deny"`).
- [x] **AC2 (Single-Agent HITL & Zero-Daemon Tool Gating)**: I `AntigravityAdapter::normalize_tool_call` (`crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs`) mappes kun reelt passive/interaktive læse- og dialogværktøjer uden filsti til `ToolActionType::ReadFile`: `ask_question`, `list_plugin_accounts`, `search_marketplace`, `search_web` samt `manage_task` *udelukkende* når `args.Action` (case-insensitivt) er `"list"` eller `"status"`. Værktøjerne `invoke_subagent`, `define_subagent`, `manage_subagents`, `send_message` og `schedule` samt `manage_task` med `Action: "kill" | "send_input"` (eller manglende/ukendt `Action`) skal mappes til `ToolActionType::Other` (fail-closed `4038: Deny`). Verificeres i `antigravity_adapter_test.rs`.
- [x] **AC3 (Strict In-Flight Release Gatekeeper & Fail-Closed `cargo check`)**: Gendan strict kontrol af `verification-report.json` og 0% manifest-drift (`verify_workspace_state_match`) i trin 7 af `execute_release_workflow` (`crates/xgauntlet-cli/src/render.rs` og `crates/xgauntlet-core/src/features/release/`) jf. Task 042 AC3 uden at bryde bagudkompatibiliteten på `ReleaseReadinessOptions` (kun `DirtyWorktree`-tjekket må springes over under in-flight transaktionel staging, mens `check-release --strict` fortsat håndhæver begge). Fejl på `cargo check` under `Cargo.lock`-synkronisering i `render.rs` håndteres fail-closed i stedet for `let _ = ...`. Dækkes af tests i `release_in_flight_test.rs` (eller `release_engine_test.rs` inden for 1.000-linjers loftet).
- [x] **AC4 (Isoleret Hook-Stdin i `Commands::Telemetry`)**: I `crates/xgauntlet-cli/src/main.rs` udledes `effective_format` *før* der læses fra `stdin`, så `stdin` udelukkende læses (via `clean_stdin` mod UTF-8 BOM) når `effective_format` er `antigravity-hook` / `antigravity`. Almindelige CLI-/script-kald (`box`, `json` m.fl.) og andre harnesses (jf. ADR 0004) må ikke læse `stdin` eller kalde `AntigravityAdapter`.
- [x] **AC5 (Ærlig opgavehistorik for Task 050)**: `tasks/050-antigravity-ide-2-6-0-support-and-release.md` opdateres til `status: done` og `**Status**: DONE`, de manglende berørte stier (`crates/xgauntlet-core/src/features/policy/path.rs`, `crates/xgauntlet-core/src/features/adapters/models.rs`, `package.json`) tilføjes i Task 050's `scope`, og `AC5` i Task 050 sættes til `[ ]` med en revisionsnote, der henviser til Task 051.
- [ ] **AC6 (Genforsegling af Task 051 & Lokalt `v0.10.0` Tag)**: Task 051 markeres `status: done` / `**Status**: DONE`, frisk evidens forsegles med 0% drift i `verification-report.json` og `evidence.md`, det lokale u-pushede tag `v0.10.0` flyttes til det endelige forseglede release-commit (`git tag -d v0.10.0` og `git tag -a v0.10.0 -m "Release v0.10.0"`), og `xgauntlet check-release --strict` passerer med exit code 0 (`RELEASE READY`) på rent arbejdstræ.

## 🚫 Must NOT
- Må ALDRIG udføre remote publication (`git push`) jf. ADR 0003.
- Må ALDRIG udføre destruktive git-handlinger (`git reset --hard`, `git checkout .`, `git clean -f`).
- Må IKKE udlede eller tilsidesætte `EnforcementContext` workspace ud fra uprivilegerede tool-argumenter (`Path`, `TargetFile`, `file_path` m.fl.) jf. ADR 0006.
- Må IKKE bryde serde- eller struct-bagudkompatibilitet for eksisterende felter i `ReleaseReadinessOptions`.
- Må IKKE introducere "født grønne" tests (`born-green tests`); alle nye accepttests skal bevises røde i `[RED]` før produktionskode ændres i `[GREEN]`.
- Må IKKE overskride arkitekturlofterne i `gauntlet.toml` (kildefiler <= 800 linjer, testfiler <= 1.000 linjer, ingen inflation af ratchets jf. ADR 0010).

## 📝 Revisions
- 2026-11-01: Oprettet på baggrund af Two-Axis Code Review af `v0.9.0..07c6cd9` for at sikre fuld overensstemmelse med xGauntlets formål før publicering af `v0.10.0`.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 051`
- `cargo test -p xgauntlet-core --test antigravity_adapter_test`
- `cargo test -p xgauntlet-core --test release_in_flight_test`
- `cargo test -p xgauntlet-core --test release_engine_test`
- `cargo run -p xgauntlet-cli -- check-architecture`
- `cargo test --workspace`
- `cargo run -p xgauntlet-cli -- check-release --strict`

