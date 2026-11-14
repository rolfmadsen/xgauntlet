---
type: Task Package
title: "Task 089: Sync Public README, NPM CLI README & CHANGELOG for Multi-Agent Release Communication"
description: "Opdaterer README.md, packages/cli/README.md (byte-paritet til npmjs) og CHANGELOG.md [Unreleased] med dokumentation for ADR 0014 / Task 088 (worktree-isolerede parallelle sessioner, delegation-policy og task bind/unbind)."
status: done
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-14T22:28:00Z" }
tags: [docs, readme, changelog, npm, adr-0014]
scope:
  - "README.md"
  - "packages/cli/README.md"
  - "CHANGELOG.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
  - "crates/xgauntlet-core/tests/single_agent_governance_test.rs"
---

# Task 089: Sync Public README, NPM CLI README & CHANGELOG for Multi-Agent Release Communication

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-14`

---

## 🎯 Formål

Opdatere de brugervendte udgivelsesdokumenter ([README.md](../README.md), [packages/cli/README.md](../packages/cli/README.md) og [CHANGELOG.md](../CHANGELOG.md)), så nye og eksisterende brugere på både GitHub og npmjs får klar og præcis formidling af multi-agent/worktree-supervision jf. [ADR 0014](../docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md) og [Task 088](088-worktree-isolated-multi-agent-supervision.md).

---

## 📋 Acceptance Criteria

- [x] **AC1 — README.md opdateret med ADR 0014 & Multi-Agent/Worktree DX:** Erstat tidligere single-agent-formulering med dokumentation for parallelle `git worktree`-sessioner (`xgauntlet task bind`/`unbind`), sikker subagent/swarm-delegation på tværs af alle 4 harnesses og link til `ADR 0014` i ADR-listen og CLI-referencen.
- [x] **AC2 — NPM Package README Byte-Paritet:** `packages/cli/README.md` synkroniseres byte-for-byte med root `README.md`, så `test_npm_cli_readme_identical_to_root_readme` passerer.
- [x] **AC3 — CHANGELOG.md `[Unreleased]`:** Dokumenter Task 088 / ADR 0014 under `## [Unreleased]` i Keep a Changelog-format uden at ændre tidligere frigivne versioner eller bumpe versionsnumre.

---

## 🚫 Must NOT

- **Må IKKE** bryde byte-for-byte pariteten mellem `README.md` og `packages/cli/README.md`.
- **Må IKKE** bumpe versionsnumre eller modificere tidligere frigivne sektioner i `CHANGELOG.md`.
- **Må IKKE** ændre produktionskode under `crates/**/src/`.

---

## 📝 Revisions

- 2026-11-14: Oprettet opgavepakke for offentlig formidling af ADR 0014 / Task 088 på GitHub og npmjs.

---

## 🧪 Verifikation

- `cargo run -p xgauntlet-cli -- check-spec -t 089`
- `cargo test -p xgauntlet-core --test release_manifest_parity_test`
- `cargo test -p xgauntlet-core --test release_engine_test`
