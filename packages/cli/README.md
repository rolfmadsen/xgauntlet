<p align="center">
  <a href="#-hurtig-start--installation"><b>🚀 Hurtig Start & Installation</b></a> •
  <a href="#-pipeline-og-gates-sådan-virker-det"><b>🧭 Pipeline & Gates</b></a> •
  <a href="#-arkitektur--designprincipper"><b>🎯 Arkitektur</b></a> •
  <a href="#️-mappestruktur-package-by-feature"><b>🏗️ Mappestruktur</b></a> •
  <a href="#️-fuld-cli-reference"><b>🛠️ CLI Reference</b></a> •
  <a href="#-rust-api"><b>🦀 Rust API</b></a> •
  <a href="#️-arkitektur-adrs"><b>🏛️ ADRs</b></a>
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

Bygget i **Rust** for lynhurtig sub-3ms koldstart uden baggrundsdæmoner, og distribueret via **NPX** for øjeblikkelig brug på tværs af **Linux**, **macOS** og **Windows 11**.

---

## 🚀 Hurtig Start & Installation

Kom i gang på under 10 sekunder uden forudgående installation via NPX:

### 1. Initialiser dit projekt
Stil dig i rodmappen på dit projekt (Rust, TypeScript, Python eller polyglot) og kør:

```bash
# Åbn dit projektkatalog
cd ~/sti/til/dit-projekt

# Scaffold samtlige in-repo styringsfiler direkte via NPX
npx xgauntlet init
```

#### 📦 Hvad `xgauntlet init` opretter lokalt i projektet (In-Repo Single Source of Truth):
| Fil / Mappe | Formål |
|---|---|
| [`gauntlet.toml`](gauntlet.toml) | Deklarativ konfiguration af lintere, typer, tests og verifikationslag |
| [`CONTEXT.md`](CONTEXT.md) | Domæne-glossary for projektet (Aristoteles' *definitio per genus et differentiam*) |
| [`CODING_STANDARDS.md`](CODING_STANDARDS.md) | Multi-stack kodestandarder og arkitekturinvarianter |
| [`spec.md`](spec.md) | Makro-specifikation og system-invarianter |
| [`tasks/001-bootstrap.md`](tasks/) | Opgavemappe til håndhævelse af task-kontrakter & acceptkriterier |
| [`docs/adr/`](docs/adr/) | Architecture Decision Records (ADR) til projekt-specifikke beslutninger |
| [`.agents/AGENTS.md`](.agents/AGENTS.md) | Retningslinjer for AI-agenter, Response HUD og task-management |
| [`.agents/hooks.json`](.agents/hooks.json) | Pre-Invocation Hook til gatekeeperen |
| [`CLAUDE.md`](CLAUDE.md) | Retningslinjer og sikkerhedsinvarianter for Claude Code |

> [!TIP]
> **🛡️ Ikke-destruktiv Garanti (Safety First):**  
> `xgauntlet init` overskriver **aldrig** eksisterende filer i dit projekt, medmindre du udtrykkeligt angiver `--force`.

### 2. Kør Verifikation & Tjek Evidens
Når du eller agenten arbejder på en opgave i projektet, afvikles gauntlettet direkte:

```bash
# Kør gauntlet og forseg evidens for en opgave:
npx xgauntlet verify --task-id 001-bootstrap

# Valider kildetræets integritet mod rapporten (drift-kontrol):
npx xgauntlet check-evidence

# Valider opgavespecifikation og CONTEXT.md definitionsformat:
npx xgauntlet check-spec

# Kør miljø- og toolchain-diagnostik:
npx xgauntlet doctor
```

> [!IMPORTANT]
> **🚪 Zero Lock-in & Ren Afinstallation:**  
> Alt ligger lokalt i dit Git-træ. xGauntlet efterlader ingen globale dæmoner, baggrundsprocesser eller systemændringer på din maskine.

---

## 🧭 Pipeline og Gates: Sådan virker det

`xGauntlet` arbejder på to adskilte niveauer:
1. **Metodiske retningslinjer (Agent Skills)**: Proceskrav (såsom sokratisk grilling, TDD-disciplin og arkitektur-review), som agenten instrueres i at følge (`grill-me`, `old-coder`, `code-review`).
2. **Håndhævede software-gates (CLI & In-Memory WASM)**: Deterministiske kontrolpunkter i Rust-koden (`check-spec`, pre-invocation WASM hook, `verify`, `check-evidence` og `check-release`), der fysisk blokerer uautoriserede handlinger med exit-koder og multi-digest integritetskontrol.

### Oversigt over udviklingsflowet

```text
[ 1. IDÉAFKLARING ]             ──► Metodisk skill (grill-me / grill-with-docs)
         │                          (Opdaterer CONTEXT.md & docs/adr/)
         ▼
[ 2. SPECIFIKATION ]            ──► HÅRD GATE: xgauntlet check-spec
         │                          (OKF metadata, formål, kriterier, Must NOT, Aristoteles-glossar)
         ▼
[ 3. TDD & KODNING ]            ──► HÅRD GATE: Zero-Ambient-Authority WASM Policy Hook
         │                          (Kræver aktiv task for beskyttede stier, forhindrer git push)
         ▼
[ 4. FLERLAGS VERIFIKATION ]    ──► HÅRD GATE: xgauntlet verify
         │                          (Kører linters, typer, tests; matematisk pre/post manifestkontrol)
         ▼
[ 5. KODESTANDARD-REVIEW ]      ──► Hybrid skill: code-review & udvikleraccept
         │                          (Audit mod CODING_STANDARDS.md)
         ▼
[ 6. DRIFT-KONTROL ]            ──► HÅRD GATE: xgauntlet check-evidence
         │                          (Verificerer at workspace matcher rapporten via Git OID hash)
         ▼
[ 7. RELEASE READINESS ]        ──► HÅRD GATE: xgauntlet check-release
                                    (Versionssynkronisering, CHANGELOG.md og ADR-krydsreferencer)
```

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
├── docs/adr/                     # Arkitekturbeslutninger (ADRs 0001-0007)
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
│   │       ├── policy/           # Capability requests & trusted context
│   │       ├── evidence/         # Git OID SHA-256 manifest & verification-report
│   │       ├── gauntlet/         # Multi-layer runner, processtyring, timeouts
│   │       ├── diagnostics/      # Actionable diagnostics parser
│   │       ├── tasks/            # OKF v0.2 parser & check-spec
│   │       ├── adapters/         # Vertikale harness-slices (Antigravity, Claude, Codex)
│   │       ├── config/           # gauntlet.toml loader & schema
│   │       ├── scaffold/         # Ikke-destruktiv init-motor
│   │       └── doctor/           # Miljø- og toolchain-diagnostik
│   │
│   └── xgauntlet-cli/            # Native CLI executable (xgauntlet)
│
└── packages/
    └── cli/                      # NPM distributionspakke (xgauntlet)
        └── bin/xgauntlet.js      # Zero-dependency platform bootstrapper
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

### 5. Release Readiness (`check-release`)
Validerer at versioner matcher på tværs af manifests (`Cargo.toml`, `package.json`), `CHANGELOG.md` og ADR-krydsreferencer:
```bash
xgauntlet check-release
```

### 6. Workspace Diagnostik (`doctor`)
Undersøg workspace-konfiguration, stakke og miljøforudsætninger på tværs af Linux, macOS og Windows:
```bash
xgauntlet doctor
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

---

## 🙏 Anerkendelse & Inspiration (Credits)

`xGauntlet` bygger videre på idéer og pionerarbejde inden for stringent verifikation:
- **[Robert C. Martin ("Uncle Bob")](https://x.com/unclebobmartin/status/2080257779395154409)**: For TDD, Clean Craftsmanship og idéen om at omgive koden med en uomgængelig *gauntlet*.
- **[amazingang (old-coder)](https://github.com/amazingang/old-coder)**: For formuleringen af Evidence-First filosofien (*"Trust moves from inspection to constraints"*).
- **[Matt Pocock](https://github.com/mattpocock)**: For sokratiske workflow-skills (`grill-me`, `grill-with-docs`, `code-review`).
- **[Bytecode Alliance](https://bytecodealliance.org/)**: For Wasmtime og WebAssembly Component Model med formel Zero Ambient Authority.
