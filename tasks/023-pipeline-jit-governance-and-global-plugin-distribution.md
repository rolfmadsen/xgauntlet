---
type: Task Package
title: "Task 023: 7-Step Pipeline JIT Governance & Global Plugin Distribution"
description: "Etablere cross-platform global plugin-distribution for xGauntlet-skills samt Just-In-Time prompt-styring forankret i README.md's 7-trins pipeline og 4 AI-roller"
status: active
generated: { by: human:maintainer, at: "2026-09-13T16:35:00Z" }
tags: [plugin, distribution, jit-skills, telemetry, 7-steps, ai-roles, antigravity, claude-code, codex, mistral, cross-platform, adr-0003, adr-0004, adr-0006, adr-0007]
---

# Task 023: 7-Step Pipeline JIT Governance & Global Plugin Distribution

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-13`
**Erstatter**: [Task 020](020-global-plugin-distribution-and-jit-skill-injection.md) (Deprecated)

## 🎯 Formål & Løsningsdesign
Etablere en ren, decoupled og cross-platform distribution af xGauntlets agent-skills og metoderegler, så de er tilgængelige globalt på udviklerens maskine uden at forurene forretningsprojekters kildetræ, kombineret med kirurgisk Just-In-Time (JIT) prompt-injektion forankret i systemets **7-trins pipeline og 4 AI-roller** jf. [README.md](README.md), [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0003](docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md), [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md) og [ADR 0007](docs/adr/0007-local-transparent-supervisor-and-wasm-verifier.md):

### 1. Global Plugin Distribution (`xgauntlet plugin install`)
- **Omdøbning og registrering**:
  - Flytte og omdøbe `.agents/plugins/agent-gauntlet/` til `.agents/plugins/xgauntlet/`.
  - Opdatere `plugin.json` (`"name": "xgauntlet"`), `hooks.json` og tilhørende skill-metadata.
- **Batteries-Included Indlejring i Rust-kernen**:
  - Den samlede suite af 11 standard Markdown-skills og plugin-manifestet bages direkte ind i Rust-binæren via `include_str!`:
    - *Sokratisk Afklaring & Domænemodellering*: `grill-me`, `grill-with-docs`, `domain-modeling`
    - *Specifikation & Opgavedekomponering*: `to-spec` (for [spec.md](spec.md)), `to-tasks` (for `tasks/<id>.md` med tracer-bullets og DAG)
    - *Håndværk & Fejlisolation*: `old-coder`, `diagnose`
    - *Arkitektur & Deep Modules*: `codebase-design`, `improve-codebase-architecture` (med interaktiv visuel HTML-rapport)
    - *Audit & Kodeanmeldelse*: `code-review`
    - *Session Retrospektiv*: `retro` (forbedring af miljø, lintere og `AGENTS.md`)
  - Derved kan xGauntlet udrulles out-of-the-box uden afhængighed af eksterne netværkskald under installation.
- **Cross-Platform Harness Discovery Engine**:
  - Etablere deterministisk detektion i Rust for installerede agent-harnesses på tværs af **Linux (x64/arm64)**, **macOS (Intel og Apple Silicon M1–M4)** samt **Windows 10/11 (x64/arm64)**:
    - *Google Antigravity IDE*: `~/.gemini/config/plugins/xgauntlet/`
    - *Claude Code*: `~/.claude/skills/xgauntlet/` eller global settings
    - *OpenAI Codex*: `~/.codex/plugins/xgauntlet/`
    - *Mistral Vibe*: `~/.vibe/plugins/xgauntlet/`
  - Understøtte flagene: `--global`, `--harness <name>`, `--dry-run`, `--force`, `--target <dir>` og `--json`.
- **Slutbruger-redigerbarhed (Anti-Lock-in)**:
  - Skills udpakkes som rigtige Markdown-filer i brugerens globale katalog, så udvikleren har 100% frihed til at tilpasse formuleringer, regler og prompts.

### 2. Fasedrevet JIT Governance & 7 Specialiserede AI-Roller (Matt Pocock Design)
I stedet for tunge statiske manualer eller numre-baserede trin (der jf. Matt Pococks *writing-for-agents* skaber "the pull of post-completion steps" og forhastet arbejde), injicerer telemetrimotoren lynhurtige **fasedirektiver (<45 tokens)** via harness-hooks (`PreInvocation` for Antigravity, `additionalContext` for Claude/Codex/Mistral). 

Hvert direktiv anvender:
1. **Eksplicit Aktiv Rolle**: `Active Role: <Navn (Funktionelt anker)>` tvinger modellen ind i en skarp persona.
2. **Semantisk Fase**: Angiver den aktuelle tilstand uden sekvensnumre for at forankre modellen 100% i nuet.
3. **Positiv Målsætning (`Target`)**: 0 negationer. WASM-kernen håndhæver forbud (kode 4039); prompten leder mod målet.
4. **Binært Afslutningskriterium (`Gate (Done)`)**: Tjekbar grænse, der modvirker *premature completion*.
5. **Front-Loaded Context Pointer (`Pointer`)**: Præcis trigger til on-demand skills.

#### De 7 JIT-Fasedirektiver & Specialiserede Roller:
1. **Phase: Ideation & Context**
   * *Aktiv Rolle*: `Active Role: System Architect (Scope & Invariants)`
   * *Target*: Establish operational boundaries and ubiquitous terminology in CONTEXT.md.
   * *Gate (Done)*: Human approves scope and domain glossary.
   * *Pointer*: Socratic questioning: invoke grill-with-docs or domain-modeling to formalize CONTEXT.md and ADRs.
2. **Phase: Specification & Task Binding**
   * *Aktiv Rolle*: `Active Role: Requirements Engineer (Contracts & Criteria)`
   * *Target*: Formalize acceptance criteria (- [ ]) and Must NOT invariants in tasks/<id>.md.
   * *Gate (Done)*: Command 'xgauntlet check-spec -t <id>' exits with 0 errors.
   * *Pointer*: Specification synthesis: invoke to-spec to formalize spec.md and to-tasks to generate verified tasks/<id>.md.
3. **Phase: Implementation (TDD)**
   * *Aktiv Rolle*: `Active Role: TDD Craftsman (Red-Green-Refactor)`
   * *Target*: Execute tight TDD cycle: failing test (RED) ➔ minimal fix (GREEN) ➔ refactor.
   * *Gate (Done)*: All acceptance assertions pass with atomic git phase-checkpoints.
   * *Pointer*: TDD loop: invoke old-coder for test-first development or diagnose for root-cause isolation.
4. **Phase: Multi-Layer Verification**
   * *Aktiv Rolle*: `Active Role: Verification & QA Engineer (Gauntlet & Anti-Tamper)`
   * *Target*: Execute verification gauntlet and seal anti-tamper evidence.
   * *Gate (Done)*: Command 'xgauntlet verify --task <id> --save' records verdict PASSED.
   * *Pointer*: Architecture refactor: invoke codebase-design or improve-codebase-architecture for deep modules and inspect verification-report.json.
5. **Phase: Standards & Spec Audit**
   * *Aktiv Rolle*: `Active Role: Independent Code Reviewer (Standards & Smells)`
   * *Target*: Two-axis review: Axis A (CODING_STANDARDS.md smells), Axis B (Task acceptance criteria).
   * *Gate (Done)*: Explicit human approval of audit findings before task closure.
   * *Pointer*: Code review: invoke code-review to audit git diff across both axes.
6. **Phase: Evidence Integrity & Drift Check**
   * *Aktiv Rolle*: `Active Role: Evidence & Integrity Auditor (Drift & Trust Boundary)`
   * *Target*: Verify workspace Git-tree/blob OID integrity and commit final sealed task checkpoint.
   * *Gate (Done)*: Command 'xgauntlet check-evidence' reports 0 drift findings, task marked DONE, and local checkpoint committed.
   * *Pointer*: Checkpoint gate: invoke 'xgauntlet checkpoint --phase done' to seal evidence and execute local commit.
7. **Phase: Release Readiness**
   * *Aktiv Rolle*: `Active Role: Release & Operations Engineer (Release & Attestation)`
   * *Target*: Verify 100% clean git worktree, synchronize release readiness, and display session handoff prompt.
   * *Gate (Done)*: Git worktree confirmed clean (ready for user 'git push') and 🏁 SESSION HANDOFF card displayed.
   * *Pointer*: Session retrospective: invoke retro for environment improvements and inspect 'git status' for clean worktree.

### 3. Diagnostic & Doctor Integration
- Udvide `xgauntlet doctor` med en dedikeret `Harnesses`-kategori, der rapporterer tilstedeværelse af installerede agent-harnesses samt installationsstatus for xGauntlet-pluginet.

### 4. Sanering af In-Repo Governance & ADR Decoupling
- **Decouple Platform Invariants fra Lokale Forretnings-ADRs**:
  - xGauntlets indbyggede regler (Surgical Gatekeeper / No Remote Push jf. kode 4039, samt Two-Tier Evidence Model) er fysisk bagt ind i Rust/WASM-kernen og håndhæves af værktøjet uanset projektets filer. De skal forankres som **Platform Invariants** i `spec.md` og `CODING_STANDARDS.md`, ikke som lokale ADR-dokumenter i forretningsprojekter.
  - Template-motoren i `features/scaffold/templates.rs` opdateres, så `xgauntlet init` ikke længere optager `docs/adr/0001-package-by-feature-architecture.md` som en tvungen fil, men i stedet opretter en neutral ADR-skabelon (`docs/adr/template.md`) og overlader `docs/adr/` 100% til projektets egne forretningsbeslutninger.
  - *Package-by-Feature (Screaming Architecture)* bevares som universel transversel kvalitetsstandard i `CODING_STANDARDS.md`.
  - Prompts, `.agents/AGENTS.md`-skabeloner og JIT micro-nudges saneres, så henvisninger til platformregler ikke linker til fiktive/lokale stier (`docs/adr/0003-...`), hvilket eliminerer 404-links og navnerumskollisioner i klientprojekter (som observeret i `knowledgegraphstudio`).

## 📋 Acceptance Criteria
- [x] Mappen `.agents/plugins/agent-gauntlet/` omdøbes til `.agents/plugins/xgauntlet/` med opdateret `plugin.json` (`name: "xgauntlet"`).
- [x] `cargo run -p xgauntlet-cli -- validate-plugin --plugin-dir .agents/plugins/xgauntlet` validerer med 0 fejl.
- [x] Den samlede suite af 11 Markdown-skills (`grill-me`, `grill-with-docs`, `domain-modeling`, `to-spec`, `to-tasks`, `old-coder`, `diagnose`, `codebase-design`, `improve-codebase-architecture`, `code-review`, `retro`) og manifestet indlejres i Rust-kernen (`include_str!`).
- [x] Cross-platform harness discovery engine implementeres i Rust med understøttelse af Linux, macOS (Intel og Apple Silicon M1–M4) og Windows (x64/arm64).
- [x] CLI subcommand `xgauntlet plugin install` implementeres med understøttelse af flagene `--global`, `--harness <name>`, `--dry-run`, `--force`, `--target <dir>` og `--json`.
- [x] Global plugin-installation opretter en gyldig plugin- og skill-struktur i de detekterede harness-kataloger uden at overskrive brugerdata uden `--force`.
- [x] Telemetrimotoren (`features/telemetry/`) genererer JIT fasedirektiver for samtlige 7 faser med aktive specialiserede AI-roller, positive targets, checkable gate-bounds, front-loaded pointers (<45 tokens pr. direktiv) samt Clean Worktree Guarantee ved session handoff.
- [x] Harness-adapterne (`antigravity`, `claude_code`, `codex`, `mistral`) udstiller de genererede 7 fasedirektiver i deres respektive hook-payloads (`PreInvocation` og `PostToolUse`).
- [x] `xgauntlet doctor` rapporterer fundne harnesses og status for globale xGauntlet-plugins under kategorien `Harnesses`.
- [x] `features/scaffold/templates.rs` saneres, så `xgauntlet init` stilladserer `docs/adr/template.md` frem for at okkupere `docs/adr/0001-package-by-feature-architecture.md`.
- [x] Skabeloner for `.agents/AGENTS.md` og JIT-prompts saneres for brudte referencer til lokale `docs/adr/0003-...` filer og benytter i stedet eksplicitte platforminvarianter.
- [x] Conformance-tests i `crates/xgauntlet-core/tests/` dækker cross-platform harness-opdagelse, plugin-installation, 7-faset JIT prompt-generering og opdateret template-scaffolding med 100% grøn status.
- [x] `cargo run -p xgauntlet-cli -- check-spec -t 023-pipeline-jit-governance-and-global-plugin-distribution` validerer med 0 fejl.
- [x] Fuld workspace testsuite passerer (`cargo test --workspace`) uden linter-advarsler (`cargo clippy --workspace --all-targets -- -D warnings`).

## 🚫 Must NOT
- Må IKKE overskrive eksisterende brugerkonfigurationer eller tilpassede skills i `~/.gemini/config/` eller andre harness-kataloger uden eksplicit `--force`.
- Må IKKE tvinge unødvendige statiske skill-filer ind i forretningsprojekters kildetræ under `xgauntlet init`.
- Må IKKE overstige 8 linjer eller ~50 tokens pr. JIT-injektion for at forhindre context bloat og token-spild.
- Må IKKE indeholde negationer ("må ikke", "forbud") i JIT-prompts jf. Pococks negation-invariant.
- Må IKKE forringe sub-3ms koldstarts-invarianten for hook- og telemetriafvikling.
- Må IKKE forurene klientprojekters `docs/adr/` med xGauntlet-specifikke interne implementerings-ADRs.
- Må IKKE foretage remote publication handlinger (`git push`) eller destruktive filoperationer.
- Må IKKE introducere baggrunds-dæmoner jf. Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-13: Udvidet skill-suiten til 11 fuldt integrerede Markdown-skills (tilføjet domain-modeling, to-spec, to-tasks, codebase-design, improve-codebase-architecture og retro) og afstemt 1:1 som front-loaded pointers i de 7 JIT fasedirektiver.
- 2026-09-13: Etableret 'Clean Worktree Guarantee' i Phase 6 og 7: Phase 6 forsegler og committer lokalt via 'xgauntlet checkpoint --phase done', så Phase 7 garanterer et 100% rent git worktree ('Git: clean'), der tillader brugeren direkte at køre 'git push'.
- 2026-09-13: Refaktoreret med Matt Pococks *writing-for-agents* principper: JIT-prompts er nu fasedrevne uden sekvens-tal (modvirker premature completion), roller er specialiseret 1:1 pr. fase med funktionelle ankre (`Active Role:`), og prompts er 100% positive med tjekbare `Gate (Done)` bounds og under 45 tokens.
- 2026-09-13: Udvidet med sanering af in-repo governance og ADR decoupling: Platform-invarianter (Surgical Gatekeeper, Two-Tier evidens) adskilles fra klientprojekters lokale `docs/adr/`, og `xgauntlet init` stilladserer ren `docs/adr/template.md` i stedet for at okkupere `0001`.
- 2026-09-13: Task 023 oprettet som afløser for Task 020 (deprecated). Omfanget er udvidet til at forankre JIT-promptstyringen direkte i README.md's 7-trins pipeline og de 4 AI-roller, samt deterministisk cross-platform harness discovery.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 023-pipeline-jit-governance-and-global-plugin-distribution`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- validate-plugin --plugin-dir .agents/plugins/xgauntlet`
- `cargo run -p xgauntlet-cli -- doctor`
