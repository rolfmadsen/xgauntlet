---
type: Knowledge Bundle Index
title: "xGauntlet Context & Domain Glossary"
description: "Kernebegreber, arkitekturgrænser og definitioner for xGauntlet"
status: stable
generated: { by: process:xgauntlet-init, at: "2026-09-06T16:22:00Z" }
tags: [glossary, domain-model, ubiquitous-language, okf, wasm, rust]
---

# xGauntlet Context & Domain Glossary

This document defines the core ubiquitous language for `xGauntlet` using Aristotle's formula (*definitio per genus et differentiam*). It captures domain concepts without implementation noise.

---

## 📖 Core Concepts

**Task**:
An executable unit of engineering work, that has bounded acceptance criteria and verifiable completion evidence.
_Avoid_: Ticket, issue, story, workitem.

**Layer**:
A verification step, that executes a specific analysis or testing command within a bounded timeout.
_Avoid_: Stage, phase, check-item.

**Gauntlet**:
A sequential verification pipeline, that executes verification layers with fail-closed semantics and halts on the first mandatory failure.
_Avoid_: Test runner, CI script, harness.

**Two-Tier Trust Boundary**:
A derived classification model, that categorizes verification records across two progressive trust boundaries: local cooperative drift detection (`LOCAL_UNSUPERVISED`) and cryptographically attested CI provenance (`CI_ATTESTED`).
_Avoid_: Security level, verification stage, permission rank.

**Zero-Daemon Engine**:
A standalone native binary engine, that executes capability evaluations and multi-layer gauntlets in-memory with sub-3ms latency without background daemons, socket activations, or ambient system processes.
_Avoid_: Background runner, helper daemon, supervisor process.

**WASM Policy Component**:
A zero-ambient-authority WebAssembly component, that deterministically evaluates capability requests against an immutable enforcement context without access to ambient system resources (filesystem, network, clock, random).
_Avoid_: Wasm sandbox, plugin module, custom script.

**Self-Mutation Invariant**:
A mathematical integrity contract, that compares pre- and post-verification workspace manifests and rejects the verification run if any source code, test assertions, or configuration files were altered during test execution.
_Avoid_: Code tamper, dirty tree check, test side-effect.

**Diagnostic Finding**:
A structured defect model, that pinpoints an exact file location, error category, message, and actionable remediation hint.
_Avoid_: Error log, raw stderr, traceback.

**Diagnostic Report**:
A consolidated finding summary, that groups diagnostic findings per executed layer.
_Avoid_: Test result, summary log.

**Stack Profile**:
A preconfigured collection of verification layers, that matches the conventions and toolchains of a specific programming language.
_Avoid_: Environment, runtime config.

**Canonical Workspace Manifest**:
A deterministic SHA-256 digest, that captures normalized Git-tree/blob OID hashes (immune to CRLF/LF conversions) and raw file contents across in-scope workspace paths.
_Avoid_: Git commit, workspace hash, checksum.

**Verification Report**:
An unsigned data record, that binds verification layer outcomes, diagnostic findings, and task contracts to the workspace manifest digests.
_Avoid_: Proof report, receipt, certification.

**Attestation Bundle**:
A detached cryptographic statement, that binds an authenticated, independent CI identity to verification reports and source digests via keyless OIDC and DSSE/in-toto envelopes.
_Avoid_: Local signature, HMAC receipt, inline certificate.

**Architecture Decision Record (ADR)**:
An immutable decision record, that captures an architectural choice, its context, and consequences.
_Avoid_: Design doc, spec document, meeting notes.

**Source Drift**:
A state discrepancy, where the current workspace source tree hash no longer matches the source tree hash bound in the evidence record.
_Avoid_: Code drift, stale build, dirty working tree.

**Harness Adapter**:
An autonomous vertical feature slice, that translates platform-specific agent events, tool calls, and manifests into canonical gauntlet operations.
_Avoid_: Plugin bridge, wrapper script, foreign hook.

**Capability Request**:
A strongly-typed operation descriptor, that represents an agent action before policy evaluation.
_Avoid_: Raw tool payload, json argument, command invocation.

**Trusted Enforcement Context**:
An immutable runtime descriptor, that defines workspace boundaries, active task authority, and security policies independently of caller input.
_Avoid_: Hook arguments, ambient context, caller state.

**Plugin Manifest**:
A declarative metadata ledger, that defines an adapter plugin's exposed skills, hooks, and verified entrypoints.
_Avoid_: Config file, package descriptor, json header.

**Open Knowledge Format (OKF)**:
A vendor-neutral knowledge specification, that defines Markdown document metadata, provenance, trust tiers, and lifecycle signals via YAML frontmatter.
_Avoid_: Custom header, config comment, doc schema.

**Trust Tier**:
A derived credibility classification, that categorizes a document's verification state (`unverified`, `machine-confirmed`, `human-reviewed`) based on its `verified` metadata.
_Avoid_: Approval score, rating, trust flag.

**Attested Computation**:
A standalone execution contract, that binds an explicit runtime and deterministic attester to computational output receipts.
_Avoid_: Verification script, runner config, calculation function.

**Release Readiness Gate**:
A pre-flight verification contract, that evaluates version harmony across project manifests, changelog synchronization, and architecture decision record coverage prior to software publication.
_Avoid_: Release check, version linter, publication script.

**Composite Coding Standards**:
A unified guidelines document, that combines language-specific conventions and transversal engineering invariants across multiple programming stacks within a polyglot workspace.
_Avoid_: Merged docs, mixed styleguide, combined rules.

**Workspace Diagnostics**:
A fast inspection contract, that evaluates local host environment prerequisites, toolchain availability, Git repository state, and declarative governance health without side effects.
_Avoid_: System audit, health check, env sniffer.

**Phase Checkpoint**:
A verified local Git commit, that binds the current workspace state to a specific TDD lifecycle phase (`SPEC`, `RED`, `GREEN`, `REFACTOR`, `GAUNTLET`) after confirming its phase-specific verification invariant.
_Avoid_: Quick save, commit hook, progress tag.

**Dynamic Response HUD**:
A real-time telemetry card, that renders active task progression, Git dirty state, and execution scope boundaries at the top of an agent's interactive response.
_Avoid_: Status badge, markdown header, info banner.

**Task Package Lifecycle**:
A deterministic sequence of operational states (`ACTIVE`, `BLOCKED`, `DONE`), that governs task progression from initial intent scoping to evidence sealing.
_Avoid_: Ticket workflow, task pipeline, sprint state.

