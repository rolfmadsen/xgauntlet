---
type: System Specification
title: Specification - xGauntlet System Architecture & Capabilities
description: Macro system architecture, philosophy, and invariants for xGauntlet
status: stable
generated: { by: process:agent-gauntlet-init, at: "2026-09-06T16:15:00Z" }
tags: [specification, architecture, invariants, wasm, rust, npx]
---

# Specification: xGauntlet System Architecture & Capabilities

## 🎯 Philosophy & Core Principles

1. **Zero-Daemon & Low-Latency Execution**:
   - Single standalone native Rust binary (`xgauntlet`) with sub-3ms cold start.
   - Zero background services, zero daemons, zero socket files, zero port conflicts.
   - Cross-platform parity across Linux (x86_64, aarch64), macOS (Apple Silicon, Intel), and Windows 11 (x86_64).

2. **Zero Ambient Authority WASM Policy Engine**:
   - Deterministic capability evaluation governed by versioned WIT contract (`wit/gauntlet_policy.wit`).
   - Pure functional execution via embedded Wasmtime in memory.
   - Zero capabilities granted: no filesystem, no network, no clock, no random, no secrets.

3. **Mathematical Anti-Tamper Manifest & Two-Tier Trust Model (ADR 0005)**:
   - Self-mutation invariant: Pre- and post-test verification of Git-blob SHA-256 digests.
   - Any modification of source code, test assertions, or configuration during verification triggers an immediate tamper failure.
   - Local verification generates self-contained `verification-report.json` and updates `evidence.md`.
   - CI runner provides authoritative cryptographic Sigstore OIDC attestation.

4. **Uncle Bob Clean Architecture & TDD**:
   - Package-by-Feature (Screaming Architecture).
   - Strict Red -> Green -> Refactor lifecycle.
   - Multi-layer gauntlet: Linters, typecheckers, unit/feature tests, invariant tests, and mutation gauntlet (`cargo-mutants`).

5. **Multi-Harness Native Support**:
   - Autonomous vertical slices for Google Antigravity IDE, Claude Code, and OpenAI Codex.

---

## 📐 Architecture & Feature Modules (`Package-by-Feature`)

- `crates/gauntlet-policy-engine`: Deterministic WebAssembly component implementing `wit/gauntlet_policy.wit`.
- `crates/xgauntlet-core/src/features/`:
  - `wasm`: Embedded Wasmtime runtime host (`WasmRuntimeHost`). In-memory WebAssembly module instantiation with Zero Ambient Authority (no filesystem, no network, no clock, no random). Exposes linear memory bridge (`alloc`, `dealloc`, `evaluate_json_wasm`).
  - `policy`: Capability request and trusted context evaluator (`CapabilityRequest`, `EnforcementContext`, `PolicyDecision`, `PolicyEvaluator`, `WasmPolicyEngine`). Enforces task-binding, path protection, destructive command denials, and verification whitelists.
  - `evidence`: Canonical workspace manifest, report generation, drift detection, and release gate.
  - `gauntlet`: Multi-layer process runner (`std::process::Command`), timeouts, exit code handling.
  - `diagnostics`: Actionable diagnostics parser transforming compiler/test output into structured LLM hints.
  - `tasks`: OKF v0.2 Markdown parser, task binding, and Aristotelian glossary validation (`check-spec`).
  - `adapters`: Vertical slices for Antigravity, Claude Code, and OpenAI Codex.
  - `config`: Declarative `gauntlet.toml` schema and stack profiles.
  - `scaffold`: Safe, non-destructive project bootstrap engine (`xgauntlet init`).
  - `doctor`: Fast environment, Git, and toolchain diagnostics.
  - `release`: Release readiness gatekeeper, manifest version harmony, CHANGELOG.md verification, and ADR coverage validation (`check-release`).
- `crates/xgauntlet-cli`: Native CLI frontend.
- `packages/cli`: Zero-dependency Node.js distribution launcher (`xgauntlet` / `npx xgauntlet`).

---

## 🚫 Must NOT (System Invariants)

- Må IKKE introducere OS-specifikke dæmoner eller service managers (`systemd`, `launchd`, Windows Service).
- WebAssembly policy-motoren må IKKE tildeles ambient capabilities (intet netværk, filsystem, system-ur eller tilfældighedskilder).
- LLM-agenten må ALDRIG kunne omgå eller manipulere verifikationsresultater uden at udløse en manifest-fejl.
- Må IKKE foretage utilsigtede remote publication kommandoer (`git push`).
- Må IKKE introducere udokumenterede eksterne runtime-afhængigheder (f.eks. Python).

---

## 🧪 Multi-Layer Verification Contracts

- [ ] 100% test pass rate på tværs af samtlige unit- og feature-suiter på Linux, macOS og Windows.
- [ ] 100% mutation kill-rate på policy engine og manifest drift detection.
- [ ] Sub-5ms koldstart på hook capability evaluation.
- [ ] Sub-50ms diagnostisk inspektion for doctor subsystemet uden eksterne runtime-dæmoner.
- [ ] Sub-5ms koldstarts-evaluering og mekanisk validering af release readiness via `check-release`.
