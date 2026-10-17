---
type: Roadmap
title: 'xGauntlet Roadmap & Harness Strategy'
description: 'Oversigt over planlagte, evaluerede og fravalgte agent-harnesses, adapter-slices samt metodiske agent skills'
status: draft
tags:
  - roadmap
  - harnesses
  - adapters
  - architecture
  - skills
  - methodology
---

# 🗺️ xGauntlet Roadmap

Dette dokument skitserer den fremtidige integrationsstrategi og roadmap for agent-harnesses og metodiske agent-skills i **xGauntlet**. Hver understøttet harness og skill implementeres som en autonom, isoleret vertikal feature-slice jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md) og [ADR 0004: Vertical Slice Harness Adapters](docs/adr/0004-harness-adapter-slices.md).

---

## 🎯 Planlagte Harness Adapters (Kommende)

Følgende harnesses er planlagt til understøttelse i kommende versioner:

- [ ] **[Cline](https://marketplace.visualstudio.com/items?itemName=saoudrizwan.claude-dev)**  
  *Platform*: VS Code Extension  
  *Beskrivelse*: Autonom kodningsassistent i VS Code med tool-kald og CLI-eksekvering.

---

## 💡 AI-forslag til Harness-kandidater

Kandidater identificeret på baggrund af eksisterende arkitekturreferencer i repositoriet (ADR'er og opgaver), som kan evalueres til fremtidige feature-slices:

- [ ] **[OpenHands](https://github.com/All-Hands-AI/OpenHands)** (DeepSeek / OpenDevin)  
  *Reference*: Eksplicit nævnt som fremtidigt designmål i [ADR 0004](docs/adr/0004-harness-adapter-slices.md), men mangler endnu en formel adapter-specifikation og hook-mapping.

- [ ] **[Gemini CLI](https://github.com/google-gemini)**  
  *Reference*: Allerede defineret som målplatform for global plugin-distribution (`~/.gemini/config/plugins/xgauntlet`) i [Task 020](tasks/020-global-plugin-distribution-and-jit-skill-injection.md); kan formaliseres med dedikeret CLI-harness adapter.

- [ ] **[Zed](https://zed.dev/)**  
  *Reference*: Højtydende editor skrevet i Rust med egne agentic slash-commands/hooks; ideelt match til xGauntlets native Rust-motor og Zero-Daemon filosofi.

- [ ] **[Windsurf](https://codeium.com/windsurf)** (Codeium)  
  *Platform*: VS Code-baseret agentic IDE; deler adapter-karakteristika og hook-infrastruktur med Mistral Vibe og Cline.

- [ ] **[GitHub Copilot](https://github.com/features/copilot)**  
  *Platform*: VS Code Extension & CLI; udbredt økosystem, men kræver afklaring af hook-interception og integrationsmuligheder.

---

## 🧠 Foreslåede Agent Skills & Metodiske Udvidelser

xGauntlet evaluerer løbende førende open-source metodologier og skill-biblioteker for kodende AI-agenter. Formålet er at destillere de stærkeste operationelle mønstre og tilpasse dem xGauntlets ufravigelige **Evidence-First Development & Clean Craftsmanship** principper (herunder OKF v0.2 task-pakker, zero ambient authority og lokale checkpoint-invarianter).

Nedenfor grupperes de evaluerede kandidater efter deres ophavs-bibliotek forud for eventuel integration i `.agents/plugins/xgauntlet/skills/`.

---

### 📦 Addy Osmani Agent Skills (`[addyosmani/agent-skills](https://github.com/addyosmani/agent-skills/)`)

Inspireret af Addy Osmanis produktionsorienterede ingeniør-skills, der fokuserer på at dæmme op for agent-slaphed, forældede træningsvægte og bekræftelsesbias:

#### 1. 🦹 `doubt` (Doubt-Driven Adversarial Review)
- **Formål**: Modvirke bekræftelsesbias og hallucineret overmod før arkitektur eller kritiske snitflader fastfryses.
- **Mekanisme**: Ekstraherer en isoleret `ARTIFACT` og `CONTRACT` og stripper forfatterens egne rationaler. Spawner en adversarial anmelder (modstander-prompt/kold model): *"Find hvad der er galt med dette snit. Antag forfatteren er overmodig. Søg skjulte koblinger, uanfægtede antagelser og uudtalte fejltilstande."*
- **Placering i loop**: `SPEC` (før godkendelse) og ved irreversible arkitekturbeslutninger.

#### 2. 📏 `constraint` (Constraint-Driven Development & Anti-Suppression Ratchets)
- **Formål**: Forhindre agenter i i stilhed at sænke barren for at passere en test (anti-slaphed).
- **Mekanisme**: Håndhæver et ufravigeligt kvalitetsgulv:
  - Forbud mod nye undertrykkelseskommentarer (`#[allow(...)]`, `@ts-ignore`, `eslint-disable`) uden eksplicit ADR-dispensation.
  - Forbud mod stubs (`todo!()`, `throw new Error("Not implemented")`) og tomme fejlfangere.
  - Numeriske ratchets: Mutant-score og test-antal er monotont voksende og må aldrig regredere.
- **Placering i loop**: Integreret i `xgauntlet check-spec` og `verify`.

#### 3. 🧹 `simplify` (Code Simplification & Refactor Discipline)
- **Formål**: Give `REFACTOR`-fasen i TDD operationel substans, så kode gøres enklere uden at ændre observerbar adfærd.
- **Mekanisme**: Frosne assertions overholdes strengt, *Chesterton's Fence* (forstå koden før den slettes), afgrænsning strengt til ændrede linjer (forbud mod ukontrolleret drive-by refaktorering) samt prioritering af klarhed over snuhed.
- **Placering i loop**: `REFACTOR` (mellem GREEN og GAUNTLET).

#### 4. 📚 `source` (Source-Driven Development & Anti-Hallucination)
- **Formål**: Forhindre at agenter gætter crate- eller biblioteks-API'er ud fra forældede vægte.
- **Mekanisme**: 4-trins verificeret arbejdsgang: `Detect` (læs nøjagtige versioner i `Cargo.toml`/`package.json`) → `Fetch` (hent autoritativ officiel dokumentation som untrusted data) → `Implement` → `Cite` (angiv kildereferencer i koden).
- **Placering i loop**: `SPEC` og `GREEN`.

#### 5. 💬 `interview` (Én-spørgsmåls Sparring med Kvalificeret Gæt)
- **Formål**: Forbedre brugerinteraktionen i [discover](.agents/plugins/xgauntlet/skills/discover/) og afdække det reelle behov hurtigt uden lange spørgebatterier.
- **Mekanisme**: Én fokuseret afklaring ad gangen med et vedhæftet begrundet gæt (`GUESS: ...`), der tvinger agenten til at blotlægge egne antagelser og gør det kognitivt let for brugeren at korrigere retningen.
- **Placering i loop**: `DISCOVER` / `GRILL`.

---

### 📦 Obra Superpowers (`[obra/superpowers](https://github.com/obra/superpowers/)`)

Inspireret af Jesse Vincents agentiske softwaremetodologi, der er designet til at tvinge AI-agenter til at arbejde som stringente seniorudviklere med strukturel isolation og kompromisløs fejldiagnose:

#### 1. 🌿 `worktree` (Git Worktree Isolation & Submodule Guard)
- **Formål**: Beskytte udviklerens primære arbejdsmappe og aktive branch mod eksperimentelle, destruktive eller parallelle agent-ændringer.
- **Mekanisme**: Detekterer automatisk eksisterende isolation (`GIT_DIR != GIT_COMMON`) med *submodule guard* (`git rev-parse --show-superproject-working-tree`). Opretter et isoleret arbejdsområde (`git worktree add`) til feature-udvikling og implementation plans, før der skrives en eneste linje kode.
- **Placering i loop**: Prækondition før `SPEC` / `BUILD`.

#### 2. 🔬 `systematic-debugging` (The Iron Law of Root-Cause Analysis)
- **Formål**: Total udryddelse af spekulativ "symptom-lapperi" og uovervejede hurtige rettelser.
- **Mekanisme**: Håndhæver **The Iron Law**: *"NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST"*. 
  - Flerlags diagnostisk instrumentering ved systemgrænser (log præcis data-ind og data-ud for hver komponent).
  - Baglæns datastrøm-sporing til kilden af den defekte værdi.
  - Mønstersammenligning mod eksisterende velfungerende referencekode i samme kodebase.
  - Afprøvning af én enkelt falsificerbar hypotese ad gangen.
- **Placering i loop**: Berigelse af xGauntlets [diagnose skill](.agents/plugins/xgauntlet/skills/diagnose/).

#### 3. 📝 `review-focus` & Step Granularity (Task-arkitektur)
- **Formål**: Lukke de blinde vinkler i specifikationer, som glade use-case tests ofte overser.
- **Mekanisme**:
  - **Review Focus sektion**: Enhver opgaveplan skal identificere *"De fem inputklasser eller fejltilstande som specifikationen antyder, men som ingen opgavetests afprøver"*.
  - **Atomar trin-granularitet**: Hvert trin i opgaven opdeles i én isoleret handling med verificerbart resultat (*"Skriv rød test"* → *"Verificér fejl"* → *"Minimal kode"* → *"Verificér grøn"* → *"Lokal checkpoint commit"*).
- **Placering i loop**: [spec skill](.agents/plugins/xgauntlet/skills/spec/) og `tasks/`-formatering.

#### 4. 🎯 `anti-goals` (Anti-Goals & Reconnaissance i Brainstorming)
- **Formål**: Afdække operationelle fiaskokriterier og uønsket adfærd, før der udarbejdes en opgavepakke.
- **Mekanisme**: Afdækker eksplicit **Anti-Goals** (*"hvad ville gøre denne feature til en fiasko, selvom koden teknisk set kompilerer og fungerer?"*). Indfører desuden en formaliseret "Recon"-tilladelse, hvor agenten beder om lov til at undersøge kodebasen eller webkilder, før den fremlægger et arkitekturforslag.
- **Placering i loop**: [discover skill](.agents/plugins/xgauntlet/skills/discover/).

#### 5. 🏁 `finish-branch` (Deterministisk Branch Integration & Cleanup)
- **Formål**: Sikre at færdige grene eller worktrees afsluttes, verificeres og integreres deterministisk uden uafklaret affald.
- **Mekanisme**: 5-trins protokol: (1) Kør fuld verifikation (`xgauntlet verify`) → (2) Detektér git-miljø (worktree vs. normal repo) → (3) Fastlæg base branch → (4) Fremlæg fast 3-punkts integration menu (Merge lokalt med test af merged state / Opret PR / Behold gren) → (5) Sikker fjernelse af worktree.
- **Placering i loop**: [release skill](.agents/plugins/xgauntlet/skills/release/) og afslutning af opgaver.

#### 6. 🧪 `skill-tdd` (Test-Driven Skill Authoring)
- **Formål**: Anvende TDD-disciplin direkte på udvikling og raffinering af agent-skills.
- **Mekanisme**: Test agent-adfærd før og efter en skill introduceres: Bevis at en agent uden skillen afviger eller tager uheldige genveje, og bevis at den med skillens instruktioner følger den snævre sti uden undtagelse.
- **Placering i loop**: Metodisk udvikling af nye xGauntlet-skills.

---

### 📦 Matt Pocock Skills (`[mattpocock/skills](https://github.com/mattpocock/skills)`)

Inspireret af Matt Pococks ingeniør-skills, der bekæmper "vibe coding" og etablerer fælles domænesprog (Ubiquitous Language), vertikale tracer-bullet slices og to-akset anmeldelse:

#### 1. 🎯 `tracer-bullets` & Expand-Contract Pattern (`to-tickets`)
- **Formål**: Etablere stringent opgaveopdeling, hvor enhver feature skæres vertikalt, mens brede refaktoreringer håndteres uden at knække CI.
- **Mekanisme**:
  - **Tracer Bullets**: Hver opgave skærer et komplet, tyndt vertikalt snit gennem alle lag (typer, logik, CLI, tests) og kan verificeres isoleret.
  - **Expand-Contract ved Wide Refactors**: Refaktoreringer med stor *blast radius* (f.eks. omdøbning af centrale traits eller types) må *ikke* presses ind i én gigantisk feature-slice. De sekventeres i 3 faser: (1) *Expand* (tilføj ny form parallelt med den gamle uden at bryde eksisterende kode), (2) *Migrate* (migrér kaldesites i batches per modul, så CI forbliver grøn igennem hele forløbet), (3) *Contract* (slet den gamle form, når alle referencer er væk).
- **Placering i loop**: [spec skill](.agents/plugins/xgauntlet/skills/spec/) og `tasks/`-nedbrydning.

#### 2. 🏛️ `deepening` & The Deletion Test (`improve-codebase-architecture`)
- **Formål**: Bekæmpe arkitektonisk friktion og overfladiske moduler ("shallow modules") jf. John Ousterhouts designfilosofi.
- **Mekanisme**:
  - Scanning af kodebasens hotspots (`git log`) for steder, hvor et modul har en snitflade, der er næsten lige så kompleks som implementationen.
  - **The Deletion Test**: *"Vil det at slette dette modul koncentrere kompleksiteten eller blot flytte den rundt?"* Hvis sletning koncentrerer kompleksiteten, er modulet berettiget; hvis det blot flytter den, er det et overfladisk modul.
  - Forankring i naturlige test-seams ("grænsefladen er testoverfladen").
- **Placering i loop**: [review skill](.agents/plugins/xgauntlet/skills/review/) og arkitektonisk refaktorering.

#### 3. 🗺️ `wayfinder` (Navigering i Arkitektonisk Tåge via Beslutnings-DAG)
- **Formål**: Strukturere store, uoverskuelige epics (større end én chat-session kan rumme), hvor ruten til målet endnu er uklar ("fog of war").
- **Mekanisme**: Opretter et overordnet kort (*shared map*) med *beslutnings-billetter* (spørgsmål, hvis resultat er en arkitekturbeslutning, ikke kodeleverancer). Arbejder systematisk langs frontlinjen af ublokerede beslutninger, før selve byggefasen initieres.
- **Placering i loop**: Makro-planlægning forud for `spec.md` og store omlægninger.

#### 4. 📋 `to-questionnaire` (Asynkron Beslutnings-indkapsling)
- **Formål**: Forhindre at agenter gætter på forretningskritiske detaljer, når brugeren mangler viden fra en ekstern interessent.
- **Mekanisme**: Omdanner uafklarede spørgsmål til et asynkront Markdown-spørgeskema til tredjepart. Agenten interviewer alene om *"hvem skal udfylde det, og hvilke konkrete svar er nødvendige for at komme videre"*, og genererer et målrettet dokument.
- **Placering i loop**: [discover skill](.agents/plugins/xgauntlet/skills/discover/).

#### 5. 🔍 `two-axis-review` (Parallel Standard- vs. Spec-anmeldelse)
- **Formål**: Adskille teknisk håndværksmæssig renhed fra funktionel opgaveopfyldelse.
- **Mekanisme**: Kører to uafhængige subagenter parallelt:
  - **Standards-akse**: Verificerer kodestil og Martin Fowlers kode-lugte (*Code Smells* baseline fra *Refactoring* kap. 3).
  - **Spec-akse**: Verificerer punkt for punkt, om koden opfylder den oprindelige specifikation i opgaven uden scope creep.
- **Placering i loop**: [review skill](.agents/plugins/xgauntlet/skills/review/) (allerede delvist adopteret som xGauntlets kernereview).

---

### 🎯 Fase-opdelt Syntese & Prioriteringsmatrix (Samtlige 16 Kandidater)

For at gøre det let at evaluere og sammenligne de 16 identificerede kandidat-skills på tværs af de tre biblioteker, er de her systematisk grupperet efter **xGauntlets livscyklusfaser** og prioriteret efter deres umiddelbare værdi for vores **Evidence-First Development & Clean Craftsmanship** principper.

#### Prioriteringskriterier:
- 🔴 **P1 (Kritisk / Hurtig gevinst)**: Lukker velkendte agent-svagheder (hallucination, symptom-lapperi, unødvendig kompleksitet) med høj effekt og lav modstand.
- 🟡 **P2 (Høj strategisk værdi)**: Giver markant metodisk eller arkitektonisk løft, men forudsætter tilpasning af arbejdsgange eller udvidet tooling.
- 🟢 **P3 (Specialiseret / Fremtidig iteration)**: Avancerede arbejdsgange til særlige niche-situationer eller meta-udvikling.

---

#### 📊 Sammenlignende Prioriteringsmatrix (16 Kandidater)

| # | Fase i xGauntlet | Prioritet | Kandidat Skill | Kilde | Nuværende xGauntlet dækning | Merværdi & Prioriteringsbegrundelse |
| :-: | :--- | :---: | :--- | :--- | :--- | :--- |
| **1** | **DISCOVER** (Idéfase) | 🔴 **P1** | `interview` | Addy Osmani | Delvis ([discover](.agents/plugins/xgauntlet/skills/discover/)) | **Én ting ad gangen + GUESS**: Tvinger agenten til at blotlægge antagelser og eliminerer udmattende spørgebatterier. |
| **2** | **DISCOVER** (Idéfase) | 🔴 **P1** | `anti-goals` | Superpowers | Delvis (Must NOT) | **Eksplicitte fiaskokriterier**: Definerer hvad der gør en feature ubrugelig, selvom koden teknisk set kompilerer. |
| **3** | **DISCOVER** (Idéfase) | 🟡 **P2** | `to-questionnaire` | Matt Pocock | Ingen | **Asynkron afklaring**: Omdanner blokeringer til et færdigt spørgeskema til eksterne interessenter i stedet for at gætte. |
| **4** | **DISCOVER** (Idéfase) | 🟡 **P2** | `wayfinder` | Matt Pocock | Ingen | **Beslutnings-DAG i tågen**: Strukturerer store, uoverskuelige epics som en graf af arkitekturbeslutninger frem for kodeopgaver. |
| **5** | **SPEC** (Opgavepakke) | 🔴 **P1** | `tracer-bullets` & Expand-Contract | Matt Pocock | Delvis (OKF `tasks/`) | **Sikker blast-radius**: Vertikale snit til features; formel 3-fase Expand-Contract migration ved brede refaktoreringer. |
| **6** | **SPEC** (Opgavepakke) | 🔴 **P1** | `review-focus` & Trin-Granularitet | Superpowers | Delvis (Kriterier) | **De 5 blinde vinkler**: Opgaven skal opliste de 5 fejltilstande som use-case tests typisk overser, med atomare delskridt. |
| **7** | **SPEC** (Opgavepakke) | 🟡 **P2** | `doubt` | Addy Osmani | Ingen (kun post-review) | **In-flight modstander**: Spawner en adversarial anmelder af isoleret `ARTIFACT + CONTRACT` før spec godkendes. |
| **8** | **RED** (Fejlende test) | 🟡 **P2** | `worktree` isolation | Superpowers | Delvis (ADR 0003 commits) | **Fysisk isolation**: Automatisk git worktree med submodule-guard til beskyttelse af aktiv branch under agent-arbejde. |
| **9** | **GREEN** (Implementation) | 🔴 **P1** | `source` (Source-Driven) | Addy Osmani | Ingen formel regel | **Anti-hallucination**: 4-trins verifikation mod officielle docs (`Detect → Fetch → Implement → Cite`). |
| **10** | **REFACTOR** (Oprydning) | 🔴 **P1** | `simplify` | Addy Osmani | Delvis (TDD refactor) | **Operationel refaktorerings-disciplin**: *Chesterton's Fence*, afgrænsning strictly til ændrede linjer, frosne assertions. |
| **11** | **REFACTOR** (Oprydning) | 🟡 **P2** | `deepening` & Deletion Test | Matt Pocock | Delvis (Fowler smells) | **Ousterhout dybe moduler**: *The Deletion Test* fjerner overfladiske moduler, hvor snitfladen er lige så kompleks som koden. |
| **12** | **GAUNTLET** (Verifikation) | 🔴 **P1** | `constraint` (Anti-Suppression) | Addy Osmani | Delvis (`xgauntlet verify`) | **Mekanisk kvalitetsgulv**: Forbud mod nye suppressions (`#[allow]`, `@ts-ignore`) og monotont voksende test/mutant-ratchets. |
| **13** | **REVIEW** (Anmeldelse) | 🔴 **P1** | `two-axis-review` | Matt Pocock | Adopteret ([review](.agents/plugins/xgauntlet/skills/review/)) | **Parallel Standards vs. Spec**: Uafhængige subagenter uden kontekstforurening (bør udvides med HTML-rapportering). |
| **14** | **DIAGNOSE** (Fejlfinding) | 🔴 **P1** | `systematic-debugging` | Superpowers | Delvis ([diagnose](.agents/plugins/xgauntlet/skills/diagnose/)) | **The Iron Law**: *"No fixes without root cause"*; flerlags boundary-logging og baglæns datastrøm-sporing. |
| **15** | **RELEASE** (Afslutning) | 🔴 **P1** | `finish-branch` | Superpowers | Delvis ([release](.agents/plugins/xgauntlet/skills/release/)) | **Deterministisk integration**: Fast 3-punkts menu og obligatorisk testverifikation på det mergede resultat før cleanup. |
| **16** | **META** (Skill-udvikling) | 🟢 **P3** | `skill-tdd` | Superpowers | Ingen | **Metodisk skill-udvikling**: TDD på selve agent-instruktionerne (bevis at agenten fejler uden skillen og lykkes med den). |

---

#### 📋 Detaljeret Evaluering af Samtlige 16 Kandidater per Fase

##### 1. Idé- og Behovsafklaring (`DISCOVER` / `GRILL`)
1. **`interview` (Addy Osmani) — 🔴 P1**:
   - *Hvorfor*: Agenter fejler ofte ved at overvælde brugeren med lange spørgebatterier. Reglen om *"ét spørgsmål ad gangen med vedhæftet kvalificeret gæt (`GUESS:`)"* gør det lynhurtigt for brugeren at korrigere agentens kurs.
   - *Tiltag*: Integreres som obligatorisk adfærd i xGauntlets [discover skill](.agents/plugins/xgauntlet/skills/discover/).
2. **`anti-goals` (Obra Superpowers) — 🔴 P1**:
   - *Hvorfor*: Det er ofte vigtigere at vide hvad systemet *ikke* må blive til. Spørgsmålet *"Hvad ville gøre denne feature til en fiasko, selvom koden kompilerer?"* fanger uønskede bivirkninger i opløbet.
   - *Tiltag*: Integreres som fast afsnit i `discover`-interviewet forud for opgaveoprettelse.
3. **`to-questionnaire` (Matt Pocock) — 🟡 P2**:
   - *Hvorfor*: Når en arkitektonisk detalje afhænger af en ekstern interessent, gætter agenter ofte eller går i stå. Evnen til at syntetisere et asynkront Markdown-spørgeskema er et stærkt unblocking-værktøj.
   - *Tiltag*: Etableres som hjælpeskill i discover-fasen til asynkron vidensindsamling.
4. **`wayfinder` (Matt Pocock) — 🟡 P2**:
   - *Hvorfor*: Komplekse epics, der spænder over mange sessioner, drukner ofte i kontekst-forurening. Opdeling i *beslutnings-billetter* (beslutninger forud for kode) skaber klar sigtbarhed i tågen.
   - *Tiltag*: Evalueres som makro-planlægningsværktøj til fremtidige v2-epics i xGauntlet.

##### 2. Kravspecifikation & Opgavenedbrydning (`SPEC`)
5. **`tracer-bullets` & Expand-Contract (Matt Pocock) — 🔴 P1**:
   - *Hvorfor*: Horisontale opgaver (f.eks. "kun database" eller "kun CLI") efterlader halvbagt kode. Vertikale tracer bullets sikrer fuldt testbare skiver, og Expand-Contract sikrer, at store omlægninger ikke knækker CI undervejs.
   - *Tiltag*: Indarbejdes som ufravigelig regel i `tasks/`-retningslinjerne i [AGENTS.md](.agents/AGENTS.md) og [spec skill](.agents/plugins/xgauntlet/skills/spec/).
6. **`review-focus` & Trin-Granularitet (Obra Superpowers) — 🔴 P1**:
   - *Hvorfor*: Use-case tests tester kun den lykkelige vej. Kravet om eksplicit at identificere *"De fem oversete inputklasser/fejltilstande"* lukker kritiske huller. Samtidig tvinger atomar granularitet (*"Skriv rød test → bevis fejl → minimal kode → bevis grøn → lokal commit"*) agenten til disciplin.
   - *Tiltag*: Tilføjes som obligatoriske sektioner i xGauntlets OKF v0.2 opgavepakker.
7. **`doubt` (Addy Osmani) — 🟡 P2**:
   - *Hvorfor*: Forfatter-bias er reel. Ved at strippe forfatterens egne forklaringer og sende isoleret `ARTIFACT + CONTRACT` til en adversarial modstander-model fanges skjulte koblinger før implementering.
   - *Tiltag*: Etableres som valgfri adversarial gatekeeper i SPEC-fasen ved irreversible ADR-beslutninger.

##### 3. Test-Drevet Udvikling & Workspace Setup (`RED`)
8. **`worktree` isolation (Obra Superpowers) — 🟡 P2**:
   - *Hvorfor*: Fysisk adskillelse af agent-arbejdet beskytter udviklerens aktive branch mod ødelagte tilstande. Submodule-guard sikrer robust detektering.
   - *Tiltag*: Overvejes som en CLI-feature (`xgauntlet task start <id> --worktree`), der automatisk klargør et isoleret git worktree.

##### 4. Minimal Implementation (`GREEN`)
9. **`source` / Source-Driven Development (Addy Osmani) — 🔴 P1**:
   - *Hvorfor*: LLM'er hallucinerer ofte forældede API-mønstre fra ældre træningsdata. Kravet om en 4-trins kildestyring (`Detect → Fetch → Implement → Cite`) eliminerer gætteværk.
   - *Tiltag*: Tilføjes som bindende instruks i TDD-fasen for al kode, der interagerer med eksterne crates og biblioteker.

##### 5. Refaktorerings-disciplin (`REFACTOR`)
10. **`simplify` (Addy Osmani) — 🔴 P1**:
    - *Hvorfor*: REFACTOR-fasen forsømmes ofte eller udarter til ukontrolleret kode-omskrivning. *Chesterton's Fence*, afgrænsning strictly til ændrede linjer og forbud mod drive-by refactorings holder diffen kirurgisk ren.
    - *Tiltag*: Tilføjes som eksplicit tjekliste i [tdd skill](.agents/plugins/xgauntlet/skills/tdd/) under REFACTOR-trinnet.
11. **`deepening` & The Deletion Test (Matt Pocock) — 🟡 P2**:
    - *Hvorfor*: Forhindrer over-abstrahering og overfladiske moduler. *The Deletion Test* sikrer, at moduler kun eksisterer, hvis de rent faktisk indkapsler og koncentrerer kompleksitet.
    - *Tiltag*: Indarbejdes som arkitektonisk heuristik i Fowler-kataloget i [review skill](.agents/plugins/xgauntlet/skills/review/).

##### 6. Multi-Layer Verifikation & Gatekeeping (`GAUNTLET`)
12. **`constraint` & Anti-Suppression Ratchets (Addy Osmani) — 🔴 P1**:
    - *Hvorfor*: Agenter har en tendens til at tilføje `@ts-ignore` eller `#[allow(...)]` for hurtigt at passere en test. Et mekanisk kvalitetsgulv og voksende ratchets forhindrer, at barren i stilhed sænkes.
    - *Tiltag*: Indbygges mekanisk i `xgauntlet check-spec` og `verify`, så nye undertrykkelseskommentarer blokerer gauntlet-passet.

##### 7. To-akset Kodeanmeldelse (`REVIEW`)
13. **`two-axis-review` (Matt Pocock) — 🔴 P1**:
    - *Hvorfor*: Sikrer at kode både lever op til håndværksmæssige standarder (Fowler Code Smells) og funktionel specifikation, evalueret i parallelle subagenter for at undgå kontekstforurening.
    - *Tiltag*: Er allerede xGauntlets kernemodel i [review skill](.agents/plugins/xgauntlet/skills/review/); kan udvides med automatisk generering af visualiseret HTML-rapport i `/tmp`.

##### 8. Disciplineret Fejlfinding (`DIAGNOSE`)
14. **`systematic-debugging` & The Iron Law (Obra Superpowers) — 🔴 P1**:
    - *Hvorfor*: Forhindrer febrilsk symptom-lapperi. Reglen *"NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST"* sammen med flerlags boundary-logging sikrer, at årsagen altid bevises empirisk før koden ændres.
    - *Tiltag*: Integreres som ufravigelig protokol i xGauntlets [diagnose skill](.agents/plugins/xgauntlet/skills/diagnose/).

##### 9. Færdiggørelse & Overdragelse (`RELEASE`)
15. **`finish-branch` (Obra Superpowers) — 🔴 P1**:
    - *Hvorfor*: Sikrer en deterministisk afslutning på feature-arbejde med fast 3-punkts valgmenu og obligatorisk verifikation af den fusionerede tilstand, før midlertidige ressourcer ryddes op.
    - *Tiltag*: Indarbejdes som afslutningssekvens i [release skill](.agents/plugins/xgauntlet/skills/release/) og `SESSION HANDOFF`.

##### 10. Meta-Udvikling & Skill Governance (`META`)
16. **`skill-tdd` (Obra Superpowers) — 🟢 P3**:
    - *Hvorfor*: Giver en systematisk metode til at teste nye agent-skills: Bevis at en agent uden skillen fejler eller tager uønskede genveje, og bevis at den med skillens instruktioner konsekvent lykkes.
    - *Tiltag*: Anvendes internt ved udvikling og validering af nye skills til xGauntlet-platformen.

---

## ⛔ Bevidste Fravalg (Non-Goals)

Følgende værktøjer og platforme er bevidst fravalgt som officielle integrationer:

| Harness | Årsag / Begrundelse | Status |
| :--- | :--- | :--- |
| **Cursor** | Opkøb / tilknytning til Elon Musks X / xAI. xGauntlet prioriterer åbne, gennemsigtige standarder og leverandøruafhængige harnesses. | ❌ Afvist |