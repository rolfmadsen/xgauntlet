# Changelog

All notable changes to the `xGauntlet` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.0] - 2026-09-09

### 🚀 Added
- **Multi-Harness Dynamic HUD Adapters (`Task 016`, `Task 017`, `Task 018`)**:
  - **Claude Code**: `PostToolUse` event-interception der udskriver live Variant B Unicode box-drawing telemetri-kort efter tool-kald.
  - **OpenAI Codex**: `PostToolUse` integration og response-wrapping (`wrap_response`) med adaptiv terminal/agent-telemetri.
  - **Google Antigravity**: `PreInvocation` hook-interceptor via `xgauntlet hook antigravity`, der injicerer dynamisk Blockquote HUD forud for agent-prompts.
  - CLI subcommand flags: Tilføjet `--harness <HARNESS>` til `verify` og `checkpoint` med automatisk telemetri-indkapsling.

### 🛡️ Hardened & Refactored
- **HUD & Harness Adapter Hardening & OS Resilience (`Task 019`)**:
  - Garanteret fast 64-tegns linjebredde i `render_box_card` via deterministisk felt-budgettering og `truncate_with_ellipsis`.
  - Sti-normalisering af Windows backslashes (`\`) til standard POSIX (`/`) i telemetri og task-referencer.
  - DRY konsolidering af fælles `PostToolUse` JSON hook merge-logik mellem Claude Code og Codex.
  - Typesikker `HarnessKind` med understøttelse af gængse aliaser (`claude`, `claude_code`, `codex`, `openai`, `openai_codex`, `antigravity`, `google_antigravity`).
  - Robust fejlhåndtering (`?`) og registrering af scaffoldede filer under `xgauntlet init`.
  - Automatisk opgradering af forældede python-hooks i `.agents/hooks.json` til `xgauntlet hook antigravity`.

## [0.3.0] - 2026-09-07

### 📚 Documentation & Universal Distribution
- **Universal Global Installation Model**:
  - Streamlined global CLI installation (`npm install -g xgauntlet`) as the single canonical standard across all tech stacks (Python, Go, Rust, TypeScript).
  - Clarified sub-3ms PreToolUse gatekeeper hook execution latency in `$PATH` without Node.js startup overhead.
- **Poly-Harness Sameksistens & Scaffolding Scope**:
  - Documented in `README.md` and `packages/cli/README.md` the clear boundary between the 7 universal in-repo governance files and the 2 harness-specific agent configs (`CLAUDE.md`, `.agents/`).
- **Surgical In-Repo Cleanup & Zero Lock-in**:
  - Added dedicated de-installation guide distinguishing between surgical in-repo governance cleanup (`rm -f gauntlet.toml .agents/hooks.json ...`) without touching developer tasks or ADRs, and global CLI uninstallation (`npm uninstall -g xgauntlet`).

## [0.2.0] - 2026-09-07

### 🚀 Added
- **Dynamic Cockpit HUD & TDD Checkpoint Protocol (`Task 012`)**:
  - Live 5-line status card HUD for agent responses with phase, criteria progress, git telemetry, and quick-links.
  - Strict local git commit protocol on TDD phase transitions (`SPEC`, `RED`, `GREEN`, `REFACTOR`, `DONE`) adhering to ADR 0003.
- **Task Lifecycle & Intent Scaffolding (`Task 013`)**:
  - CLI subcommand `xgauntlet task init <title>` with automatic slugification, number allocation, and OKF frontmatter template.
  - CLI subcommand `xgauntlet task list` displaying active/done status, intent, criteria progress, and metadata.
- **Phase Bound TDD Checkpoint Engine (`Task 014`)**:
  - CLI subcommand `xgauntlet checkpoint <phase>` enforcing phase transitions, clean working tree, test state verification, and atomic git commit creation.

### 🛡️ Hardened & Security
- **P0 Security & Policy Boundary Hardening (`Task 015`)**:
  - Integrated `WasmPolicyEngine` directly into harness adapters (`HarnessAdapter::evaluate_invocation`) executing in-memory Wasmtime evaluation with fail-closed semantics.
  - Path traversal and workspace containment defense via `WorkspaceRelativePath` blocking escape attempts across all adapters.
  - Command chaining defense (`&&`, `;`, `|`) blocking privilege escalation behind whitelisted command prefixes.
  - Fail-closed task binding eliminating synthetic default-task fallbacks in the gauntlet pipeline.
  - Typesafe serde deserialization inside the WebAssembly policy engine security boundary.
  - Expanded anti-tamper manifest scopes (`.agents/`, `.github/`, `CONTEXT.md`, `CODING_STANDARDS.md`) and exact digest matching.

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
