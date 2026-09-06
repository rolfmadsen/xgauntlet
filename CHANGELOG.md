# Changelog

All notable changes to the `xGauntlet` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-09-06

### 🚀 Added
- **Zero-Daemon WASM Policy Verifier (`Task 002`)**:
  - Embedded zero-ambient-authority WebAssembly policy engine (`crates/gauntlet-policy-engine`) evaluated via in-memory Wasmtime host (`WasmRuntimeHost`).
  - Strict denials for destructive commands (`git push`, `rm -rf`, `git reset --hard`) and task-bound write authorizations.
- **Specification, Task & Aristotelian Glossary Gatekeeper (`Task 003`)**:
  - Shift-left CLI subcommand `check-spec` validating Open Knowledge Format (OKF) task packages and glossary invariants.
- **Evidence & Workspace Manifest Engine (`Task 004`)**:
  - Deterministic Canonical Workspace Manifest with Git-blob SHA-256 multi-digests and CRLF/LF line-ending immunity.
  - Two-Tier Trust Model (`LOCAL_UNATTESTED` vs `CI_ATTESTED`) and CLI subcommand `check-evidence`.
- **Gauntlet Execution Engine (`Task 005`)**:
  - Multi-layer fail-closed verification pipeline (`xgauntlet verify`) with timeouts and self-mutation invariant enforcement.
- **Vertical Slice Harness Adapters (`Task 006`)**:
  - Universal Pre-Tool Use interceptors and plugin validators for Google Antigravity, Claude Code, and OpenAI Codex.
- **Declarative Configuration & Multi-Stack Profiles (`Task 007`)**:
  - Schema validation for `gauntlet.toml` with presets for Rust, TypeScript/Node.js, Python, and Go via `check-config`.
- **Safe Non-Destructive Project Bootstrap Engine (`Task 008`)**:
  - Safe scaffolding subcommand `xgauntlet init` generating in-repo governance files without overwriting existing assets unless `--force` is given.
- **Fast Environment, Git & Toolchain Diagnostics (`Task 009`)**:
  - High-speed subcommand `xgauntlet doctor` performing sub-50ms host, git, governance, toolchain, and engine inspections.
- **Release Readiness Gatekeeper & Manifest Harmony Engine (`Task 010`)**:
  - Mechanical release gate `xgauntlet check-release` validating version harmony across manifests (`Cargo.toml`, `package.json`, `pyproject.toml`), `CHANGELOG.md` synchronization, and ADR coverage across `README.md` and `spec.md`.

### 🛡️ Hardened & Refactored
- **Code Review & Architecture Audit Remediation (`Task 011`)**:
  - Eliminated `.unwrap()` calls in production code across evidence manifest and release engine.
  - Consolidated ISO 8601 UTC timestamp generation into `features/evidence/time.rs`.
  - Fixed rustdoc intra-doc link warnings in `features/release/models.rs`.
  - Removed deprecated daemon documentation (`docs/supervisor-systemd.md`) aligning strictly with Zero-Daemon architecture (ADR 0007).
