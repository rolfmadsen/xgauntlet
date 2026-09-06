---
type: Architectural Decision Record
title: 'ADR 0001: Package-by-Feature Architecture'
status: stable
tags:
- architecture
- adr
generated:
  by: human:maintainer
  at: '2026-08-23T10:00:00Z'
verified:
- by: human:maintainer
  at: '2026-08-23T10:00:00Z'
---

# 1. Package-by-Feature (Screaming Architecture)

**Status**: `accepted`  
**Date**: `2026-08-22`  

## Context
Tidligere var kildekoden opdelt i flade filer eller tekniske lag. Dette gjorde koden svær at navigere, øgede utilsigtet kobling og slørede domænegrænserne.

## Decision
Al domænelogik, forretningsregler og tilhørende tests i `xGauntlet` organiseres som **Package-by-Feature** under `crates/xgauntlet-core/src/features/<feature>/` og tilhørende feature-tests. Hver feature er et selvstændigt, høj-sammenhængende modul med sine egne typer, logik og enhedstests:
- `features/wasm`: Wasmtime runtime host.
- `features/policy`: Capability evaluation & trusted context.
- `features/evidence`: Manifest hashing, verification report & drift detection.
- `features/gauntlet`: Multi-layer process runner & timeout kontrol.
- `features/diagnostics`: Actionable diagnostics parser.
- `features/tasks`: OKF Markdown parser & check-spec.
- `features/adapters`: Vertikale harness-slices (Antigravity, Claude Code, Codex).
- `features/config`: gauntlet.toml loader & validering.
- `features/scaffold`: Ikke-destruktiv init-motor.
- `features/doctor`: Miljø- og toolchain-diagnostik.

Kerne-kassen `crates/xgauntlet-cli/src/main.rs` orkestrerer udelukkende disse feature-moduler uden egen forretningslogik. WebAssembly policy-motoren er isoleret i `crates/gauntlet-policy-engine`, og distributions-bootstrapperen i `packages/cli/`.

## Consequences
Nye funktioner må ikke tilføjes som løse filer i crate-roden, men skal placeres i en dedikeret feature-undermappe. Teststrukturen skal spejle kildestrukturen 1:1. Cargo workspace strukturen sikrer ren modularitet og isoleret kompilering.
