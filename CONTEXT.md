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
_Avoid_: Ticket, user-story, workitem, backlog-ticket.

**Layer**:
A verification step, that executes a specific analysis or testing command within a bounded timeout.
_Avoid_: Stage, build-step, check-item.

**Gauntlet**:
A sequential verification pipeline, that executes verification layers with fail-closed semantics and halts on the first mandatory failure.
_Avoid_: Test runner, CI script, test-suite-wrapper.

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
_Avoid_: Runtime preset, language profile, toolchain bundle.

**Canonical Workspace Manifest**:
A deterministic SHA-256 digest, that captures normalized Git-tree/blob OID hashes (immune to CRLF/LF conversions) and raw file contents across in-scope workspace paths.
_Avoid_: Git commit, workspace hash, checksum.

**Verification Report**:
An unsigned data record, that binds verification layer outcomes, diagnostic findings, and task contracts to the workspace manifest digests.
_Avoid_: Proof report, receipt, certification.

**Attestation Bundle (AttestationStatus, AttestationReport)**:
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

**Codebase Topology Graph**:
A directed graph model, that maps workspace packages, modules, files, and symbols along with their deterministic import and call dependencies.
_Avoid_: Dependency tree, code map, syntax graph.

**Blast Radius Analysis (BlastRadiusReport)**:
A deterministic traversal query, that calculates the transitive downstream components affected by modifications to a specific module or symbol within bounded traversal depth.
_Avoid_: Impact assessment, regression check, ripple effect.

**Structured Command Tokenization**:
An argument parsing model, that lexes shell command invocations into quote-delimited vectors and evaluates them against policy rules without substring matching.
_Avoid_: String matching, regex parsing, naive filter.

**Task Scope Enforcement**:
A fail-closed authorization gate, that restricts file modifications strictly to filepaths declared within the active task's scope array.
_Avoid_: Path filter, file restriction, folder guard.

**Test Scope Isolation**:
A boundary permission rule, that restricts modification authority exclusively to verified test suites when production task scope is undeclared.
_Avoid_: Test only mode, dev flag, sandbox test.

**Attestation Verification**:
A detached validation contract, that verifies Sigstore OIDC cryptographic bundles and subject digests against trusted GitHub repository identity and release provenance before binary execution.
_Avoid_: Checksum verification, hash match, signature check.

**Transactional Release Staging**:
A non-destructive workspace backup pattern, that stages modified assets in an isolated temporary directory during release bumps and rolls back on failure without invoking destructive Git operations.
_Avoid_: Git rollback, hard reset, cleanup script.

**Bootstrapper Integrity Verification**:
A client-side verification gate, that authenticates pre-compiled distribution archives against Sigstore attestation ledgers and static repository identifiers prior to unarchiving into the local cache.
_Avoid_: Tarball check, download validator, integrity test.

**Session Health**:
A quantitative telemetry model, that measures interaction turn counts, tool payload bytes, and estimated token saturation to detect cognitive fatigue and recommend clean session handoffs before attention dilution occurs.
_Avoid_: Session status, token limit, chat age, turn guard.

**Attention Dilution**:
A degenerative state of context degradation, where excessive conversational turns and cumulative tool payloads reduce model reasoning fidelity and increase hallucination risk.
_Avoid_: Context rot, memory drift, prompt bloat.

**Session Lineage Trail**:
An append-only cryptographic audit ledger, that records sequential engineering role contributions, phase checkpoints, and Git commit OIDs via tamper-evident SHA-256 hash-chaining across isolated chat sessions.
_Avoid_: Audit log, commit history, session list, chat transcript.

**Session Nonce**:
A deterministic or pseudo-random unique run identifier, that disambiguates independent AI harness sessions contributing to the same task.
_Avoid_: Session ID, conversation key, run token.

**Test Provenance**:
An epistemic verification record, that captures the empirical negative capability and failure evidence of an acceptance test prior to production code implementation.
_Avoid_: Test history, test log, test trace.

**Born-Green Test**:
A compromised test case, that is introduced concurrently with or subsequent to production code without prior empirical verification of its negative failure capability.
_Avoid_: Tautological test, premature test, unverified test, silent green.

**Inverse Preflight**:
A non-destructive temporal verification mechanism, that executes suspected born-green tests against pre-implementation baseline source trees to empirically confirm failure without data loss.
_Avoid_: Rollback test, baseline test, diff test, time travel test.

**Fast-Track Bugfix Lane**:
An accelerated two-phase verification workflow (`RED → DONE`) for bounded defect resolutions touching at most three source files without architectural modifications or ADR changes.
_Avoid_: Shortcut fix, unverified patch, bypass lane, quick fix.

**Worktree Inverse Proof**:
A mechanical single-step inverse verification protocol, executed in an isolated temporary Git worktree against unchanged HEAD to prove that a regression test fails before accepting a concurrent production bugfix.
_Avoid_: In-place rollback, stash test, manual checkout, dirty test.

**Domain**:
An overarching sphere of business knowledge and problem space, that defines the operational mission and subject area addressed by a software system.
_Avoid_: Technical layer, utility folder, dump directory, generic category.

**Subdomain**:
A cohesive functional subject area (`features/<subdomain>/`) within a Domain, that groups related business capabilities under shared domain concepts.
_Avoid_: Horizontal tier, helper directory, catch-all package.

**Bounded Context**:
An explicit linguistic and architectural system boundary in the solution space, within which every domain term in `CONTEXT.md` carries a single unambiguous meaning without implicit translation.
_Avoid_: System scope, global namespace, monolith boundary, module group.

**Shared Kernel**:
A minimal, framework-agnostic subset of domain models and primitives (`core/`), that multiple Vertical Slices depend on without coupling directly to each other.
_Avoid_: God module, global state store, UI model dump.

**Universal Layer Model**:
A four-tier top-down architectural dependency hierarchy (`app -> features -> core -> shared`), that enforces strict one-way downward imports and prohibits framework leakage into the Shared Kernel (`core`).
_Avoid_: Bidirectional modules, circular package graph, spaghetti imports.

**Vertical Slice**:
An autonomous, black-box encapsulated functional capability (`features/<slice>/` or `features/<subdomain>/<slice>/`), that owns its internal domain models and exposes a narrow public facade with explicit input state and output actions (`Stateful UI-Slice` or `Stateless Pipeline-Slice`) without importing upward or across peer slices.
_Avoid_: Horizontal layer, prop-drilled view helper, god-component fragment, leaky submodule.

**Living Slice Specification**:
A persistent capability contract (`specs/<slice>.md` or `specs/<subdomain>/<slice>.md`), that maintains the authoritative current invariants and behavior of a single Vertical Slice across multiple incremental task deltas (`spec_ref`).
_Avoid_: Ephemeral task note, frozen history doc, feature ADR.

**ADR Lifecycle**:
A four-state governance state machine (`proposed | accepted | superseded | deprecated`), that tracks the validity of cross-cutting architectural decisions and requires explicit `superseded_by` lineage when retiring obsolete records.
_Avoid_: Zombie ADR, permanent draft, feature changelog.

**Architecture Gate (ArchitectureConfig / ArchitectureReport / ArchitectureFinding / ArchitectureExemption)**:
A declarative structural verification engine, that enforces file line ceilings, monotonic ratchets, design token boundaries, downward layer dependencies, and black-box Vertical Slice isolation.
_Avoid_: Style linter, regex scanner, manual code audit.

**Checkpoint (CheckpointPhase / CheckpointOptions / CheckpointResult / CheckpointConfig / CheckpointError)**:
A phase-bound local Git commit transaction, that verifies TDD phase preflight invariants before recording an atomic state transition.
_Avoid_: Autosave, unverified commit, manual tag.

**Session Handoff**:
A structured context-transfer artifact, that summarizes completed task evidence, Git state, and the next engineering role for a clean chat session.
_Avoid_: Chat summary, handoff note, session dump.

**Doctor (DoctorCategory / DoctorCheckStatus / DoctorVerdict / DoctorCheckItem / DoctorOptions / DoctorReport / DoctorError)**:
A side-effect-free diagnostic subsystem, that inspects host platform prerequisites, Git repository health, governance files, and stack toolchains.
_Avoid_: Env sniffer, health daemon, system probe.

**Config (GauntletConfig / PathsConfig / ConfigValidationIssue / ConfigValidationReport / ConfigError / ValidationSeverity / ValidationIssue)**:
A declarative repository configuration model (`gauntlet.toml`), that specifies stack profiles, document paths, architectural boundaries, and verification layers.
_Avoid_: Dotfile settings, ad-hoc preferences, global registry.

**Adapter (NormalizedToolCall / AdapterValidationResult / AdapterHookVerdict)**:
A platform-normalizing harness translation contract, that converts vendor-specific hook payloads into canonical capability requests and verdicts.
_Avoid_: Hook wrapper, raw JSON bridge, vendor shim.

**Policy Decision (ToolActionType / DecisionVerdict / EnforcementContext)**:
A deterministic authorization outcome (`Allow`, `Deny`, `Ask`, `ForceAsk`), that binds a reason code and explanation to an evaluated capability request.
_Avoid_: Permission boolean, access flag, filter result.

**Evidence Metadata (VerificationVerdict / CheckStatus / CheckSummary / ExecutionOrigin / VcsMetadata / WorkspaceState / ExecutionMetadata / TaskContractSummary / TrustDecision)**:
A structured provenance record within a Verification Report, that captures layer check summaries, VCS state, execution origin, and multi-digest workspace hashes.
_Avoid_: Build log, test counter, run metadata.

**Migration (MigrationOptions / MigrationItem / MigrationReport / MigrationError)**:
A non-destructive schema modernization engine, that upgrades legacy in-repo governance files and hook configurations while preserving user-defined settings.
_Avoid_: Overwrite script, destructive upgrade, config reset.

**Plugin Distribution (PlatformTarget / DiscoveredHarness / PluginInstallOptions / PluginInstallStatus / PluginTargetInstallReport / PluginInstallReport / PluginError)**:
A cross-platform skill and hook installer, that deploys the sovereign agent skill suite into detected local AI harnesses.
_Avoid_: Package manager, extension downloader, manual copy.

**Release Engine (ReleaseFindingCategory / ReleaseFinding / ReleaseReadinessOptions / ReleaseReadinessReport / ReleaseError / SemverBump)**:
A transactional release verification and version-bumping subsystem, that enforces manifest parity, changelog rotation, and accepted ADR coverage.
_Avoid_: Publish script, tag pusher, version bumper.

**Scaffold (ScaffoldAction / ScaffoldFileReport / ScaffoldOptions / ScaffoldResult / ScaffoldError / ScaffoldTaskOptions / TaskScaffoldResult)**:
A safe, non-destructive template generator, that initializes in-repo governance files and task packages without overwriting existing user assets.
_Avoid_: Project generator, boilerplate cloner, force-init.

**Spec & Glossary Readiness (SpecReadinessReport / AdrReadinessReport / GlossaryDriftOptions / GlossaryDriftReport / FindingType)**:
A shift-left and post-implementation validation report, that verifies OKF v0.2 task contracts, ADR metadata, and two-way alignment between source code models and `CONTEXT.md`.
_Avoid_: Markdown linter, spellcheck report, doc validator.

**Topology Model (NodeType / EdgeType / Direction / TopologyNode / TopologyEdge / TopologyGraph / TopologyOptions / TopologyError)**:
An AST-derived structural graph representation, that models packages, modules, structs, enums, and their directed dependency edges.
_Avoid_: Call tree, file list, import dump.






