//! Standard templates for in-repo governance, stack profiles, and agent harness configurations.

use crate::features::config::default_config_for_stack;

/// Pair of relative file path and rendered text content for a scaffolded governance file.
pub struct ScaffoldTemplate {
    pub relative_path: &'static str,
    pub content: String,
}

/// Generates all 9 canonical in-repo governance files for the target stack and project name.
pub fn generate_templates(stack: &str, project_name: &str) -> Vec<ScaffoldTemplate> {
    let now = "2026-09-06T18:00:00Z";
    let today = "2026-09-06";

    vec![
        ScaffoldTemplate {
            relative_path: "gauntlet.toml",
            content: render_gauntlet_toml(stack),
        },
        ScaffoldTemplate {
            relative_path: "CONTEXT.md",
            content: render_context_md(project_name, now),
        },
        ScaffoldTemplate {
            relative_path: "CODING_STANDARDS.md",
            content: render_coding_standards_md(project_name, stack),
        },
        ScaffoldTemplate {
            relative_path: "spec.md",
            content: render_spec_md(project_name, now),
        },
        ScaffoldTemplate {
            relative_path: "tasks/001-bootstrap.md",
            content: render_task_bootstrap_md(project_name, today, now),
        },
        ScaffoldTemplate {
            relative_path: "docs/adr/0001-package-by-feature-architecture.md",
            content: render_adr_0001_md(now),
        },
        ScaffoldTemplate {
            relative_path: ".agents/AGENTS.md",
            content: render_agents_md(project_name),
        },
        ScaffoldTemplate {
            relative_path: ".agents/hooks.json",
            content: render_hooks_json(),
        },
        ScaffoldTemplate {
            relative_path: "CLAUDE.md",
            content: render_claude_md(project_name),
        },
    ]
}

fn render_gauntlet_toml(stack: &str) -> String {
    let config = default_config_for_stack(stack);
    config.render_toml()
}

fn render_context_md(project_name: &str, timestamp: &str) -> String {
    format!(
        r#"---
type: Knowledge Bundle Index
title: "{project_name} Context & Domain Glossary"
description: "Kernebegreber, arkitekturgrænser og definitioner for {project_name}"
status: stable
generated: {{ by: process:xgauntlet-init, at: "{timestamp}" }}
tags: [glossary, domain-model, ubiquitous-language, okf]
---

# {project_name} Context & Domain Glossary

This document defines the core ubiquitous language for `{project_name}` using Aristotle's formula (*definitio per genus et differentiam*). It captures domain concepts without implementation noise.

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

**Canonical Workspace Manifest**:
A deterministic SHA-256 digest, that captures normalized Git-tree/blob OID hashes and raw file contents across in-scope workspace paths.
_Avoid_: Git commit, workspace hash, checksum.

**Verification Report**:
An unsigned data record, that binds verification layer outcomes, diagnostic findings, and task contracts to the workspace manifest digests.
_Avoid_: Proof report, receipt, certification.
"#
    )
}

fn render_coding_standards_md(project_name: &str, stack: &str) -> String {
    format!(
        r#"# Composite Coding Standards: {project_name}

Dette dokument fastlægger de tværgående kodestandarder og håndværksmæssige principper for `{project_name}` (primær stack: `{stack}`).

---

## 🏛️ Transversale Arkitektur- & Kvalitetsinvarianter

1. **Package-by-Feature (Screaming Architecture)**:
   - Al kode organiseres i autonome feature-moduler, der indkapsler forretningslogik, modeller og lokale tests.
   - Undgå flade kataloger med tekniske lag (`models/`, `views/`, `controllers/`).

2. **Test-Driven Development (Red-Green-Refactor)**:
   - Skriv altid fejlede accept- eller enhedstests først (RED).
   - Implementér den minimale kode, der gør testen grøn (GREEN).
   - Refaktorér med bevaret adfærd under fuld testdækning (REFACTOR).

3. **Nul Compiler- & Linter-Advarsler**:
   - Kodebasen skal til enhver tid kompilere og lintes med 0 advarsler under `-D warnings` / tilsvarende flags.

4. **Fejlhåndtering & Fail-Closed**:
   - Håndter alle fejl eksplicit via type-sikre resultater.
   - Slug aldrig exceptions eller fejl uden struktureret rapportering.
"#
    )
}

fn render_spec_md(project_name: &str, timestamp: &str) -> String {
    format!(
        r#"---
type: System Specification
title: Specification - {project_name} Architecture & Capabilities
description: Macro system architecture, philosophy, and invariants for {project_name}
status: active
generated: {{ by: process:xgauntlet-init, at: "{timestamp}" }}
tags: [specification, architecture, invariants]
---

# Specification: {project_name} Architecture & Capabilities

## 🎯 Philosophy & Core Capabilities
1. **Clean Architecture & Autonomous Features**:
   - Autonome feature-moduler med klare ansvarsområder og veldefinerede grænseflader.
2. **Deterministic Verification**:
   - Multi-layer verifikations-pipeline styret af deklarativ konfiguration (`gauntlet.toml`).

## 📐 Architecture & Feature Modules
- `src/features/`: Forretningsdomæner og autonome komponenter.
- `tasks/`: Eksekverbare opgavepakker med acceptkriterier.
- `docs/adr/`: Arkitektoniske beslutningsreferater.

## 🚫 Must NOT (System Invariants)
- Må IKKE introducere skjulte runtime-afhængigheder eller udokumenterede baggrundsprocesser.
- Må IKKE omgå deklarative verifikationslag eller ignorere fejlede tests.
- Må IKKE foretage utilsigtede remote publication kommandoer (`git push`).

## 🧪 Multi-Layer Verification Contracts
- [ ] 100% test pass rate på alle konfigurerede lag.
- [ ] 0 linter- og typechecker-advarsler.
"#
    )
}

fn render_task_bootstrap_md(project_name: &str, today: &str, timestamp: &str) -> String {
    format!(
        r#"---
type: Task Package
title: "Task 001: Project Setup & Baseline Verification Gauntlet"
description: "Initialisere projektstruktur, deklarativ gauntlet konfiguration og køre første grønne verifikationskørsel for {project_name}"
status: active
generated: {{ by: process:xgauntlet-init, at: "{timestamp}" }}
tags: [bootstrap, setup, gauntlet]
---

# Task 001: Project Setup & Baseline Verification Gauntlet

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `{today}`

## 🎯 Formål
Etablere projektets fundament for `{project_name}`, konfigurere `gauntlet.toml`, validere domæne-glossary og sikre at den første verifikationskørsel er 100% grøn.

## 📋 Acceptance Criteria
- [ ] `gauntlet.toml` er konfigureret med de korrekte verifikationslag for projektets stack.
- [ ] `CONTEXT.md` definerer projektets centrale forretnings- og domænebegreber jf. Aristoteles' formel.
- [ ] `spec.md` indeholder overordnede arkitekturprincipper og systeminvarianter.
- [ ] Første verifikationskørsel gennemføres med succes (`xgauntlet verify`).

## 🚫 Must NOT
- Må IKKE introducere udokumenterede afhængigheder eller baggrundsprocesser.
- Må IKKE tillade fejlede tests eller kompilatorfejl i verifikationskørslen.

## 📝 Revisions
- {today}: Oprettet via `xgauntlet init`.

## 🧪 Verifikation
- `xgauntlet check-spec -t 001-bootstrap`
- `xgauntlet check-config`
- `xgauntlet verify --task 001-bootstrap`
"#
    )
}

fn render_adr_0001_md(timestamp: &str) -> String {
    format!(
        r#"---
type: Architectural Decision Record
title: 'ADR 0001: Package-by-Feature Architecture'
status: stable
tags: [architecture, adr]
generated: {{ by: process:xgauntlet-init, at: "{timestamp}" }}
---

# 1. Package-by-Feature (Screaming Architecture)

**Status**: `accepted`  
**Date**: `2026-09-06`  

## Context
Tidligere var kildekoden ofte opdelt i tekniske lag (f.eks. controllere, modeller, services), hvilket spredte sammenhængende domænelogik og øgede utilsigtet kobling.

## Decision
Al domænelogik, forretningsregler og tilhørende tests organiseres som **Package-by-Feature** i selvstændige, modulære mapper. Hver feature indkapsler sine egne modeller, logik og enhedstests.

## Consequences
Nye funktioner tilføjes i isolerede feature-moduler. Dette gør kodebasen overskuelig for både mennesker og AI-agenter og minimerer utilsigtet regression.
"#
    )
}

fn render_agents_md(project_name: &str) -> String {
    format!(
        r#"# Agent Guidelines: {project_name}

Dette repository følger **Evidence-First Development & Clean Craftsmanship** metodikken.

---

## 📊 Standard Response HUD Protocol
Formatér altid toppen af samtlige synlige agent-svar med det transparente Cockpit Task HUD kort (maks. 5 linjer):
> ### 🛡️ [Task: <Task Title / ID>] `[<Task Type>: <Phase>]`
> **Status**: `Phase: <SPEC | RED | GREEN | REFACTOR | GAUNTLET | DONE>` | `Gauntlet: <PASS | FAIL | PENDING>` | `Git: <branch>@<oid> • <clean | dirty: N files>`
> **Progress**: `Criteria: X/Y [■■□□□]` | `Scope: <affected crates/paths>`
> **Links**: 📋 [Task](tasks/) • 📄 [Spec](spec.md) • 📖 [Glossary](CONTEXT.md) • 🏛️ [ADR](docs/adr/README.md) • 🧪 [Evidence](evidence.md)
> 💡 **Next Action:** <kort beskrivelse af næste umiddelbare handling>

---

## 💡 Intent-to-Task Sparringsprocedure (Idéfase)
Når en bruger henvender sig med et ustruktureret eller uformelt ønske, fungerer agenten som proaktiv sparringspartner gennem en 4-trins model før en formel opgavefil oprettes i `tasks/`:
1. **Formål & Afgrænsning**: Afdæk det reelle behov, kerneegenskaber og operationelle grænser (hvad skal løses, og hvad skal eksplicit udelades?).
2. **Invarianter & Must NOT**: Fastlæg negative begrænsninger og arkitektoniske barrierer, der under ingen omstændigheder må brydes (f.eks. Zero-Daemon, Zero Ambient Authority, ingen eksterne sockets eller utilsigtede afhængigheder).
3. **RED Test-hypotese**: Formuler en præcis hypotese om den observerbare fejl, regressionsrisiko eller manglende adfærd, som en fejlet accepttest skal påvise.
4. **ADR-triggere**: Vurder om ændringen introducerer irreversible trade-offs eller bryder eksisterende beslutninger i `docs/adr/`. Hvis en beslutning udfordres, skal en ny ADR formuleres.

---

## 🔄 Core Development Loop
```text
SPEC / GRILL → (Human Approval) → RED → GREEN → REFACTOR → GAUNTLET → EVIDENCE
```

1. **SPEC / GRILL**: Konkrete eksekverbare kriterier i `tasks/<task>.md` og `spec.md`, afstemt med `CONTEXT.md`.
2. **RED**: Skriv fejlede accepttests først, og bevis at de fejler med den forventede årsag.
3. **GREEN**: Minimal implementation for at få testene til at passere.
4. **REFACTOR**: Oprydning i kode og modularitet, mens assertionerne forbliver frosne.
5. **GAUNTLET**: Kør multi-layer verifikation via `xgauntlet verify`:
   - Linters & Static Analysis
   - Type Checks & Kompilering
   - Acceptance & Unit Tests
   - Invariant & Spec Tests (`xgauntlet check-spec`)
   - Mutation Testing Gauntlet
6. **EVIDENCE**: Forsegl verifikationsrapport og evidens i `verification-report.json` og `evidence.md`.
7. **SESSION HANDOFF**: Vis `🏁 SESSION HANDOFF` kortet med starter-prompt til næste session.

---

## 🔒 Lokal TDD Phase Checkpoint Protokol (ADR 0003)
For at sikre sporbarhed, atomiske tilbagerulningspunkter og beskytte mod context rot, skal agenten udføre lokale git commits (`git add` og `git commit`) ved hver fase-overgang i TDD-løkken jf. [ADR 0003](docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md):
- `SPEC`: `task(<id>): initialize task specification and criteria`
- `RED`: `test(<id>): add failing acceptance test for <feature> [RED]`
- `GREEN`: `feat(<id>): implement minimal logic to satisfy test [GREEN]`
- `REFACTOR`: `refactor(<id>): clean up module boundaries and types [REFACTOR]`
- `DONE`: `chore(<id>): seal evidence and mark task DONE`

**Kritiske Invarianter (ADR 0003):**
- Foretag ALDRIG remote publication handlinger (`git push`).
- Foretag ALDRIG destruktive reset handlinger (`git reset --hard` eller `git clean -f`).
- Alle commits forbliver strengt lokale checkpoints på udviklerens maskine.
"#
    )
}

fn render_hooks_json() -> String {
    r#"{
  "agent-gauntlet-gatekeeper": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "xgauntlet hook antigravity",
            "timeout": 30
          }
        ]
      }
    ],
    "PreInvocation": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "xgauntlet telemetry --format antigravity-hook"
          }
        ]
      }
    ]
  }
}
"#
    .to_string()
}

fn render_claude_md(project_name: &str) -> String {
    format!(
        r#"# Claude Code Guidelines: {project_name}

Dette repository anvender xGauntlet til verifikation, specifikationsvalidering og evidensforsegling.

## Sikkerhedsinvarianter
1. Foretag ALDRIG remote git push (`git push`).
2. Kør `xgauntlet verify` før opgaver afsluttes for at forsegle evidens.
3. Respekter `tasks/` acceptkriterier og `CONTEXT.md` domæne-glossary.
"#
    )
}
