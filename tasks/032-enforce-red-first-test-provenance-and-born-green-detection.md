---
type: Task Package
title: "Task 032: Enforce RED-First Test Provenance & Born-Green Detection"
description: "Håndhæve test-proveniens og forhindre 'født grønne' tests ved at detektere tests tilføjet under eller efter GREEN fasen uden forudgående bevis for rød fejl"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-06T22:30:00Z" }
tags: [task-lifecycle, tdd, checkpoint, test-provenance, born-green, epistemic-integrity, gatekeeper, preflight, adr-0003]
scope: ["crates/xgauntlet-core/src/features/checkpoint/**", "crates/xgauntlet-core/src/features/config/**", "crates/xgauntlet-cli/**", "evidence.md", "verification-report.json"]
test_refs: ["crates/xgauntlet-core/tests/provenance_engine_test.rs"]
---

# Task 032: Enforce RED-First Test Provenance & Born-Green Detection

**Status**: `TODO`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-06`  

## 🎯 Formål & Løsningsdesign
Formålet med denne opgave er at eliminere den epistemiske sårbarhed kendt som **"Født Grønne Tests" (Born-Green Tests)** i xGauntlets TDD-checkpointmotor og verifikationspipeline.

I Evidence-First Development og stringent Test-Driven Development (TDD) er en test ikke blot en samling assertions, der producerer et grønt flueben i CI. En test er et **epistemisk bevis for en observerbar adfærd**. Værdien af dette bevis hviler fundamentalt på testens **negative kapabilitet (negative capability)**: det empiriske bevis for, at testen *faktisk fejler*, når den ønskede adfærd mangler eller er defekt i kildekoden.

Når en test skrives *efter* eller *samtidig med* at produktionskoden implementeres, passerer den øjeblikkeligt første gang, den eksekveres. Den er dermed "født grøn" uden nogensinde at have påvist sin evne til at afvise et defekt system.

---

### Problemstillingen: Hvad sker der i praksis?
Under en typisk TDD-udviklingscyklus forløber processen ideelt set:
`SPEC ➔ RED ➔ GREEN ➔ REFACTOR ➔ GAUNTLET`

1. **RED-fasen**: Udvikleren eller AI-agenten skriver tests for de specificerede acceptkriterier (`AC1`, `AC2`, `AC3`). Preflight-checket (`xgauntlet checkpoint --phase red`) bekræfter, at testene fejler (`exit code != 0`), og forsegler et atomisk lokalt checkpoint:
   `test(042): add failing acceptance tests for ac1-ac3 [RED]`
2. **GREEN-fasen**: Udvikleren/agenten implementerer den minimale produktionskode for at få testene til at passere.
3. **Anomalien opstår under GREEN**: Under arbejdet med implementationen opdager udvikleren eller agenten et overset acceptkriterie, en manglende kant-case eller en overset forgrening. I stedet for at stoppe op og køre en isoleret RED-cyklus, fristes udvikleren/agenten til at skrive den nye test *direkte ind i testfilen, mens produktionskoden allerede er til stede*.
4. **Den falske tryghed**: Testsuiten afvikles. Alle tests (både de oprindelige 3 og den nye 4. test) passerer grønt. Checkpointet forsegles:
   `feat(042): implement logic for ac1-ac4 [GREEN]`

---

### Epistemisk Risikoanalyse: Hvorfor er 'Født Grønne Tests' farlige?
Uden adgang til det specifikke projekt, hvor observationen oprindeligt fandt sted, kan faren illustreres via generelle softwaremønstre (f.eks. en generisk gestus-genkender, en tilstandsmaskine eller en transaktionsbehandler):

Antag et krav: *"En dobbeltklik-hændelse må kun udsendes, hvis to på hinanden følgende klik registreres på samme koordinat inden for 250 ms uden markørbevægelse."*

Hvis testen `test_stationary_double_click_emits_event()` tilføjes, mens produktionskoden allerede er skrevet, og passerer med det samme:
1. **Tautologiske assertions & Mock-kollision**: Hvad hvis testen uforvarende tester en default mock, der altid returnerer `DoubleClicked`, eller assert-betingelsen er trivielt sand (`assert_eq!(emitted, emitted)`)? Fordi testen aldrig kørte mod den oprindelige kodebase, hvor `DoubleClicked` *ikke* fandtes, opdages fejlen i testen aldrig.
2. **Tavs ikke-eksekvering & Forgreningsfejl**: Testen kan passere fordi den rammer en early-return eller en urelateret code path, der tilfældigvis tilfredsstiller assertionen, uden reelt at aktivere den nye tids- eller koordinat-validering.
3. **Utestet diagnostisk kvalitet**: En accepttest fungerer som fremtidig regressionstest. Hvis testen aldrig har fejlet i praksis, kender vi ikke dens fejlmeddelelse. Ofte opdages det først måneder senere i CI, at assertion-beskeden er ubrugelig eller vildledende.
4. **AI Agent Sycophancy & Hallucinated Safety**: Særligt LLM-baserede agenter har en stærk bias mod at skrive tests, der spejler og bekræfter den kode, modellen netop selv har genereret. Hvis agenten tillades at introducere nye tests i GREEN-fasen, brydes den modstridende kontrol (adversarial testing), og agenten kan producere cirkulære tests, der giver et falsk indtryk af 100% kriteriedækning.

---

### Nuværende Arkitektonisk Svaghed i xGauntlet
I den nuværende implementation af `xgauntlet-core`:
- `CheckpointPhase::Red` verificerer, at testlaget *fejler* (`!res.passed`).
- `CheckpointPhase::Green` verificerer, at testlaget *passerer* (`res.passed`).
- **Manglende Invariant**: `CheckpointPhase::Green` undersøger **IKKE** Git-diffen mellem det forudgående `[RED]` checkpoint og det nuværende arbejdsindeks for nyoprettede testfunktioner!
- Hvis en ny `#[test]`-funktion (eller tilsvarende i andre sprog) optræder i et `feat(...) [GREEN]` commit uden at have eksisteret i det forudgående `test(...) [RED]` commit, tillader xGauntlet committet uden advarsel eller afvisning.

---

### Foreslået Løsningsdesign: To-Sporet Håndhævelse

Vi indfører to komplementære mekanismer i xGauntlet for at lukke dette hul:

#### Spor 1: Test-Proveniens Git-Diff Gate (`detect_born_green_tests`)
Ved `xgauntlet checkpoint --phase green` eller `xgauntlet checkpoint --phase refactor`:
1. Find det seneste lokale RED checkpoint commit i opgavens historik (eller `HEAD`, hvis RED blev committed lige før).
2. Beregn diffen af testfiler mellem RED-committet og arbejdsgrenen.
3. Parse test-definitioner via AST eller deterministiske mønstre pr. sprogstak (f.eks. `#[test]`, `test(...)`, `it(...)`, `def test_`).
4. Hvis nye test-definitioner findes i GREEN/REFACTOR-fasen, som ikke fandtes i RED-fasen:
   - Hvis `--strict-provenance` er aktiv: Afvis checkpointet med fejlen:
     `CheckpointError::BornGreenTestDetected { tests: Vec<String> }`
     og instruer agenten/udvikleren i enten at isolere testen i et nyt RED-checkpoint eller anvende retroaktiv validering.

#### Spor 2: Mekanisk "Inverse Preflight" (Tidsrejse-Verifikation)
For at undgå unødig friktion, hvis en udvikler eller agent reelt opdager et manglende scenarie undervejs:
1. I stedet for at tvinge udvikleren til manuelt at fjerne al produktionskode, tilbyder xGauntlet en automatisk **Inverse Preflight** (`--verify-provenance` / automatisk preflight fallback):
   - xGauntlet isolerer ændringerne: Den nye test bevares i testfilen, mens ændringerne i produktionskode (`src/`, `crates/`, osv.) midlertidigt udelades/gemmes i hukommelsen (non-destructive baseline tree check).
   - Testsuiten eksekveres mod baseline-koden (produktionskoden før GREEN-ændringerne).
   - **Invarianskrav**: Den nye test **SKAL FEJLE** (`exit code != 0`). Hvis den passerer mod baseline, er den beviseligt tautologisk eller irrelevant for den nye kode, og afvises straks med `TautologicalTestDetected`.
   - Produktionskoden genanvendes, og testsuiten eksekveres igen.
   - **Invarianskrav**: Testen **SKAL NU PASSÉRE** (`exit code == 0`).
   - Fejl-outputtet fra baseline-kørslen gemmes som testens retroaktive proveniensbevis i `verification-report.json`.

---

## 📋 Acceptance Criteria
- [ ] **AC1 (Test Identifier Extraction Engine)**:
  - Oprette modulet `crates/xgauntlet-core/src/features/checkpoint/provenance.rs`.
  - Implementere `extract_test_identifiers(file_path: &Path, content: &str) -> Vec<String>` der udtrækker test-navne baseret på stakkens konfiguration (Rust `#[test]` / `#[tokio::test]`, TypeScript/JavaScript `test(` / `it(`, Python `def test_`, Go `func Test`).
- [ ] **AC2 (Git Diff Provenance Inspector)**:
  - Implementere `detect_born_green_tests(workspace: &Path, base_commit_oid: &str) -> Result<Vec<String>, CheckpointError>`:
    - Sammenligner test-definitioner mellem `base_commit_oid` og det nuværende arbejdsindeks.
    - Returnerer listen af nyopståede test-identifikatorer, som ikke eksisterede i base commit.
- [ ] **AC3 (Inverse Preflight Engine)**:
  - Implementere `verify_inverse_test_failure(workspace: &Path, test_identifiers: &[String]) -> Result<TestProvenanceEvidence, CheckpointError>`:
    - Opretter en midlertidig, ikke-destruktiv baseline ved at deaktivere produktionstræets diff (f.eks. via Git tree isolation eller in-memory patch reversal).
    - Eksekverer det relevante testlag mod baseline for at bekræfte, at de angivne tests fejler.
    - Validerer at testen fejler med en legitim assertion- eller syntaksfejl relateret til den manglende adfærd.
    - Genopretter produktionstræet og verificerer, at testene passerer grønt.
    - Returnerer `TestProvenanceEvidence` med baseline stderr/stdout som bevis.
- [ ] **AC4 (Checkpoint Preflight Integration i `preflight.rs`)**:
  - Opdatere `run_preflight_check` for `CheckpointPhase::Green` og `CheckpointPhase::Refactor`:
    - Finder det forudgående RED checkpoint commit OID.
    - Kører proveniens-inspektion på arbejdsgrenen.
    - Hvis born-green tests detekteres: Kører automatisk `verify_inverse_test_failure` (eller fejler hårdt hvis strict mode kræver forudgående RED commit).
    - Afviser checkpointet med `CheckpointError::TautologicalTestDetected`, hvis testen passerede mod baseline uden produktionsændringer.
- [ ] **AC5 (Deklarativ Konfiguration & CLI Styring)**:
  - Udvide `GauntletConfig` i `features/config/models.rs` med `checkpoint.enforce_provenance: bool` (default: `true`).
  - Udvide `CheckpointOptions` med:
    - `strict_provenance: bool` (kræver at tests SKAL være committet i RED; ingen in-flight tilføjelser).
    - `allow_born_green: bool` (nød-tilsidesættelse til legacy kodebaser).
  - Udvide `xgauntlet checkpoint` CLI subkommandoen med flagene `--strict-provenance` og `--allow-born-green`.
- [ ] **AC6 (Evidens & Manifest Attestation)**:
  - Udvide `verification-report.json` og `manifest.rs` med sektionen `test_provenance`:
    - Lister alle verificerede tests med deres proveniensstatus (`RedCheckpointVerified` vs `InversePreflightVerified`).
    - Inkluderer baseline failure digest for enhver test verificeret via inverse preflight.
- [ ] **AC7 (Retningslinjer & AI Harness Vejledning)**:
  - Opdatere template for `.agents/AGENTS.md` og suveræn `tdd` skill med explicit vejledning:
    - *Regel*: "Nye tests må ALDRIG fødes grønne. Hvis et nyt kriterie eller kant-case opdages under GREEN, skal testen bevises rød via en ny RED-cyklus eller verificeres via xGauntlets inverse preflight."
- [ ] **AC8 (Testsuite & Verifikation)**:
  - Tilføje tests i `crates/xgauntlet-core/tests/provenance_engine_test.rs`, der verificerer:
    - Detektering af ny test i diff mellem commits.
    - Afvisning af en tautologisk test, der passerer både før og efter produktionskode.
    - Godkendelse af en test, der fejler før og passerer efter produktionstræet.
  - Verificere at `cargo run --manifest-path Cargo.toml -p xgauntlet-cli -- check-spec -t 032-enforce-red-first-test-provenance-and-born-green-detection -w .` validerer med 0 fejl i xGauntlet.
  - 100% grøn testsuite (`cargo test --workspace`).

---

## 🚫 Must NOT
- Må IKKE udføre destruktive Git-handlinger (`git reset --hard`, `git clean -f` eller overskrivning af ucommittede filer) under inverse preflight verifikation; kildekode og arbejdsindeks skal beskyttes 100% mod datatab.
- Må IKKE tillade born-green tests at passere ubemærket ind i et `[GREEN]` checkpoint uden verificeret negativt fejlbevis, når `enforce_provenance` er slået til.
- Må IKKE foretage remote publication handlinger (`git push`) jf. ADR 0003.
- Må IKKE introducere baggrundsdæmoner jf. Zero-Daemon invarianten.
- Må IKKE tilføje unødig kørselsoverhead til projekter eller opgaver, hvor ingen nye tests er tilføjet under GREEN (diff = 0 tests ➔ fortsæt øjeblikkeligt uden inverse preflight).

---

## 📝 Revisions
- 2026-10-06: Task oprettet som ACTIVE for håndhævelse af RED-first test-proveniens og eliminering af 'født grønne' tests.
- 2026-10-25: Moderniseret med OKF v0.2 scope og test_refs, samt tilpasset xGauntlets suveræne tdd skill.

---

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test provenance_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run --manifest-path Cargo.toml -p xgauntlet-cli -- check-spec -t 032-enforce-red-first-test-provenance-and-born-green-detection -w .`
