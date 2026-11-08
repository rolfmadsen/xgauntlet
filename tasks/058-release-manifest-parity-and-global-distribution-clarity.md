---
type: Task Package
title: "Task 058: Release Manifest Parity (.agents Plugin & Policy) and Global Distribution Clarity"
description: "Sikrer at .agents/plugins/xgauntlet/plugin.json og policy.json inkluderes i extract_declared_versions og update_manifest_versions, opdaterer metadata til v0.12.0, og præciserer global installation (--force) vs projekt-init i dokumentationen."
status: done
intent: bug
generated: { by: process:antigravity-task-init, at: "2026-11-07T20:10:00Z" }
tags: [release, manifests, plugin, bundle, documentation, parity]
scope:
  - "crates/xgauntlet-core/src/features/release/**"
  - ".agents/plugins/xgauntlet/plugin.json"
  - ".agents/plugins/xgauntlet/policy.json"
  - "crates/xgauntlet-core/tests/**"
  - "README.md"
  - "packages/cli/README.md"
  - "tasks/058-release-manifest-parity-and-global-distribution-clarity.md"
test_refs:
  - "crates/xgauntlet-core/tests/release_manifest_parity_test.rs"
---

# Task 058: Release Manifest Parity (.agents Plugin & Policy) and Global Distribution Clarity

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-07`

---

## 🎯 Formål & Baggrund

Audit af installationen og udrulningen af version `0.12.0` afslørede to væsentlige uklarheder og en konkret metadata-synkroniseringsdefekt:
1. **Metadata Version Drift i `plugin.json` og `policy.json`:**
   Når `xgauntlet release bump` / `release prepare` afvikles, opdaterer `update_manifest_versions` udelukkende `Cargo.toml`, `package.json` og `pyproject.toml`.
   Eftersom [`.agents/plugins/xgauntlet/plugin.json`](.agents/plugins/xgauntlet/plugin.json) indlejres i Rust-binæren ved kompilering via `include_str!`, blev `plugin.json` efterladt på version `0.8.4` under udgivelsen af 0.9.0–0.12.0. Når brugere efterfølgende kører `xgauntlet plugin install --global --force`, skrives et manifest ud med `"version": "0.8.4"`, selvom de 6 skills er opdateret til `0.12.0`.
2. **Dokumentationsklarhed: Global vs Projekt-Lokal Scaffolding:**
   Der er behov for at tydeliggøre forskellen på `xgauntlet plugin install --global` (der installerer/opdaterer de 6 skills i AI-agenternes globale mapper, og kræver `--force` for at overskrive eksisterende installationer ved opgradering) versus `xgauntlet init` (der udelukkende opretter in-repo styringsfiler som `gauntlet.toml`, `CONTEXT.md` og `spec.md`, og *ikke* duplikerer skills ind i projektmapper).

---

## 📋 Acceptance Criteria

- [x] **AC1 — Release Manifest Extraction & Version Bump Parity:**
  - `extract_declared_versions` og `update_manifest_versions` i `crates/xgauntlet-core/src/features/release/manifests.rs` skal inkludere:
    - `.agents/plugins/xgauntlet/plugin.json` (nøglen `"version"`)
    - `.agents/plugins/xgauntlet/policy.json` (nøglen `"version"`)
  - Både kildekoden i `.agents/plugins/xgauntlet/plugin.json` og `policy.json` opdateres til `"0.12.0"`.
- [x] **AC2 — Manifest Harmony Gate Invariant:**
  - `xgauntlet check-release` og versionsvalidering skal håndhæve paritet, så enhver fremtidig release-drift mellem Cargo, npm og `.agents/plugins/xgauntlet/plugin.json` fejler lukket (fail-closed).
- [x] **AC3 — Dokumentation i GitHub & npm README:**
  - Opdater både `README.md` og `packages/cli/README.md` med:
    - Eksplicit forklaring af opgraderingsproceduren (`xgauntlet plugin install --global --force`), herunder hvorfor `--force` kræves for at overskrive eksisterende skills.
    - Klar sondring mellem globale agent-skills (`~/.gemini/config/plugins/xgauntlet/`, `~/.claude/` osv.) og projekt-lokal styring via `xgauntlet init` (`gauntlet.toml`, `CONTEXT.md`, `spec.md`), så det fremgår, at projekter ikke skal indeholde egne skill-kopier.
- [x] **AC4 — Regressions- og Enhedstest:**
  - Opret `crates/xgauntlet-core/tests/release_manifest_parity_test.rs`, der beviser at `plugin.json` og `policy.json` ekstraheres, harmoniseres og opdateres korrekt af manifest-motoren.

---

## 🚫 Must NOT

- Må IKKE bryde bagudkompatibilitet for eksisterende `Cargo.toml`, `package.json` eller `pyproject.toml` version-håndtering.
- Må IKKE ændre på strukturen eller ekstra nøgler i `plugin.json` og `policy.json` under version-bump.
- Må IKKE udføre remote publication handlinger (`git push`) jf. ADR 0003.

---

## 📝 Revisions

- 2026-11-07: Oprettet Task 058 efter opdagelse af metadata version-drift i `plugin.json` og behov for præcisering af global vs lokal installation.

---

## 🧪 Verifikation

- `cargo run -p xgauntlet-cli -- check-spec -t 058`
- `cargo test --test release_manifest_parity_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `xgauntlet check-spec`
