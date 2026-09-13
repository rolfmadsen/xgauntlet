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
  - Standard Markdown-skills (`grill-me`, `grill-with-docs`, `old-coder`, `diagnose`, `code-review`) og plugin-manifestet bages direkte ind i Rust-binæren via `include_str!`. Derved kan xGauntlet udrulles out-of-the-box uden afhængighed af eksterne netværkskald under installation.
- **Cross-Platform Harness Discovery Engine**:
  - Etablere deterministisk detektion i Rust for installerede agent-harnesses på tværs af **Linux (x64/arm64)**, **macOS (Intel og Apple Silicon M1–M4)** samt **Windows 10/11 (x64/arm64)**:
    - *Google Antigravity IDE*: `~/.gemini/config/plugins/xgauntlet/`
    - *Claude Code*: `~/.claude/skills/xgauntlet/` eller global settings
    - *OpenAI Codex*: `~/.codex/plugins/xgauntlet/`
    - *Mistral Vibe*: `~/.vibe/plugins/xgauntlet/`
  - Understøtte flagene: `--global`, `--harness <name>`, `--dry-run`, `--force`, `--target <dir>` og `--json`.
- **Slutbruger-redigerbarhed (Anti-Lock-in)**:
  - Skills udpakkes som rigtige Markdown-filer i brugerens globale katalog, så udvikleren har 100% frihed til at tilpasse formuleringer, regler og prompts.

### 2. JIT 7-Trins Pipeline Governance (Micro-Nudges)
I stedet for at dumpe tunge manualer på 300 linjer ind i konteksten eller begrænse styringen til den interne TDD-mikroløkke, injicerer telemetrimotoren lynhurtige **fase-direktiver (7–9 linjer, <100 tokens)** via harness-hooks (`PreInvocation` for Antigravity, `additionalContext` for Claude/Codex/Mistral) direkte knyttet til de **7 trin og 4 AI-roller** fra [README.md](README.md):
- **Trin 1: Idé- og kontekstafklaring** (`status: DRAFT` eller ingen aktiv opgave):
  * *Rolle 1*: `Senior Software Engineer (System Architecture & Requirements)`.
  * *Instruktion*: Udfordr antagelser med brugeren, opdater `CONTEXT.md` og `docs/adr/`. Må IKKE røre kildetræ.
  * *Anbefalet skill*: `grill-me` eller `grill-with-docs`.
- **Trin 2: Specifikation & Opgavebinding**:
  * *Rolle 1*: `Senior Software Engineer (System Architecture & Requirements)`.
  * *Instruktion*: Formaliser i `tasks/<id>.md` med OKF v0.2 frontmatter, acceptkriterier og Must NOT.
  * *Hård gate*: Kør `xgauntlet check-spec -t <id>`.
- **Trin 3: TDD & Implementering (Værkstedet)** (`status: ACTIVE`):
  * *Rolle 2*: `Senior Software Engineer (Feature Implementation & Testing)`.
  * *Instruktion*: Følg TDD-cyklussen: RED ➔ GREEN ➔ REFACTOR.
  * *WASM Gatekeeper*: Forbyder `git push`, `git reset --hard` (ADR 0003: kode 4039) og blokerer ændringer i kildetræ uden aktiv opgave.
  * *Anbefalet skill*: `diagnose`.
- **Trin 4: Flerlags Verifikation**:
  * *Rolle 2*: `Senior Software Engineer (Feature Implementation & Testing)`.
  * *Instruktion*: Kør `xgauntlet verify --task <id> --save`. Pre/post manifestkontrol, anti-tamper og forsegl `verification-report.json`.
- **Trin 5: Review mod Kodestandarder**:
  * *Rolle 3*: `Senior Software Engineer (Independent Code Review & Audit)`.
  * *Instruktion*: To-akset audit: Akse A mod `CODING_STANDARDS.md` (Fowler smells) og Akse B mod `spec.md`/`tasks/`. Kræver eksplicit udviklergodkendelse.
  * *Anbefalet skill*: `code-review`.
- **Trin 6: Drift-kontrol (Two-Tier Model)**:
  * *Rolle 3*: `Senior Software Engineer (Independent Code Review & Audit)`.
  * *Hård gate*: Kør `xgauntlet check-evidence`. Verificer Git blob OID invariant mod forseglede rapporter. Ved grøn status markeres opgaven `DONE`.
- **Trin 7: Release Readiness**:
  * *Rolle 4*: `Release & Operations Engineer (Release Attestation & Deployment)`.
  * *Hård gate*: Kør `xgauntlet check-release` (versionsharmoni, `CHANGELOG.md`, samtlige ADR-krydsreferencer). Vis `🏁 SESSION HANDOFF`-kort.

### 3. Diagnostic & Doctor Integration
- Udvide `xgauntlet doctor` med en dedikeret `Harnesses`-kategori, der rapporterer tilstedeværelse af installerede agent-harnesses samt installationsstatus for xGauntlet-pluginet.

## 📋 Acceptance Criteria
- [ ] Mappen `.agents/plugins/agent-gauntlet/` omdøbes til `.agents/plugins/xgauntlet/` med opdateret `plugin.json` (`name: "xgauntlet"`).
- [ ] `cargo run -p xgauntlet-cli -- validate-plugin --plugin-dir .agents/plugins/xgauntlet` validerer med 0 fejl.
- [ ] De 5 standard Markdown-skills (`grill-me`, `grill-with-docs`, `old-coder`, `diagnose`, `code-review`) og manifestet indlejres i Rust-kernen (`include_str!`).
- [ ] Cross-platform harness discovery engine implementeres i Rust med understøttelse af Linux, macOS (Intel og Apple Silicon M1–M4) og Windows (x64/arm64).
- [ ] CLI subcommand `xgauntlet plugin install` implementeres med understøttelse af flagene `--global`, `--harness <name>`, `--dry-run`, `--force`, `--target <dir>` og `--json`.
- [ ] Global plugin-installation opretter en gyldig plugin- og skill-struktur i de detekterede harness-kataloger uden at overskrive brugerdata uden `--force`.
- [ ] Telemetrimotoren (`features/telemetry/`) genererer JIT prompt-direktiver for samtlige 7 trin i README.md og angiver den aktive af de 4 AI-roller.
- [ ] Harness-adapterne (`antigravity`, `claude_code`, `codex`, `mistral`) udstiller de genererede 7-trins JIT-direktiver i deres respektive hook-payloads (`PreInvocation` og `PostToolUse`).
- [ ] `xgauntlet doctor` rapporterer fundne harnesses og status for globale xGauntlet-plugins under kategorien `Harnesses`.
- [ ] Conformance-tests i `crates/xgauntlet-core/tests/` dækker cross-platform harness-opdagelse, plugin-installation og 7-trins JIT prompt-generering med 100% grøn status.
- [ ] `cargo run -p xgauntlet-cli -- check-spec -t 023-pipeline-jit-governance-and-global-plugin-distribution` validerer med 0 fejl.
- [ ] Fuld workspace testsuite passerer (`cargo test --workspace`) uden linter-advarsler (`cargo clippy --workspace --all-targets -- -D warnings`).

## 🚫 Must NOT
- Må IKKE overskrive eksisterende brugerkonfigurationer eller tilpassede skills i `~/.gemini/config/` eller andre harness-kataloger uden eksplicit `--force`.
- Må IKKE tvinge unødvendige statiske skill-filer ind i forretningsprojekters kildetræ under `xgauntlet init`.
- Må IKKE overstige 12 linjer eller ~120 tokens pr. JIT-injektion for at forhindre context bloat og token-spild.
- Må IKKE forringe sub-3ms koldstarts-invarianten for hook- og telemetriafvikling.
- Må IKKE foretage remote publication handlinger (`git push`) eller destruktive filoperationer.
- Må IKKE introducere baggrunds-dæmoner jf. Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-13: Task 023 oprettet som afløser for Task 020 (deprecated). Omfanget er udvidet til at forankre JIT-promptstyringen direkte i README.md's 7-trins pipeline og de 4 AI-roller, samt deterministisk cross-platform harness discovery.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 023-pipeline-jit-governance-and-global-plugin-distribution`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- validate-plugin --plugin-dir .agents/plugins/xgauntlet`
- `cargo run -p xgauntlet-cli -- doctor`
