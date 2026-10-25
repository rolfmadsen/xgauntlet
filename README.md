<p align="center">
  <a href="#-hurtig-start--installation"><b>🚀 Hurtig Start & Installation</b></a> •
  <a href="#-pipeline-og-gates-sådan-virker-det"><b>🧭 Pipeline & Gates</b></a> •
  <a href="#-arkitektur--designprincipper"><b>🎯 Arkitektur</b></a> •
  <a href="#️-mappestruktur-package-by-feature"><b>🏗️ Mappestruktur</b></a> •
  <a href="#️-fuld-cli-reference"><b>🛠️ CLI Reference</b></a> •
  <a href="#-rust-api"><b>🦀 Rust API</b></a> •
  <a href="#️-arkitektur-adrs"><b>🏛️ ADRs</b></a> •
  <a href="#-licens"><b>📄 Licens</b></a>
</p>

---

<p align="center">
  <h1 align="center">xGauntlet 🛡️</h1>
</p>

<p align="center">
  <em>Universel multi-stack verifikations- og actionable diagnostics motor bygget på Robert C. Martin ("Uncle Bob") TDD, Clean Craftsmanship & Zero-Daemon WebAssembly</em>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/xgauntlet"><img src="https://img.shields.io/npm/v/xgauntlet.svg?color=blue" alt="NPM Version" /></a>
  <a href="https://github.com/rolfmadsen/xgauntlet/actions/workflows/ci.yml"><img src="https://github.com/rolfmadsen/xgauntlet/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI" /></a>
  <a href="docs/adr/0005-two-tier-verification-and-attestation-model.md"><img src="https://img.shields.io/badge/evidence-Two--Tier%20Trust%20Model-blue.svg" alt="Evidence Model" /></a>
  <a href="wit/gauntlet_policy.wit"><img src="https://img.shields.io/badge/WASM-Zero%20Ambient%20Authority-purple.svg" alt="WASM Policy" /></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/rust-2021%20edition-orange.svg" alt="Rust Version" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT" /></a>
</p>

---

**Dokumentation**: [Makro-Spec](spec.md) • [Domæne-Glossary](CONTEXT.md) • [Kodestandarder](CODING_STANDARDS.md) • [Arkitektur (ADRs)](docs/adr/)

**Kildekode & Pakker**: [GitHub](https://github.com/rolfmadsen/xgauntlet) • [NPM Pakke](packages/cli)

---

**xGauntlet** omgiver AI-genereret kode med et kompromisløst verifikations-gauntlet (Linters, Type-checkere, Unit tests, Invariant-tests og Mutationsafprøvning), håndhæver en deterministisk **Zero Ambient Authority WebAssembly policy-kerne** (`wit/gauntlet_policy.wit`) og oversætter rå fejludskrifter til **Actionable Diagnostics** i et feedback-loop, som AI-agenter kan handle direkte på.

Bygget i **Rust** for lynhurtig sub-3ms koldstart uden baggrundsdæmoner, og distribueret via **NPM** og **Cargo** for øjeblikkelig brug på tværs af **Linux**, **macOS** og **Windows 11**.

---

## 🚀 Hurtig Start & Installation

xGauntlet følger samme enkle model som `git`: Værktøjet installeres én gang globalt på maskinen, hvorefter du initialiserer det lokalt i de repositories, du ønsker at sætte under governance.

### 1. Installer xGauntlet (én gang på din maskine)

Kør følgende tre trin for at installere CLI-motoren globalt, hente den verificerede native engine og aktivere de 6 suveræne agent-skills:

```bash
# Trin 1a: Installer CLI-motoren og gatekeeperen globalt i dit system ($PATH):
npm install -g xgauntlet

# Trin 1b: Fetch og verificer native engine via Sigstore i ~/.cache/xgauntlet/
xgauntlet status

# Trin 1c: Installer de 6 suveræne agent-skills i dine AI-harnesses (Antigravity, Claude Code, Codex, Mistral):
xgauntlet plugin install --global
```

> [!NOTE]
> **⚡ Hvordan fungerer den automatiske bootstrapper & cache?**  
> NPM-pakken indeholder en lynhurtig JavaScript-bootstrapper. Første gang du kører en `xgauntlet`-kommando (f.eks. `xgauntlet status` eller `xgauntlet init`), downloader den automatisk den prækompilerede native Rust-binærfil til din platform fra GitHub Releases, verificerer dens kryptografiske Sigstore-attestation og cacher den i:
> - **Linux / macOS**: `~/.cache/xgauntlet/v<version>/xgauntlet`
> - **Windows**: `%LOCALAPPDATA%\xgauntlet\v<version>\xgauntlet.exe`  
>
> *(Hvis du arbejder inde i xGauntlets eget kildekode-repository, bruger bootstrapperen automatisk din lokale `target/debug/xgauntlet` direkte).*

> [!NOTE]
> **🤖 Hvorfor kræves `xgauntlet plugin install --global`?**  
> `npm install -g` placerer udelukkende CLI-binæren i dit systems `$PATH`. Hvert AI-agentmiljø (Google Antigravity IDE, Claude Code, OpenAI Codex, Mistral Vibe) har sin egen konfigurationsmappe til skills (f.eks. `~/.gemini/config/plugins/xgauntlet/skills/` eller `~/.claude/skills/`).  
> `xgauntlet plugin install --global` detekterer automatisk dine installerede agent-harnesses og udpakker de 6 suveræne skills (`discover`, `spec`, `tdd`, `review`, `diagnose`, `release`) som ægte, redigerbare Markdown-filer direkte i agenternes globale mapper.

> [!TIP]
> **⚡ Hvorfor global installation frem for blot `npx`?**  
> Når `xgauntlet` er installeret globalt i dit `$PATH`, kører gatekeeper-hooks (`xgauntlet hook ...`) direkte som maskinkode på **under 3 millisekunder** ved hvert eneste værktøjskald fra din AI-agent – helt uden Node.js koldstarts-overhead.  
> *(Du kan dog stadig køre `npx xgauntlet init`, hvis du blot vil afprøve værktøjet flygtigt uden installation).*

### 2. Initialiser dit projekt (In-Repo Governance)

Gå ind i dit projekt – uanset om det er skrevet i Python, Rust, Go eller TypeScript:

```bash
# Åbn dit projektkatalog
cd ~/sti/til/dit-projekt

# Scaffold in-repo styringsfiler:
xgauntlet init
```

`xgauntlet init` detekterer automatisk projektets programmeringssprog (via `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod` osv.) og opretter de deklarative styringsfiler.

#### 📦 Hvad `xgauntlet init` opretter lokalt i projektet (In-Repo Single Source of Truth):

| Fil / Mappe | Type | Formål |
|---|---|---|
| [`gauntlet.toml`](gauntlet.toml) | **Fælles** | Deklarativ konfiguration af lintere, typer, tests og verifikationslag |
| [`CONTEXT.md`](CONTEXT.md) | **Fælles** | Domæne-glossary for projektet (Aristoteles' *definitio per genus et differentiam*) |
| [`CODING_STANDARDS.md`](CODING_STANDARDS.md) | **Fælles** | Multi-stack kodestandarder og arkitekturinvarianter |
| [`spec.md`](spec.md) | **Fælles** | Makro-specifikation og system-invarianter |
| [`tasks/001-bootstrap.md`](tasks/) | **Fælles** | Opgavemappe til håndhævelse af task-kontrakter & acceptkriterier |
| [`docs/adr/`](docs/adr/) | **Fælles** | Architecture Decision Records (ADR) til projekt-specifikke beslutninger |
| [`.agents/AGENTS.md`](.agents/AGENTS.md) | **Antigravity / Gemini** | Retningslinjer for AI-agenter, Response HUD og task-management |
| [`.agents/hooks.json`](.agents/hooks.json) | **Antigravity / Gemini** | PreToolUse gatekeeper hook konfiguration |
| [`CLAUDE.md`](CLAUDE.md) | **Claude Code** | Retningslinjer og sikkerhedsinvarianter for Anthropic Claude Code |

> [!NOTE]
> **🤝 Poly-Harness Sameksistens:**  
> De 7 øverste filer er helt universelle og styrer projektet for både mennesker og alle typer AI-agenter. De to harness-specifikke sæt (`CLAUDE.md` og `.agents/`) sameksisterer fredeligt uden konflikter, så du eller dit team frit kan skifte mellem f.eks. Google Antigravity IDE og Claude Code på det samme projekt. Hvis du udelukkende benytter én agent, kan det overskydende sæt frit slettes.

> [!TIP]
> **🛡️ Ikke-destruktiv Garanti (Safety First):**  
> `xgauntlet init` overskriver **aldrig** eksisterende filer i dit projekt, medmindre du udtrykkeligt angiver `--force`.

### 3. Kør Verifikation & Tjek Evidens

Når du eller agenten arbejder på en opgave i projektet, afvikles gauntlettet direkte:

```bash
# Kør gauntlet og forsegl evidens for en opgave:
xgauntlet verify --task-id 001-bootstrap

# Valider kildetræets integritet mod rapporten (drift-kontrol):
xgauntlet check-evidence

# Valider opgavespecifikation og CONTEXT.md definitionsformat:
xgauntlet check-spec

# Kør miljø- og toolchain-diagnostik:
xgauntlet doctor
```

---

### 🧹 Afinstallation & Oprydning

xGauntlet kører efter **Zero Lock-in** princippet. Der er ingen baggrundsdæmoner, ingen systemd services og ingen skjulte registre:

#### 1. Afkobl et projekt (Kirurgisk In-Repo Oprydning)
For at fjerne xGauntlet fra et projekt, fjerner du blot de specifikke styringsfiler. Dine egne kodefiler, egne tasks og egne ADR'er berøres ikke:

```bash
# Fjern xGauntlets kontrolfiler i projektet:
rm -f gauntlet.toml .agents/hooks.json verification-report.json evidence.md

# (Valgfrit) Fjern de oprindeligt scaffoldede skabeloner, hvis du ikke ønsker at bevare dem:
rm -f tasks/001-bootstrap.md docs/adr/0001-package-by-feature-architecture.md
```

#### 2. Afinstaller værktøjet fra maskinen (Global Cleanup)
Hvis du ønsker at fjerne selve `xgauntlet`-programmet og de globale agent-skills fra dit styresystem:

```bash
npm uninstall -g xgauntlet

# Slet den lokale cache-binær:
rm -rf ~/.cache/xgauntlet

# (Valgfrit) Fjern installerede agent-plugins fra dine globale AI-harnesses:
rm -rf ~/.gemini/config/plugins/xgauntlet ~/.claude/skills/xgauntlet ~/.codex/plugins/xgauntlet ~/.vibe/plugins/xgauntlet
```

---

## 🧭 Pipeline og Gates: Sådan virker det

`xGauntlet` arbejder på to adskilte niveauer:
1. **Metodiske retningslinjer (Agent Skills)**: Proceskrav (såsom sokratisk behovsafklaring, TDD-disciplin og arkitektur-review), som agenten instrueres i at følge (`discover`, `spec`, `tdd`, `review`, `diagnose`, `release`).
2. **Håndhævede software-gates (CLI & In-Memory WASM)**: Deterministiske kontrolpunkter i Rust-koden (`check-spec`, pre-invocation WASM hook, `verify`, `check-evidence`, `check-attestation`, `check-architecture` og `check-release`), der fysisk blokerer uautoriserede handlinger med exit-koder og multi-digest integritetskontrol.

> [!NOTE]
> **Bemærk om tilstande og Zero-Daemon:** Systemet styres ikke af en global database eller baggrundsdæmon, men af diskrete, uafhængige CLI-kald (`sub-3ms` koldstart). Opgavestatus (`DRAFT`, `ACTIVE`, `PASSED`, `DONE`) er forankret direkte i de enkelte task-filers OKF YAML-frontmatter (`tasks/*.md`), som evalueres on-demand af de respektive gates uden behov for en tilstandsmaskine-service.

### Oversigt over udviklingsflowet

```text
[ 1. IDÉAFKLARING ]             ──► Metodisk skill (discover)
         │                          (Opdaterer CONTEXT.md & docs/adr/)
         ▼
[ 2. SPECIFIKATION ]            ──► HÅRD GATE: xgauntlet check-spec
         │                          (OKF metadata, formål, kriterier, Must NOT, Aristoteles-glossar)
         ▼
[ 3. TDD & KODNING ]            ──► HÅRD GATE: Zero-Ambient-Authority WASM Policy Hook
         │                          (Quote-aware tokenization, task scope, forhindrer wrappers & git push)
         ▼
[ 4. FLERLAGS VERIFIKATION ]    ──► HÅRD GATE: xgauntlet verify
         │                          (Kører linters, typer, tests; matematisk pre/post manifestkontrol)
         ▼
[ 5. KODESTANDARD-REVIEW ]      ──► Hybrid skill: review & udvikleraccept
         │                          (Audit mod CODING_STANDARDS.md & Fowler smells)
         ▼
[ 6. DRIFT- & ATTESTATION ]     ──► HÅRD GATE: xgauntlet check-evidence & check-attestation
         │                          (Git OID drift-kontrol og kryptografisk Sigstore OIDC verifikation)
         ▼
[ 7. RELEASE READINESS ]        ──► HÅRD GATE: xgauntlet check-release (--strict) & release bump
                                    (WASM-sync, manifest harmony, changelog og transaktionel staging)
```

### Pipelinen trin for trin

#### 1. Idé- og kontekstafklaring
* **Type**: Metodisk proces (Agent Skill)
* **Hvad der sker**: Før der skrives specifikationer eller kode, aktiveres sparringsskillen `discover`. Agenten udfordrer antagelser, identificerer risici og afstemmer planer mod eksisterende arkitektur og ADR'er.
* **Kontrolpunkt**:
  * *Hvem godkender*: Udvikleren i direkte dialog.
  * *Hvordan*: Dialogen udmønter sig i, at agenten opdaterer `CONTEXT.md` og eventuelt udarbejder en ny ADR i `docs/adr/`.
  * *Håndhævelse i koden*: Dette er et instruktionskrav til agenten. Der findes ingen automatisk kodelås, der forhindrer oprettelse af tasks uden forudgående grilling; disciplinen bæres af udviklerens sparring med agenten. WASM-policykernen tillader udtrykkeligt skrivning til styringsdokumenter (`tasks/`, `spec.md`, `CONTEXT.md`, `CODING_STANDARDS.md`, `docs/`) uanset opgavestatus.

#### 2. Specifikation & Opgavebinding
* **Type**: Hård software-gate
* **Hvad der sker**: Opgaven defineres formelt i en markdown-fil under `tasks/` (f.eks. `tasks/001-bootstrap.md`) med eksplicit OKF v0.2 frontmatter samt eksekverbare acceptkriterier.
* **Kontrolpunkt**: Spec Gate (`xgauntlet check-spec`)
  * *Hvem godkender*: Shift-left valideringsmotoren i Rust (`crates/xgauntlet-core/src/features/tasks/validator.rs`).
  * *Hvordan*: CLI-værktøjet parser task-filerne og validerer:
    1. Valid OKF v0.2 YAML-frontmatter (`type: Task Package`, `status`, `title`, `generated`).
    2. Eksistensen af sektionen `## 🎯 Formål` (eller `## Purpose`).
    3. Eksistensen af eksekverbare acceptkriterier (`- [ ]`).
    4. Eksistensen af negative forretningsregler under `## 🚫 Must NOT`.
    5. At definitionerne i `CONTEXT.md` følger Aristoteles' formel (`**Term**:\n<Definition>\n_Avoid_: <synonymer>`).
  * *Håndhævelse i koden*: Returnerer exit-kode 1, hvis en task-fil mangler, er fejlbehæftet eller overtræder formateringskravene.

#### 3. Implementering under Zero-Ambient-Authority Policy (Værkstedet)
* **Type**: Metodisk TDD + Hård runtime-beskyttelse
* **Hvad der sker**: Koden skrives efter Red/Green TDD-princippet (først en fejlende test, derefter den minimale kode, der løser den, og til sidst refaktorisering).
* **Kontrolpunkt**: Pre-Invocation WASM Hook & Anti-Tamper Guard
  * *Hvem godkender*: Indlejret Wasmtime policy-motor (`gauntlet_policy.wasm` bygget fra `wit/gauntlet_policy.wit`) og harness-adaptere (`crates/xgauntlet-core/src/features/adapters/`).
  * *Hvordan*: Agentens værktøjskald (tool calls) overvåges deterministisk ved hvert kald via `xgauntlet hook <harness>`.
  * *Håndhævelse i koden*:
    * **Zero Ambient Authority**: WebAssembly-modulet har absolut nul adgang til værtsfilsystem, netværk, systemur eller tilfældighedskilder.
    * **Struktureret Command Tokenization**: Shell-kommandoer evalueres via en quote-aware argument-lexer. Eksekverings-wrappers (`env`, `sudo`, `eval`, `bash -c`, `sh -c`, `xargs`, `doas`, `su`, `exec`) afvises fail-closed med `reason_code: 4039`.
    * **Fail-Closed Task Scope**: Filmodifikationer (`WriteFile`) uden for den aktive opgaves erklærede `scope` blokeres hårdt (`reason_code: 4037`). Hvis task scope er udefineret, tillader test-scope isolation udelukkende skrivning til verificerede testfiler.
    * **Destruktiv Beskyttelse**: Destruktive kommandoer som `git push`, `git reset --hard`, `git clean -f`, `git branch -D` og `rm -rf /` blokeres uanset globale flag (`-C`, `-c`, `--no-pager`).
    * **Harness Compliance**: Understøtter officielt fladt `PreInvocation` skema for Google Antigravity og sikker, ikke-destruktiv hook-scaffolding mod JSON-korruption.
  * *Bemærk om TDD*: Selve rækkefølgen (Red før Green) registreres ikke historisk af test-runneren; det er en metodisk adfærd instrueret via agent-skills.

#### 4. Flerlags Verifikation
* **Type**: Hård software-gate
* **Hvad der sker**: Fuld automatisk eksekvering af projektets test- og analysesuiter med matematisk beskyttelse mod selvmutation og generering af forseglede rapporter.
* **Kontrolpunkt**: Diagnostic & Execution Engine (`xgauntlet verify`)
  * *Hvem godkender*: Verifikationspipelinen i Rust (`crates/xgauntlet-core/src/features/gauntlet/pipeline.rs`).
  * *Hvordan*: Runneren eksekverer de lag, der er defineret i `gauntlet.toml` (f.eks. Typer, Linters, Tests, Invarianter & Mutationer) med fail-closed semantik og timeouts:
    1. **Pre-manifest beregning**: Beregner kildetræets Git-blob OID digests forud for testkørsel.
    2. **Lag-eksekvering**: Kører lagene sekventielt; ved fejl parses output til Actionable Diagnostics (`DiagnosticParser`).
    3. **Post-manifest & Anti-Tamper**: Genberegner manifest efter kørsel; hvis kildekode eller testassertions muteres undervejs, afvises kørslen øjeblikkeligt (`verify_self_mutation`).
  * *Håndhævelse i koden*: Alle obligatoriske diagnostiske lag skal bestå (`exit_code == 0`). Ved succes genereres atomart `verification-report.json` (Schema v2) og `evidence.md` med deterministiske SHA-256 digests over kildetræ (`source_manifest_digest`), konfiguration, opgave og politikker.

#### 5. Review mod Kodestandarder
* **Type**: Hybrid gate (Agent Skill + Udvikleraccept)
* **Hvad der sker**: Den implementerede løsning auditeres mod arkitekturretningslinjer og regler i `CODING_STANDARDS.md`.
* **Kontrolpunkt**: Standards Review (`review` skill)
  * *Hvem godkender*: Udvikleren assisteret af agentens review-skill.
  * *Hvordan*: Agenten gennemgår diff'en op mod kodestandarderne og fremhæver eventuelle arkitekturbrud, manglende fejlhåndtering eller navngivningsfejl.
  * *Håndhævelse i koden*: Gaten er procesmæssig og beror på agentens review-rapport kombineret med udviklerens godkendelse.

#### 6. Drift- og Integritetskontrol (Two-Tier Model)
* **Type**: Hård software-gate
* **Hvad der sker**: Verificering af, at kildekoden og arbejdstræet ikke er blevet manipuleret eller er driftet efter testkørslen.
* **Kontrolpunkt**: Drift Verification (`xgauntlet check-evidence`) & Attestation Gate (`xgauntlet check-attestation`)
  * *Hvem godkender*: Drift-detektionsmotoren (`crates/xgauntlet-core/src/features/evidence/drift.rs`) og attestation-verifikatoren (`attestation.rs`).
  * *Hvordan*: Værktøjet genberegner det aktuelle kildetræs workspace-manifest og sammenligner det direkte med værdierne i `verification-report.json`.
  * *Håndhævelse i koden*:
    * **Tier 1 (Lokal drift-kontrol)**: Er blot én byte ændret i kildetræ, opgave, config eller politik efter `verify`, afvises tjekket med en specifik `DriftViolation`. Beregningen benytter Git-tree/blob OID-hashes, hvilket gør driftkontrollen fuldstændig immun over for linjeskift-forskelle (LF vs. CRLF) på tværs af styresystemer.
    * **Tier 2 (Kryptografisk Attestation i CI)**: Jf. [ADR 0005](docs/adr/0005-two-tier-verification-and-attestation-model.md) udstedes der i beskyttede CI-miljøer en kryptografisk in-toto/DSSE-attest (Sigstore/OIDC), som forsegler kildens og binærens herkomst (`attestation.bundle` og `attestation.json`). Denne kan efterprøves lokalt eller remote via `xgauntlet check-attestation`.

#### 7. Release Readiness
* **Type**: Hård software-gate
* **Hvad der sker**: Koden klargøres til release og merge ved at kontrollere synkronisering mellem versioner, ændringslog, WASM-bytecode og dokumentation.
* **Kontrolpunkt**: Release Gate (`xgauntlet check-release`) & Transaktionel Bump (`xgauntlet release bump`)
  * *Hvem godkender*: Release-orkestratoren (`crates/xgauntlet-core/src/features/release/engine.rs`).
  * *Hvordan*: Værktøjet udfører dybdegående kontroller:
    1. **Versionskonsistens**: Versionsnumre skal matche på tværs af projektets manifests (`Cargo.toml`, `package.json`, `packages/cli/package.json`).
    2. **WASM Bytecode Paritet**: Det indlejrede WebAssembly-bytecode valideres mod workspace-versionen (`CONFIG_VERSION_MISMATCH`).
    3. **Changelog-synkronisering**: `CHANGELOG.md` skal indeholde en sektion for den pågældende version.
    4. **ADR-referencer**: Samtlige ADR-dokumenter i `docs/adr/` skal være eksplicit refereret eller linket i enten `README.md` eller `spec.md`.
    5. **Strict Release Mode (`--strict`)**: Validerer rent git-arbejdstræ (med automatisk udelukkelse af cirkulære artefakter) og eksekverer kodeformatering fail-closed.
    6. **Transaktionel Release Bump**: `xgauntlet release bump [patch|minor|major]` udfører atomisk disk-staging backup (`.xgauntlet_staging_<nonce>`), opdaterer manifests, kompilerer WASM og ruller sikkert tilbage ved fejl uden destruktive git-handlinger.

### 👥 De 4 AI-roller & Session Handoff

For at undgå uendelige review-loops (*bikeshedding*) og bevare et skarpt kontekstvindue, udleder `xGauntlet` automatisk den næste ingeniør-rolle:

1. **`Senior Software Engineer (System Architecture & Requirements)`**:
   * Aktiveres ved nye eller `DRAFT`-opgaver. Udfordrer antagelser, definerer negative invarianter (`## 🚫 Must NOT`) og eksekverbare kriterier via `xgauntlet check-spec`.
2. **`Senior Software Engineer (Feature Implementation & Testing)`**:
   * Aktiveres ved `ACTIVE`-opgaver. Driver TDD-cyklussen (`RED` $\to$ `GREEN` $\to$ `REFACTOR`) og forsegler evidens via `xgauntlet verify`.
3. **`Senior Software Engineer (Independent Code Review & Audit)`**:
   * Tager over i en frisk session. Udfører to-akset granskning langs **Akse A (Standarder)** jf. `CODING_STANDARDS.md` og **Akse B (Krav)** jf. `spec.md`/`tasks/`.
4. **`Release & Operations Engineer (Release Attestation & Deployment)`**:
   * Tager over når alle opgaver og audits er godkendt. Kører `xgauntlet check-release`, synkroniserer versioner og klargør release.

> [!NOTE]
> **🛡️ Single-Agent Governance & Human-in-the-Loop (HITL):**
> xGauntlet anvender en transparent enkelt-agent model under direkte menneskeligt opsyn. Kontekst-renhed opnås via mekaniske Session Handoffs, ikke via uigennemskuelige baggrunds-subagenter. Udvikleren bevarer hermed fuld kontrol og sporbarhed over hver eneste sessionsovergang.

---

## 🎯 Arkitektur & Designprincipper

1. **Uncle Bob Clean Architecture & TDD:**
   * Forankret i de 3 Love for TDD, Transformation Priority Premise (TPP) og Single Responsibility Principle (SRP).
2. **Package-by-Feature Struktur (Screaming Architecture):**
   * Hver komponent er isoleret i en feature-underpakke under `crates/xgauntlet-core/src/features/` med høj sammenhørighed og lav kobling ([ADR 0001](docs/adr/0001-package-by-feature-architecture.md)).
3. **Zero-Daemon & Sub-3ms Koldstart:**
   * Pure native **Rust** eksekverbar binær. Ingen baggrundsdæmoner, ingen `systemd`/`launchd` services, ingen socket-nedbrud. Hver kommando udføres lynhurtigt in-process.
4. **Nul Ambient Authority WebAssembly Policy Kerne ([ADR 0007](docs/adr/0007-local-transparent-supervisor-and-wasm-verifier.md)):**
   * Policy-evaluering kompileres fra `wit/gauntlet_policy.wit` til `gauntlet_policy.wasm`.
   * Eksekveres in-memory via indlejret **Wasmtime** med absolut nul adgang til filsystem, netværk, system-ur eller tilfældighedskilder.
5. **Matematisk Anti-Tamper Manifest (Self-Mutation Invariant):**
   * Pre- og post-test beregning af Git-tree/blob OID digests (immune over for Windows LF/CRLF konverteringer).
   * Hvis kildekode eller testassertions muteres undervejs i testkørslen, afvises verifikationen øjeblikkeligt.
6. **Two-Tier Evidens & Tillidsmodel ([ADR 0005](docs/adr/0005-two-tier-verification-and-attestation-model.md)):**
   * `LOCAL_UNSUPERVISED`: Lokal deterministisk verifikationsrapport for hurtig feedback og matematisk drift-kontrol (`xgauntlet check-evidence`).
   * `CI_ATTESTED`: Autoritativ, uafviselig Sigstore OIDC / in-toto attestation udstedt i et isoleret CI-miljø.
7. **Multi-Harness Native Support ([ADR 0004](docs/adr/0004-harness-adapter-slices.md)):**
   * Autonome vertikale slices for **Google Antigravity IDE**, **Claude Code** og **OpenAI Codex**.

---

## 🏗️ Mappestruktur (`Package-by-Feature`)

```text
xGauntlet/
├── tasks/                        # Aktive og afsluttede opgavepakker (OKF v0.2)
├── docs/adr/                     # Arkitekturbeslutninger (ADRs 0001-0009)
├── CONTEXT.md                    # Domæne-glossary (Aristoteles' genus et differentiam)
├── CODING_STANDARDS.md           # Multi-stack kodestandarder (Rust, TypeScript, Python)
├── spec.md                       # Makro-specifikation & system-invarianter
├── gauntlet.toml                 # Deklarativ multi-stack konfiguration
├── wit/                          # WebAssembly Interface Types (gauntlet_policy.wit)
├── Cargo.toml                    # Cargo Workspace root
├── package.json                  # NPM Workspace root
│
├── crates/
│   ├── gauntlet-policy-engine/   # Pure WASM component (wasm32-wasip1/wasip2)
│   ├── xgauntlet-core/           # Portabel domænemotor (Package-by-Feature)
│   │   └── src/features/
│   │       ├── wasm/             # Embedded Wasmtime host integration
│   │       ├── policy/           # Capability requests & trusted context (structured tokenization)
│   │       ├── evidence/         # Git OID SHA-256 manifest, verification-report og Sigstore attestation
│   │       ├── gauntlet/         # Multi-layer runner, processtyring, timeouts
│   │       ├── diagnostics/      # Actionable diagnostics parser
│   │       ├── tasks/            # OKF v0.2 parser & check-spec
│   │       ├── adapters/         # Vertikale harness-slices (Antigravity, Claude, Codex)
│   │       ├── config/           # gauntlet.toml loader & schema
│   │       ├── scaffold/         # Ikke-destruktiv init-motor
│   │       ├── doctor/           # Miljø- og toolchain-diagnostik
│   │       └── release/          # Release gatekeeper, WASM-sync og transaktionel bump
│   │
│   └── xgauntlet-cli/            # Native CLI executable (xgauntlet)
│
└── packages/
    └── cli/                      # NPM distributionspakke (xgauntlet)
        └── bin/xgauntlet.js      # Zero-dependency bootstrapper med Sigstore integritetskontrol
```

---

## 🛠️ Fuld CLI Reference

### 1. Initialiser Workspace (`init`)
```bash
# Standard auto-detektering af stack
xgauntlet init

# Tving overskrivning af eksisterende skabeloner
xgauntlet init --force
```

### 2. Kør Verifikations-Gauntlet (`verify`)
Kør alle konfigurerede lag, udtræk actionable diagnostics og generer `verification-report.json` og `evidence.md`:
```bash
# Standard kørsel bundet til en opgave
xgauntlet verify --task-id 001-bootstrap

# Returner struktureret JSON med actionable diagnostics til LLM / AI-agenter
xgauntlet verify --diagnostics-json
```

### 3. Validering af Evidens & Drift-kontrol (`check-evidence`)
Verificerer at det aktuelle kildetræ matcher den lokale verifikationsrapport via Git-tree/blob OID digest:
```bash
xgauntlet check-evidence
```

### 4. Early-Phase Spec & Business Rules Gatekeeper (`check-spec`)
Mekanisk validering af at opgavespecifikationer indeholder forretningsinvarianter (`Must NOT`), eksekverbare acceptkriterier og overholder [CONTEXT.md](CONTEXT.md) formatet:
```bash
xgauntlet check-spec
```

### 5. Release Readiness Gatekeeper (`check-release`)
Validerer versionskonsistens, WASM-bytecode synkronisering, `CHANGELOG.md` og ADR-krydsreferencer:
```bash
# Standard kørsel under udvikling
xgauntlet check-release --allow-unreleased

# Streng release mode (fail-closed ved uncommitted filer eller formateringsfejl)
xgauntlet check-release --strict
```

### 6. Kryptografisk Attesteringskontrol (`check-attestation`)
Verificerer Sigstore in-toto/DSSE attesteringsbundter (`attestation.bundle` / `attestation.json`) lokalt eller direkte mod GitHub Releases:
```bash
# Valider lokalt bundt mod release-artefakt
xgauntlet check-attestation --bundle attestation.bundle --artifact target/release/xgauntlet

# Valider remote mod GitHub release med statisk forankret repo-ID (1359184794)
xgauntlet check-attestation --remote-tag v0.8.4
```

### 7. Deklarativ Arkitekturkontrol (`check-architecture`)
Håndhæver kodestruktur, filstørrelsesgrænser (kildekode max 800 linjer, tests max 1.000 linjer) og forbudte API-tokens:
```bash
xgauntlet check-architecture
```

### 8. Transaktionel Versionsstyring (`release bump`)
Udfører atomisk SemVer opgradering på tværs af manifests, kompilerer og synkroniserer WASM bytecode, roterer CHANGELOG.md og ruller automatisk tilbage ved fejl via disk-staging:
```bash
# Bump patch, minor eller major
xgauntlet release bump patch
```

### 9. Sessions-Overdragelseskort (`handoff`)
Genererer et standardiseret `🏁 SESSION HANDOFF` telemetrikort til ren kontekstoverdragelse i AI-sessioner:
```bash
xgauntlet handoff --task-id 001-bootstrap
```

### 10. Workspace Diagnostik (`doctor`)
Undersøg workspace-konfiguration, stakke og miljøforudsætninger på tværs af Linux, macOS og Windows:
```bash
xgauntlet doctor
```

### 11. Global Plugin & Agent Skills Distribution (`plugin install`)
Installer eller opdater de 6 suveræne agent-skills (`discover`, `spec`, `tdd`, `review`, `diagnose`, `release`) på tværs af opdagede AI agent harnesses:
```bash
# Installer automatisk til alle detekterede agent-harnesses
xgauntlet plugin install --global

# Preview handlinger uden at skrive filer til disk
xgauntlet plugin install --global --dry-run

# Tving overskrivning ved opdatering til ny xgauntlet-version
xgauntlet plugin install --global --force
```

---

## 🦀 Rust API

Du kan integrere `xgauntlet-core` direkte i dine egne Rust test-runners eller agent-værktøjer:

```rust
use std::path::Path;
use xgauntlet_core::features::evidence::compute_workspace_manifest;

// Beregn deterministisk kildemanifest (Git OID SHA-256 multi-digest)
let manifest = compute_workspace_manifest(Path::new("."))?;
println!("Source manifest digest: {}", &manifest.source_manifest_digest[..16]);
```

---

## 🏛️ Arkitektur (ADRs)

Projektets invariante tekniske valg og designprincipper er dokumenteret i [`docs/adr/`](docs/adr/):
- 🏛️ **[ADR 0001](docs/adr/0001-package-by-feature-architecture.md)**: Package-by-Feature / Screaming Architecture
- 🏛️ **[ADR 0002](docs/adr/0002-cryptographic-evidence-authority.md)**: Cryptographic Evidence Authority (Superseded by ADR 0005)
- 🏛️ **[ADR 0003](docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md)**: Surgical Gatekeeper and No Remote Push
- 🏛️ **[ADR 0004](docs/adr/0004-harness-adapter-slices.md)**: Vertical Slice Harness Adapters
- 🏛️ **[ADR 0005](docs/adr/0005-two-tier-verification-and-attestation-model.md)**: Two-Tier Verification and Attestation Model
- 🏛️ **[ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md)**: Multi-Harness Policy Adapter Contract
- 🏛️ **[ADR 0007](docs/adr/0007-local-transparent-supervisor-and-wasm-verifier.md)**: Zero-Daemon WASM Policy Verifier
- 🏛️ **[ADR 0008](docs/adr/0008-sovereign-agentic-sdlc-task-scoping-and-design-tokens.md)**: Sovereign Agentic SDLC, Task Scoping, and Design Token Governance
- 🏛️ **[ADR 0009](docs/adr/0009-autonomous-tdd-inner-loop-and-boundary-human-gates.md)**: Autonomous TDD Inner Loop and Boundary Human-in-the-Loop Gates
- 🏛️ **[ADR 0010](docs/adr/0010-adr-bound-architectural-exemptions-and-monotonic-ratchets.md)**: ADR-Bound Architectural Exemptions and Monotonic Ratchet Governance

---

## 🙏 Anerkendelse & Inspiration (Credits)

`xGauntlet` bygger videre på idéer, filosofi og pionerarbejde inden for stringent software-verifikation, agent-craftsmanship og forsyningskædesikkerhed:
- **[Robert C. Martin ("Uncle Bob")](https://x.com/unclebobmartin/status/2080257779395154409)**: For TDD, Clean Craftsmanship og princippet om at omgive koden med en uomgængelig, automatiseret *gauntlet*.
- **[amazingang (old-coder)](https://github.com/amazingang/old-coder)**: For formuleringen af Evidence-First filosofien (*"Trust moves from inspection to constraints"*).
- **[Matt Pocock](https://github.com/mattpocock)**: For banebrydende arbejde med sokratiske agent-workflows og to-akset review-metodologi.
- **[Addy Osmani](https://addyosmani.com/)**: For metodiske agent-skills inden for kildestyring under implementering, metodisk forsimpling (*Chesterton's Fence*) og adversarial kvalitets-ratchets.
- **[Jesse Vincent (obra)](https://github.com/obra/superpowers)**: For Superpowers agent-metodikken, *The Iron Law of Debugging*, anti-goals og deterministisk branch-afslutning.
- **[Bytecode Alliance](https://bytecodealliance.org/)**: For Wasmtime og WebAssembly Component Model med formel Zero Ambient Authority.
- **[Sigstore & OpenSSF](https://sigstore.dev/)**: For keyless OIDC-attestation og in-toto standarder for uafviselig forsyningskædesikkerhed.

---

## 📄 Licens

Dette projekt er licenseret under [MIT Licensen](LICENSE).

