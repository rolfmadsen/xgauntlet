---
type: Task Package
title: "Task 021: AST Codebase Topology & Token-Optimized Discovery Engine"
description: "Etablere en deterministisk Rust-baseret AST-topologimotor til token-effektiv discovery og blast radius analyse i Idé- og Spec-fasen"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-12T09:36:42Z" }
tags: [topology, ast, discovery, token-optimization, jit-context, blast-radius, rust, zero-daemon, adr-0001, adr-0004]
---

# Task 021: AST Codebase Topology & Token-Optimized Discovery Engine

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-12`

## 🎯 Formål & Gennemskuelighed
Etablere en 100% deterministisk, sub-millisekund AST- og modul-topologimotor i `crates/xgauntlet-core` (`src/features/topology/`), der reducerer LLM'ens token-forbrug under kodebase-discovery fra 30.000–50.000 tokens til under 300 tokens pr. session jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md) og [ADR 0004](docs/adr/0004-harness-adapter-slices.md):

### 1. Relevante Faser (Hvornår bringes den i spil?)
- **`SPEC / DISCOVERY` (Idé- & Afklaringsfasen)**:
  * *Udløser*: Når der *ikke* er en aktiv opgave i `tasks/`, eller når brugeren stiller udforskende spørgsmål (*"Hvor håndteres X?", "Hvilke moduler berøres af Y?"*).
  * *Adfærd*: Telemetrien leverer et lynhurtigt, struktureret højniveau-kort over systemets forretningsfeatures, deres indbyrdes afhængigheder og kerne-entiteter.
- **`RED / DIAGNOSE` (Fejlfindings- & Blast Radius fasen)**:
  * *Udløser*: Når en test fejler, eller når koden refaktoreres.
  * *Adfærd*: Topologimotoren isolerer afhængighedskæden (hvem kalder det fejlende modul, og hvilke downstream komponenter risikerer regression).
- **`GREEN / REFACTOR` (Kvalitets- & Invariantfasen)**:
  * *Adfærd*: Kontrollerer mekanisk at Package-by-Feature modulgrænser ikke overtrædes af utilsigtede cykliske afhængigheder.

### 2. Teknologi (Hvad bygger vi det på?)
- **100% ren Rust i `xgauntlet-core`**:
  * Ingen eksterne Python-runtimes, ingen tunge GraphRAG/Neo4j databaser, og **0 tokens forbrugt til at bygge grafen**.
  * Deterministisk AST- og import-ekstraktion for de understøttede stacks (Rust `mod.rs`/`Cargo.toml`, TypeScript/Node `package.json`/`import`, Python `import`, Go `go.mod`).
  * Sub-3ms koldstarts-garanti jf. xGauntlet-invarianterne.
  * Grafberegning: Hukommelseseffektiv grafstruktur med standard graf-algoritmer (BFS nabosøgning, DFS afhængighedskæder, shortest path og reachability).

### 3. Lagring & Transparens (Hvor gemmes grafen?)
- **Transient In-Memory Cache**: Beregnes on-the-fly via hooks uden tvungne disk-operationer for maksimal sub-3ms hastighed.
- **Persistent Audit Artifact**: Gemmes deterministisk i `.xgauntlet/topology.json` (og kan inspiceres direkte via `xgauntlet topology`). Udvikleren kan åbne filen og se alle noder, kanter og afhængigheder med 100% transparens.
- **JIT HUD Injektion**: Et komprimeret uddrag (top 5-10 noder i aktiv blast radius / scope) injiceres direkte i telemetriens prompt-kontekst via `PreInvocation` (Antigravity) og `PostToolUse` (Claude Code / Codex).

### 4. Forventet Effekt
- **Token-besparelse på 95-99%**: Fjerner behovet for "blind browsing", hvor LLM'en loader 20-30 filer (30.000-50.000 tokens) blot for at lokalisere koden.
- **Nul "Cache Drift"**: Da grafen udledes direkte af den faktiske kildekode på disken, svarer agenten aldrig ud fra forældede hallucinationer.

## 📋 Acceptance Criteria
- [ ] `crates/xgauntlet-core/src/features/topology/` etablerer domænemodeller (`TopologyGraph`, `TopologyNode`, `TopologyEdge`, `NodeType`, `EdgeType`, `BlastRadiusReport`).
- [ ] Parser-modul implementerer deterministisk ekstraktion af moduler, symbols og imports for de understøttede programmeringsstacks (Rust, TypeScript/Node, Python, Go) med 0 tokenforbrug.
- [ ] Graf-traverseringsmotor implementerer BFS (nabosøgning), DFS (afhængighedskæde/blast radius) og shortest path mellem to symboler/moduler.
- [ ] Topologien persisteres deterministisk i `.xgauntlet/topology.json` ved eksplicit eksport og opdateres ved ændringer.
- [ ] CLI subcommand `xgauntlet topology` udstiller:
  - `xgauntlet topology inspect`: Menneskevenlig ASCII visualisering af modulforbindelser.
  - `xgauntlet topology path <from> <to>`: Visning af afhængighedssti mellem to komponenter.
  - `xgauntlet topology blast-radius <target>`: Beregning af downstream komponenter der påvirkes.
  - `--json`: Maskinlæsbar JSON-eksport.
- [ ] Telemetrimotoren (`features/telemetry/`) udvides til at injicere en ultra-kompakt (10-15 linjers) topologisk slice i `SPEC / DISCOVERY` og `RED / DIAGNOSE` faserne via `PreInvocation` og `PostToolUse` hooks.
- [ ] Conformance integrationstest i `crates/xgauntlet-core/tests/topology_engine_test.rs` verificerer sub-3ms koldstart, 0 token-forbrug under scanning og korrekt blast-radius analyse.
- [ ] `cargo run -p xgauntlet-cli -- check-spec -t 021-ast-topology-and-codebase-discovery-engine` validerer med 0 fejl.
- [ ] Fuld workspace testsuite passerer (`cargo test --workspace`) uden linter-fejl (`cargo clippy --workspace --all-targets -- -D warnings`).

## 🚫 Must NOT
- Må IKKE anvende LLM-tokens eller eksterne netværkskald til at opbygge eller forespørge grafen.
- Må IKKE introducere baggrunds-dæmoner eller overtræde Zero-Daemon invarianten.
- Må IKKE forringe sub-3ms koldstarts-invarianten for hook- og telemetriafvikling.
- Må IKKE foretage remote publication handlinger (`git push`).
- Må IKKE generere ubegrænsede grafer, der sprænger hukommelsen ved store kodebaser (dybde- og node-bounding skal håndhæves).

## 📝 Revisions
- 2026-09-12: Task 021 oprettet som ACTIVE for AST Codebase Topology & Token-Optimized Discovery Engine.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 021-ast-topology-and-codebase-discovery-engine`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- topology inspect`

