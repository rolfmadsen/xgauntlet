# Agent Guidelines: agent-gauntlet

This repository follows the **Evidence-First Development & Clean Craftsmanship** methodology.

---

## 📊 Standard Response HUD Protocol
Formatér altid toppen af samtlige synlige agent-svar med det transparente Cockpit Task HUD kort (maks. 5 linjer):
> ### 🛡️ [Task: <Task Title / ID>] `[<Task Type>: <Phase>]`
> **Status**: `Phase: <SPEC | RED | GREEN | REFACTOR | GAUNTLET | DONE>` | `Gauntlet: <PASS | FAIL | PENDING>` | `Git: <branch>@<oid> • <clean | dirty: N files>`
> **Progress**: `Criteria: X/Y [■■□□□]` | `Scope: <affected crates/paths>`
> **Links**: 📋 [Task](tasks/) • 📄 [Spec](spec.md) • 📖 [Glossary](CONTEXT.md) • 🏛️ [ADR](docs/adr/) • 🧪 [Evidence](evidence.md)
> 💡 **Next Action:** <kort beskrivelse af næste umiddelbare handling>

---

## 🛠️ Bundled Agent Skills (`.agents/skills/`)
The agent has direct access to bundled skills located in [.agents/skills/](.agents/skills/) (and packaged under [plugins/agent-gauntlet/skills/](plugins/agent-gauntlet/skills/)). When a skill is invoked, the agent MUST view its `SKILL.md` before proceeding:

1. **[old-coder](.agents/skills/old-coder/SKILL.md)**:
   * *Purpose*: Evidence-first development methodology (SPEC $	o$ RED $	o$ GREEN $	o$ REFACTOR $	o$ GAUNTLET $	o$ EVIDENCE).
2. **[grill-me](.agents/skills/grill-me/SKILL.md)**:
   * *Purpose*: Socratic interview to stress-test designs and resolve the decision tree before writing code.
3. **[grill-with-docs](.agents/skills/grill-with-docs/SKILL.md)**:
   * *Purpose*: Challenges plans against domain concepts in [CONTEXT.md](CONTEXT.md) and creates/updates ADRs in [docs/adr/](docs/adr/).
4. **[diagnose](.agents/skills/diagnose/SKILL.md)**:
   * *Purpose*: Disciplined root-cause diagnosis loop (Reproduce $	o$ Minimize $	o$ Hypothesize $	o$ Instrument $	o$ Fix $	o$ Regression-test).
5. **[code-review](.agents/skills/code-review/SKILL.md)**:
   * *Purpose*: Two-axis review (Standards vs Spec) running parallel sub-agents with Fowler code smells baseline.

---

## 📄 Specification Governance (`spec.md`)
1. **Macro System Specification:** `spec.md` represents the repository's high-level executable specification, system-wide invariants, and capabilities (whereas `tasks/` tracks individual, isolated work packages).
2. **Standard `spec.md` Structure:**
   * `# Specification: <System / Feature Name>`
   * `## 🎯 Philosophy & Core Capabilities`: Overordnede systemegenskaber og domæneprincipper.
   * `## 📐 Architecture & Feature Modules`: Modul- og pakkestruktur (`Package-by-Feature`).
   * `## 🚫 Must NOT (System Invariants)`: Globale sikkerheds- og arkitektur-invarianter, der gælder på tværs af alle opgaver.
   * `## 🧪 Multi-Layer Verification Contracts`: Makro-verifikationskriterier og test-dækning.
3. **Hvornår `spec.md` udfyldes & opdateres:**
   * **`🚀 NEW FEATURE` & `🔄 ARCHITECTURAL REFACTOR`:** Før kodning påbegyndes, SKAL agenten sikre, at `spec.md` er opdateret og godkendt af brugeren i SPEC-fasen.
   * **`🐛 BUG FIX` & `🔍 QUERY`:** Udføres mod de eksisterende specifikationsinvarianter uden behov for omskrivning af `spec.md`.

---

## 🗂️ Task Management Protocol (`tasks/`)
1. **Curated Scope:** Every non-trivial work item is tracked as a concise markdown file in `tasks/<number>-<title>.md`.
2. **Standard Task Structure:**
   * `# Task <number>: <Title>` (Header with `Status: ACTIVE | DONE`, `Intent: 🚀 NEW FEATURE | 🐛 BUG FIX | 🔄 REFACTOR`)
   * `## 🎯 Formål`: Konkret målsætning og afgrænsning.
   * `## 📋 Acceptance Criteria`: Eksekverbare `- [ ]` punkter med klare forventede inputs og outputs.
   * `## 🚫 Must NOT`: Negative begrænsninger og arkitektur-invarianter, der under ingen omstændigheder må brydes.
   * `## 📝 Revisions`: Append-only ændringslog for mid-task ændringer og afviste forslag (hvad brugeren sagde nej til).
   * `## 🧪 Verifikation`: Konkrete kommandoer til afprøvning og validering.
3. **Clean Session Handoffs:** A new chat session starts by reading the designated `tasks/<task>.md` and `CONTEXT.md`.
4. **No Memory Rot:** Completed tasks are marked `DONE` and remain frozen; persistent domain knowledge is distilled into `CONTEXT.md` and `docs/adr/`.

---

## 🏛️ Architecture Decisions & ADR Governance (`docs/adr/`)
1. **Strict ADR Adherence:** The agent MUST strictly comply with all accepted Architecture Decision Records in `docs/adr/`.
2. **Active Sparring on Conflicts:** If a user prompt, new task, or proposed code contradicts existing ADRs or gauntlet invariants, the agent MUST immediately challenge the contradiction, surface the trade-off, and resolve the decision before proceeding.
3. **Lazy Creation:** New ADRs in `docs/adr/` are created only for irreversible, non-obvious trade-offs.

---

## 🎯 Intent Classification & Discovery
Before writing code, classify intent and align with domain terminology:
- 🔍 **QUERY / DIAGNOSIS:** Information request or root-cause discovery (read-only; use `diagnose`).
- 🚀 **NEW FEATURE / REFACTOR:** Run `grill-me` or `grill-with-docs` to resolve decisions and update `CONTEXT.md` before coding.
- 🐛 **BUG FIX:** Reproduce failure in a red test before changing production code.
- 🧐 **CODE REVIEW / AUDIT:** Independent two-axis evaluation of changes against repository standards and spec invariants (use `code-review`).

### 💡 Intent-to-Task Sparringsprocedure (Idéfase)
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
5. **GAUNTLET**: Kør multi-layer verifikation via `cargo run -p xgauntlet-cli -- verify` / `xgauntlet verify`:
   - Linters & Static Analysis (`cargo clippy`, `cargo fmt --check`)
   - Type Checks & Kompilering (`cargo check`)
   - Acceptance & Unit Tests (`cargo test --workspace`)
   - Invariant & Spec Tests (`check-spec`)
   - Mutation Testing Gauntlet (`cargo mutants`)
6. **EVIDENCE**: Forsegl verifikationsrapport og evidens i `verification-report.json` og `evidence.md`.
7. **SESSION HANDOFF**: Display the clean `🏁 SESSION HANDOFF` card with the copy-paste starter prompt and inferred engineering role in the final user-facing response:
   > ### 🏁 SESSION HANDOFF • `<task_id>`
   > **Status**: `TASK: DONE` | **Evidens**: `FORSEGLET (Two-Tier Model)` | **Næste Rolle**: `<inferred_role>`
   > 💡 *Start venligst en frisk chat-session for at bevare et skarpt kontekstvindue uden context rot.*
   >
   > 📋 **Kopiér og indsæt følgende starter-prompt i en ny chat:**
   > ```text
   > <handoff_prompt>
   > ```

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

