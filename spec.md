---
type: System Specification
title: Specification - xGauntlet System Architecture & Capabilities
description: Macro system architecture, philosophy, and invariants for xGauntlet
status: stable
generated: { by: process:xgauntlet-init, at: "2026-09-06T16:15:00Z" }
tags: [specification, architecture, invariants, wasm, rust, npx]
---

# Specification: xGauntlet System Architecture & Capabilities

## 🎯 Philosophy & Core Principles

1. **Zero-Daemon & Low-Latency Execution**:
   - Single standalone native Rust binary (`xgauntlet`) with sub-3ms cold start.
   - Zero background services, zero daemons, zero socket files, zero port conflicts.
   - Cross-platform parity across Linux (x86_64, aarch64), macOS (Apple Silicon, Intel), and Windows 11 (x86_64).

2. **Zero Ambient Authority WASM Policy Engine & Structured Tokenization**:
   - Deterministic capability evaluation governed by versioned WIT contract (`wit/gauntlet_policy.wit`).
   - Pure functional execution via embedded Wasmtime in memory.
   - Zero capabilities granted: no filesystem, no network, no clock, no random, no secrets.
   - Structured quote-aware shell command tokenization, fail-closed wrapper rejection (`sudo`, `env`, `eval`, `bash -c`), and task scope isolation.

3. **Mathematical Anti-Tamper Manifest & Two-Tier Trust Model (ADR 0005)**:
   - Self-mutation invariant: Pre- and post-test verification of Git-blob SHA-256 digests.
   - Any modification of source code, test assertions, or configuration during verification triggers an immediate tamper failure.
   - Local verification generates self-contained `verification-report.json`, updates `evidence.md`, and maintains an append-only cryptographically chained `SessionLineageTrail` across multi-session handoffs.
   - CI runner provides authoritative cryptographic Sigstore OIDC attestation (`attestation.bundle` / `attestation.json`) verified at install-time by bootstrapper and CLI (`check-attestation`).

4. **Uncle Bob Clean Architecture & TDD**:
   - Package-by-Feature (Screaming Architecture).
   - Strict Red -> Green -> Refactor lifecycle and RED-first test provenance enforcement (elimination of born-green and tautological tests).
   - Autonomous TDD inner loop with boundary Human-in-the-Loop gates ([ADR 0009](docs/adr/0009-autonomous-tdd-inner-loop-and-boundary-human-gates.md)).
   - Multi-layer gauntlet: Linters, typecheckers, unit/feature tests, invariant tests, and mutation gauntlet (`cargo-mutants`).

5. **Multi-Harness Native Support**:
   - Autonomous vertical slices for Google Antigravity IDE, Claude Code, OpenAI Codex, and Mistral Vibe.

---

## 📐 Architecture & Feature Modules (`Package-by-Feature`)

- `crates/gauntlet-policy-engine`: Deterministic WebAssembly component implementing `wit/gauntlet_policy.wit`. Structured quote-aware command tokenization, wrapper rejection, fail-closed task scope enforcement (`allowed_scopes`), and test scope isolation (`allowed_test_targets`).
- `crates/xgauntlet-core/src/features/`:
  - `wasm`: Embedded Wasmtime runtime host (`WasmRuntimeHost`). In-memory WebAssembly module instantiation with Zero Ambient Authority (no filesystem, no network, no clock, no random). Exposes linear memory bridge (`alloc`, `dealloc`, `evaluate_json_wasm`).
  - `policy`: Capability request and trusted context evaluator (`CapabilityRequest`, `EnforcementContext`, `PolicyDecision`, `PolicyEvaluator`, `WasmPolicyEngine`). Enforces task-binding, path protection, destructive command denials, and verification whitelists.
  - `evidence`: Canonical workspace manifest, multi-session tamper-evident lineage trail, report generation, drift detection, and cryptographic release attestation verification (`xgauntlet check-attestation`).
  - `gauntlet`: Multi-layer process runner (`std::process::Command`), timeouts, exit code handling.
  - `diagnostics`: Actionable diagnostics parser transforming compiler/test output into structured LLM hints.
  - `tasks`: OKF v0.2 Markdown parser, task binding, and Aristotelian glossary validation (`check-spec`).
  - `adapters`: Vertical slices for Antigravity, Claude Code, OpenAI Codex, and Mistral Vibe with flat `PreInvocation` schema compliance and non-destructive scaffolding.
  - `config`: Declarative `gauntlet.toml` schema and stack profiles.
  - `scaffold`: Safe, non-destructive project bootstrap engine (`xgauntlet init`).
  - `doctor`: Fast environment, Git, and toolchain diagnostics.
  - `release`: Release readiness gatekeeper (`check-release --strict`), manifest version harmony, CHANGELOG.md verification, ADR coverage validation, and transactional SemVer release manager (`release bump`) with in-process wasm32 compilation and non-destructive disk-staging backup/rollback.
  - `plugin`: Global plugin distribution and embedded 6-skill sovereign suite (`xgauntlet plugin install`).
  - `topology`: Deterministic AST codebase topology, graph traversals, and token-optimized blast-radius discovery engine (`xgauntlet topology`).
- `crates/xgauntlet-cli`: Native CLI frontend (`verify`, `check-spec`, `check-evidence`, `check-release`, `check-attestation`, `check-architecture`, `release bump`, `handoff`, `evidence lineage`).
- `packages/cli`: Zero-dependency Node.js distribution launcher (`xgauntlet` / `npx xgauntlet`) with cryptographic Sigstore attestation integrity verification against GitHub Repository ID (`1359184794`) before caching/execution.

---

## 🚫 Must NOT (System Invariants)

- Må IKKE introducere OS-specifikke dæmoner eller service managers (`systemd`, `launchd`, Windows Service).
- WebAssembly policy-motoren må IKKE tildeles ambient capabilities (intet netværk, filsystem, system-ur eller tilfældighedskilder).
- WebAssembly policy-motoren må IKKE tillade eksekverings-wrappers (`env`, `sudo`, `eval`, `bash -c`, `sh -c`, `xargs`, `doas`, `su`, `exec`) til at omgå restriktioner (`reason_code: 4039`).
- LLM-agenten må ALDRIG kunne omgå eller manipulere verifikationsresultater uden at udløse en manifest-fejl.
- Må IKKE tillade filmodifikationer (`WriteFile`) uden for den aktive opgaves erklærede `scope` (`reason_code: 4037`), og må kun tillade testfiler når scope er tomt.
- Må IKKE foretage utilsigtede remote publication kommandoer (`git push`).
- Må IKKE foretage destruktive git operationer (`git reset --hard` eller `git clean -f`) ved release fejlhåndtering (skal anvende atomisk disk-staging og transaktionel rollback).
- Node-bootstrapperen må IKKE eksekvere eller udpakke native binærer uden gyldig kryptografisk Sigstore attestation (fail-closed integrity).
- Må IKKE introducere udokumenterede eksterne runtime-afhængigheder (f.eks. Python).
- Må IKKE tillade "født grønne" tests (`born-green tests`) at passere ind i et `[GREEN]` eller `[REFACTOR]` checkpoint uden verificeret negativt fejlbevis i RED-fasen eller via ikke-destruktiv inverse preflight.

---

## 🧪 Multi-Layer Verification Contracts

- [ ] 100% test pass rate på tværs af samtlige unit- og feature-suiter på Linux, macOS og Windows.
- [ ] 100% mutation kill-rate på policy engine og manifest drift detection.
- [ ] Sub-5ms koldstart på hook capability evaluation.
- [ ] Sub-50ms diagnostisk inspektion for doctor subsystemet uden eksterne runtime-dæmoner.
- [ ] Sub-5ms koldstarts-evaluering og mekanisk validering af release readiness via `check-release --strict`.
- [ ] 100% verifikation af kryptografiske Sigstore attesteringsbundter via `xgauntlet check-attestation`.
