# Changelog

All notable changes to the `xGauntlet` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

## [0.9.0] - 2026-10-25

### 🚀 Added & Hardened
- **Multi-Session Lineage & Evidence Traceability (`Task 045`, `crates/xgauntlet-core`, `crates/xgauntlet-cli`)**:
  - Implementeret kryptografisk, append-only `SessionLineageTrail` i Two-Tier evidensmodellen (ADR 0005) med deterministisk sporing af session nonce, ingeniørrolle, fase og commit OID i `verification-report.json` og `evidence.md`.
  - Tilføjet CLI-kommandoen `xgauntlet lineage --task <id>` til lynhurtig inspektion og validering af sessionsintegritet.
- **Session Health, Turn & Token Telemetry (`Task 044`, `crates/xgauntlet-core`)**:
  - Etableret letvægts Session Health telemetri (`Healthy`, `AttentionNeeded`, `Fatigued`) baseret på turn-historik, payload-volumen og deterministisk tokenforbrug for proaktivt at modvirke context rot og attention dilution i Cockpit HUD.
- **Single-Agent Governance & Sovereign Agentic SDLC Alignment (`Task 043`, `.agents/plugins/xgauntlet`)**:
  - Afstemt guidelines, skills (`discover`, `spec`, `tdd`, `review`, `release`) og roadmap mod disciplineret Human-in-the-Loop single-agent software engineering uden subagent-dogmer.
- **ADR-Bound Architectural Exemptions & Monotonic Ratchet Governance (`Task 046`, `Task 049`, `crates/xgauntlet-core`, `gauntlet.toml`)**:
  - Håndhævet obligatorisk ADR-attribution (`docs/adr/0010-...`) på samtlige arkitekturundtagelser (`allowed_exemptions`).
  - Implementeret mekanisk fail-closed håndhævelse i `check-architecture`, der forhindrer autonom inflation af fil- og kompleksitetslofter (`ratchets.max_lines`).
- **Autonomous RED-to-GREEN TDD Transition & Test Provenance (`Task 030`, `Task 032`, `crates/xgauntlet-core`)**:
  - Fail-closed detektion af born-green tests via preflight assertions og automatisk overgang fra RED til GREEN ved bekræftet rød accepttest.
- **ADR OKF Frontmatter Validation & Composite TypeScript Detection (`Task 031`, `Task 029`, `crates/xgauntlet-core`)**:
  - Validering af OKF v0.2 frontmatter på tværs af alle ADRs og understøttelse af sammensatte TypeScript-projekter (`tsc -b`).

### 🐛 Fixed & Resilient
- **Worktree-Isolated Inverse Preflight Testing (`Task 047`, `crates/xgauntlet-core`)**:
  - Afværget datatabsrisiko under inverse preflight-testning ved at udføre baseline-fejltesten i et isoleret Git worktree i stedet for det aktive working tree.
- **Canonical Antigravity Hook Migration & Doctor Auto-Remediation (`Task 048`, `Task 026`, `crates/xgauntlet-core`)**:
  - Rettet in-migration af Antigravity `PreInvocation` hook, standardiseret canonical hook schema og fjernet tavs fejlhåndtering i `xgauntlet doctor --fix`.

## [0.8.5] - 2026-10-18

### 🚀 Added & Hardened
- **Workspace Agnosticism & Generic Root Manifest Scanning (`Task 041`, `crates/xgauntlet-core`)**:
  - Generaliseret `compute_workspace_manifest` så den, når `scopes` er `None`, scanner samtlige filer i workspace-roden rekursivt (udelukker git og build-artefakter) i stedet for at være bundet til hårdkodede xGauntlet-specifikke stier (`DEFAULT_SCOPES`). Gør det muligt for eksterne projekter (som fx Kant) at anvende xGauntlets manifest-, evidens- og driftkontrol.
- **Fail-Closed Git Checkpoint Scoping (`Task 041`, `crates/xgauntlet-core`)**:
  - Tilføjet `validate_staged_files_scope` i `CheckpointEngine::run_checkpoint` der håndhæver, at stagede filer matcher opgavens erklærede scopes (`allowed_scopes`), og fail-closer med `CheckpointError::ScopeViolation` for at forhindre utilsigtede stagede filer i lokale commits.
- **Harness Adapter PreToolUse Parity (`Task 041`, `crates/xgauntlet-core`)**:
  - Udvidet `merge_lifecycle_hook` for `ClaudeCodeAdapter` og `CodexAdapter` til også at konfigurere `PreToolUse` gatekeeper-hooks parallelt med `PostToolUse` telemetri, hvilket giver ensartet gatekeeper-beskyttelse på tværs af alle 4 understøttede harnesses (Antigravity, Claude Code, Codex, Mistral Vibe).
- **Sovereign Review Skill Definition of Enough (`Task 041`, `.agents/plugins/xgauntlet/skills/review/SKILL.md`)**:
  - Indført explicit trusselsmodel (hjælpsom, stokastisk AI-assistent vs. ekstern angriber), faste alvorsgrad-tærskler (kun reelle nedbrud og kontraktbrud) og obligatorisk `STATUS: SHIP IT` konklusion for at forhindre endeløs LLM fejlfinder-bias.

## [0.8.4] - 2026-10-17

### 🐛 Fixed & Hardened
- **Bootstrapper Attestation Fallback & Dual-Asset Release Parity (`Task 040`, `packages/cli`, `.github/workflows`, `crates/xgauntlet-cli`)**:
  - Implementeret transparent fallback i `fetchAttestationBundle` (`packages/cli/bin/xgauntlet.js`), så den automatisk henter `attestation.bundle` eller `attestation.json` uden 404-fejl under første kørsel efter global npm-installation.
  - Hærdet bootstrapperens cache-håndtering, så der ikke efterlades tomme cachemapper i `~/.cache/xgauntlet/vX.Y.Z/` ved afbrudte downloads.
  - Tilføjet dual-asset staging i `.github/workflows/release.yml`, så fremtidige GitHub Releases publicerer både `attestation.bundle` og `attestation.json`.
  - Tilføjet automatisk fallback til `attestation.json` i `xgauntlet check-attestation` CLI-kommandoen, når default bundt-sti mangler.

## [0.8.3] - 2026-10-17

### 🛡️ Security Hardening & Gatekeeper (v0.8.3 Release Package)
- **Release Gatekeeper Fail-Closed Hardening & Evidence Integrity (`Task 036`, `crates/xgauntlet-core`, `.github/workflows`)**:
  - Hærdet `xgauntlet check-release` mod tavse fejl; formatering og WASM-policy valideres nu obligatorisk fail-closed.
  - Tilføjet portabel `fail_on_output` til `LayerConfig` for platformsuafhængig `gofmt`-validering.
  - Genoprettet standardprofiler for eksterne projekter (`optional: true` for format, lint og types) for at undgå fejlagtige afbrydelser i udrullede miljøer.
  - Etableret strict evidens- og driftgate i `check-release --strict` med deterministisk udelukkelse af cirkulære artefakter (`verification-report.json`, `evidence.md`, `attestation.bundle`).
  - Implementeret transaktionel `release bump` med fuld `wasm32-unknown-unknown` build, isoleret disk-staging backup og automatisk rollback ved fejl uden brug af destruktive git-handlinger.
  - Opdelt GitHub Actions release-workflowet i en formaliseret DAG (`ci` -> `gate-check-release` -> `build-binaries` -> `create-github-release` -> `publish-npm`).
  - Hærdet Node-bootstrapperen til at returnere exit-kode 1 ved manglende native engine under `status` og `doctor`.
  - Tilføjet CLI-alias `--task-id` på `verify`, `check-spec` og `handoff`.
- **Structured Command Tokenization & Wrapper Rejection (`Task 037`, `crates/gauntlet-policy-engine`)**:
  - Erstattet naiv delstrengsmatching med fuld kommandotokenisering, quote-parsing og flag-håndtering.
  - Afvist eksekverings-wrappers (`env`, `sudo`, `eval`, `bash -c`, `sh -c`, `xargs`, `doas`, `su`, `exec`) fail-closed med grundkode `4039`.
  - Forhindret omgåelser af destruktive git-operationer (`push`, `reset --hard`, `clean -f`, `branch -D`) uanset whitespace-variationer og globale flag (`-C`, `-c`, `--no-pager`, osv.).
- **Fail-Closed Task Scope & Test Scope Isolering (`Task 037`, `crates/gauntlet-policy-engine`, `crates/xgauntlet-core`)**:
  - Obligatorisk opgavescope for alle filmodifikationer (`WriteFile`): tomme scopes afvises altid med `4037: Deny`.
  - Hvis `allowed_scopes` er tom, giver `allowed_test_targets` udelukkende skriveadgang til verificerede testfiler; modifikation af kildekode afvises altid strengt med `4037: Deny`.
- **Fail-Closed Context Loading (`Task 037`, `crates/xgauntlet-core`)**:
  - `EnforcementContext::from_workspace()` sluger ikke længere parsefejl; korrupte eller ugyldige task-filer sætter `has_active_task = false` og efterlader scopes tomme, så enhver modificerende operation efterfølgende afvises med `Deny`.
- **Google Antigravity Hook Schema Compliance & Scaffolding Resilience (`Task 038`, `crates/xgauntlet-core`)**:
  - Tilpasset `PreInvocation` konfiguration til det officielle Antigravity schema (fladt array af handlers uden `matcher`).
  - Hærdet `scaffold_hooks` mod korrupt JSON med struktureret afvisning (`ErrorKind::InvalidData`) uden dataoverskrivning.
  - Forhindret group-key hijacking i roden af `.agents/hooks.json`.
- **Cryptographic Release Attestation & Bootstrapper Integrity Verification (`Task 039`, `packages/cli`, `.github/workflows`, `crates/xgauntlet-cli`)**:
  - Implementeret ADR 0005 Tier 2 Sigstore attestation for native distributionsarkiver i CI via SHA-pinned `actions/attest`.
  - Forankret Node-bootstrapperens (`packages/cli/bin/xgauntlet.js`) integritetskontrol i en streng trust policy med forud forankret numerisk repository-ID (`1359184794`), repository og workflow validering via officiel `sigstore` npm-pakke før udpakning.
  - Udvidet CLI med `xgauntlet check-attestation` subcommand til lokal og ekstern verifikation af attesteringsbundter mod release-artefakter.

## [0.8.2] - 2026-10-16

### 🛡️ Hardened & Fixed
- **Universal Code Formatting Layer & Release Gatekeeper (`gauntlet.toml`, `crates/xgauntlet-core`)**:
  - Tilføjet obligatorisk `format` lag (`cargo fmt --check`) og `npm` verifikationslag til `gauntlet.toml`.
  - Opdateret standard stack-profiler for Rust, Python, Node/TypeScript og Go til automatisk at inkludere format-lag som ikke-valgfrit.
  - Hærdet `xgauntlet check-release` så den aktivt eksekverer kildekode-formatering og fail-closer ved syntaktiske eller linter afvigelser (`UNFORMATTED_CODE`).
  - Hærdet `--strict` mode til at afvise release-evaluering ved ucommittede ændringer (`DIRTY_WORKTREE`).
- **Synchronized WebAssembly Policy Verifier (`crates/gauntlet-policy-engine`, `crates/xgauntlet-core`, `crates/xgauntlet-cli`)**:
  - Genkompileret ren WebAssembly-binær motor (`wasm32-unknown-unknown`) synkroniseret med version 0.8.2.
  - Tilføjet automatiseret WASM-kompilering og distribution ved `xgauntlet release bump`.
  - Tilføjet Release Gatekeeper verifikation i `check-release`, der validerer at det indlejrede WebAssembly-bytecode matcher workspace-releasens version (`CONFIG_VERSION_MISMATCH`).
- **Idempotent NPM Publishing Pipeline (`.github/workflows/release.yml`)**:
  - Tilføjet registry-opslag (`npm view`) i GitHub Actions release pipeline før `npm publish`, hvilket forhindrer fatale `E403 Forbidden` fejl ved gentagne CI-kørsler over allerede udgivne versioner.
- **Task Lifecycle Status Harmony (`tasks/`)**:
  - Synkroniseret OKF v0.2 frontmatter status for historiske opgaver (002–011 til `done`, 029–032 til `todo`), så `check-spec` og `verify` kører med 100% grøn overholdelse.
- **Release Evidence & Integrity Note**:
  - Evidens- og driftkontrol er yderligere hærdet i v0.8.3 med streng forsegling af Tier 1 lokal evidens (`verification-report.json`) og transaktionel validering ved release.

## [0.8.1] - 2026-10-16
- Initial patch attempt (superseded by 0.8.2).

## [0.8.0] - 2026-10-16

### 🚀 Added
- **Declarative Architecture Ratchet Gatekeeper (`Task 034`, `crates/xgauntlet-core`)**:
  - `gauntlet.toml` schema udvidet med `[architecture]` sektion for deterministisk håndhævelse af kodestruktur.
  - Håndhæver filstørrelsesgrænser: kildekode (`max_file_lines = 800`) og tests (`max_test_file_lines = 1000`).
  - Monitored ratchet exemptions (`[[architecture.exemptions]]`) med spærrehage: filer på undtagelseslisten må kun falde i linjetal, aldrig stige.
  - Statisk forbudte tokens og imports (`forbidden_tokens`) til forebyggelse af utypede/forældede API-kald.
  - CLI subcommand: `xgauntlet check-architecture` samt integration i `xgauntlet verify`.
- **Mechanical Session Handoff & Semantic Release Engine (`Task 035`, `crates/xgauntlet-core`, `crates/xgauntlet-cli`)**:
  - Automatisk generering af deterministisk `🏁 SESSION HANDOFF` kort ved `xgauntlet checkpoint --phase done` og `xgauntlet handoff`.
  - To-niveau Git governance: Niveau A (daglig task-synkronisering med `git push origin <branch>` uden tag-forurening) og Niveau B (officiel udgivelse med SemVer tags).
  - Deterministisk SemVer 2.0.0 release manager via `xgauntlet release bump [patch|minor|major]`.
  - Manifest harmony synkronisering på tværs af Rust (`Cargo.toml`, alle crates) og npm (`package.json`, `packages/cli/package.json`).
  - Automatisk Keep a Changelog rotation og release readiness audit via `xgauntlet check-release`.
- **OKF v0.2 Task Scope & Test Reference Enforcement (`Task 033`, `crates/gauntlet-policy-engine`)**:
  - WebAssembly policy engine udvidet med `4037: Deny` gatekeeper, der forhindrer ændringer uden for den aktive opgaves erklærede `scope`-stier.
  - Validering af `test_refs` og krav om isolerede testsuiter før produktionstildelinger.
- **Sovereign Agent Skills**:
  - Etableret 6 suveræne skills: `discover`, `spec`, `tdd`, `review`, `diagnose`, `release`.

### 🛡️ Hardened & Refactored
- **CLI Codebase Modular Decomposition**:
  - Opdelt monolitisk CLI kildekode i fokuserede moduler (`render.rs` m.fl.) for streng overholdelse af 800-linjers arkitekturgrænsen.

## [0.7.3] - 2026-09-21

### 🚀 Added
- **Open Knowledge Format (OKF v0.2) Task Management Protocol (`crates/xgauntlet-core`)**:
  - Opdateret scaffold-skabelonen i `crates/xgauntlet-core/src/features/scaffold/templates.rs` og `.agents/AGENTS.md` til obligatorisk OKF v0.2 YAML frontmatter for opgavepakker i `tasks/`.
  - Tilføjet opgavespecifikationer for composite TypeScript project references detektion (`Task 029`), autonom RED-til-GREEN TDD-transition (`Task 030`) og ADR OKF v0.2 frontmatter-validering (`Task 031`).

### 📖 Documentation & Ergonomics
- **CLI Native Engine Pre-fetch (`README.md`)**:
  - Tilføjet `xgauntlet status` i installationsvejledningen til forhåndsafhentning af seneste native platform-binære motor til `~/.cache/xgauntlet/` før plugin-installation.

### 🛡️ Hardened & Synchronized
- **Wasm Policy Engine Synchronized (`crates/gauntlet-policy-engine`)**:
  - Genkompileret deterministisk WebAssembly policy verifier (`crates/xgauntlet-core/wasm/gauntlet_policy.wasm`) synkroniseret med version 0.7.3.

## [0.7.2] - 2026-09-14

### 🐛 Fixed
- **Windows PowerShell Hook Stdin BOM Immunity (`Task 028`)**:
  - Implementeret `clean_stdin` hjælpefunktion der fjerner UTF-8 Byte Order Mark (`\u{FEFF}` / `0xEF, 0xBB, 0xBF`) og normaliserer whitespace på alle harness hook inputs.
  - Opdateret samtlige 4 harness-adaptere (`ClaudeCodeAdapter`, `CodexAdapter`, `AntigravityAdapter`, `MistralAdapter`) til robust at afvise/acceptere payloads med BOM uden `serde_json` parsefejl ved linje 1 kolonne 1 (`Corrupt JSON payload on stdin`).
  - Sikret `$OutputEncoding = [System.Text.UTF8Encoding]::new($false)` i PowerShell shell-piping tests for ren binær UTF-8 proceskommunikation.

## [0.7.1] - 2026-09-14

### 🐛 Fixed
- **Windows CI Native Shell Piping Resilience (`Task 027`)**:
  - Gjort integrationstesten `test_subprocess_hook_via_native_shell_piping` robust på Windows CI-runners ved at anvende PowerShell til subprocess-piping og korrekt udbredelse af proces-exitkoder.
  - Omgået Windows `cmd.exe /C` citat-stripping (Rust issue #29494) der forårsagede syntaksfejl 255 ved piping af JSON-payloads til eksekverbare filer med mellemrum i stien.

## [0.7.0] - 2026-09-14

### 🚀 Added
- **Multi-Harness Hook Contracts & Schema Validation (`Task 027`)**:
  - Determinisk snapshot-fixtures og schema-overensstemmelsestests for samtlige 4 harness-konfigurationer (`.agents/hooks.json`, `.claude/settings.json`, `.vibe/hooks.toml`, `.codex/hooks.json`) samt deres telemetri- og gatekeeper-payloads.
  - End-to-end mock-proces livscyklustests med rigtig subprocess-spawning over native shell (`sh -c` og `cmd.exe /c`) der verificerer stdin-piping, stdout-parsing og konsistente exit-koder (0, 1, 2).
  - Wasm policy engine determinisme-tests for absolutte Windows-stier, backslashes og indlejrede CRLF-linjeskift.
  - Filsystem- og checkpoint-resilience tests under `.git/index.lock` filsystem-konflikter og afbrudte preflight-valideringer.
  - HUD og terminal rendering resilience: verificeret præcis 64-kolonners rammestruktur ved multi-byte UTF-8 og sikker omdirigering til non-TTY uden panics.

### 🛡️ Hardened & Fixed
- **Claude Code Gatekeeper Blocking Contract**:
  - Justeret `ClaudeCodeAdapter::handle_hook` til den officielle Claude Code standard: Returnerer Exit Code 2 ved afvisninger med struktureret JSON `hookSpecificOutput.permissionDecision = "deny"` og `permissionDecisionReason` på stdout, samt Exit Code 0 med `"permissionDecision": "allow"` ved godkendte værktøjskald.
- **Wasm Policy CRLF Command Injection Defense**:
  - Flyttet kontrol mod indlejrede ny-linjer (`\r` og `\n`) til før `.trim()` i `ToolActionType::ExecuteCommand`, så trailing CRLF-sekvenser i hvidlistede kommando-præfikser ikke kan omgå sikkerhedskontrollen.

## [0.6.0] - 2026-09-13

### 🚀 Added
- **Native ARM64 Linux CI Pipeline & Release Distribution (`Task 025`)**:
  - Tilføjet `ubuntu-24.04-arm` runner til GitHub Actions CI-matrix i `.github/workflows/ci.yml` for native ARM64 Linux testkørsler.
  - Tilføjet `aarch64-unknown-linux-gnu` target og `ubuntu-24.04-arm` runner til `.github/workflows/release.yml` for automatisk distribution af `xgauntlet-linux-arm64.tar.gz` på GitHub Releases.
  - Eksporteret og enhedstestet deterministisk platform- og arkitekturopløsning i Node.js launcheren (`packages/cli/bin/xgauntlet.js` og `packages/cli/test/launcher.test.js`).

## [0.5.2] - 2026-09-13

### 🐛 Fixed
- **Policy Engine Embedded WASM Synchronization**:
  - Recompiled deterministic WebAssembly policy verifier component (`crates/xgauntlet-core/wasm/gauntlet_policy.wasm`) from `crates/gauntlet-policy-engine` to match declared workspace version.
  - Resolves unit test assertion failure in `test_policy_version_matches_crate_version` on multi-platform CI pipelines.

## [0.5.1] - 2026-09-13

### 🐛 Fixed
- **Windows CI Line Endings & Frontmatter Detection (`Task 024`)**:
  - Gjort YAML frontmatter-validering i `plugin_distribution_test.rs` robust overfor både Unix LF (`\n`) og Windows CRLF (`\r\n`).
  - Tilføjet explicit regressionstest `test_crlf_frontmatter_detection`.
  - Konfigureret rod `.gitattributes` med `* text=auto eol=lf` og binære filtyper for at sikre ensartede LF-linjeskift på tværs af platforme og forhindre Git i at omskrive line endings på Windows CI.

## [0.5.0] - 2026-09-13

### 🚀 Added
- **AST Codebase Topology & Discovery Engine (`Task 021`)**:
  - Polyglot, zero-token AST og modul-ekstraktionsmotor i `crates/xgauntlet-core` for Rust, TypeScript/Node, Python og Go.
  - In-memory graf-traversering med BFS nabosøgning, DFS blast-radius konsekvensanalyse, shortest path beregninger og ASCII-trævisualisering.
  - Deterministisk audit-artefakt i `.xgauntlet/topology.json`.
  - CLI subcommand `xgauntlet topology` (`inspect`, `path`, `blast-radius`, `--json`).
  - Sub-3ms koldstarts-garanti med Cockpit HUD `Scope` injektion inden for Pococks <45-token budget.
- **Mistral Vibe Harness Adapter & Hooks Integration (`Task 022`)**:
  - Autonom vertikal feature-slice i `crates/xgauntlet-core/src/features/adapters/mistral/` for Mistral Vibe.
  - Normalisering af Mistral Vibe tool calls (`bash`, `write_file`, `edit`, `read`, `grep`, `task`, `ask_user_question`).
  - Fail-closed `pre_tool` gatekeeper hook der returnerer `{"decision": "allow"}` eller `{"decision": "deny", "reason": "..."}`.
  - Post-tool telemetri-injektion der udstiller `hook_specific_output.additional_context`.
  - Scaffolding og ikke-destruktiv merging for `.vibe/hooks.toml`.
  - CLI-understøttelse af `--harness mistral` i `hook`, `init`, `doctor` og `telemetry`.
- **7-Step Pipeline JIT Governance & Global Plugin Distribution (`Task 023`)**:
  - Global cross-platform plugin-installer (`xgauntlet plugin install`) der distribuerer den komplette 11-skill suite til Antigravity, Claude Code, OpenAI Codex og Mistral Vibe.
  - Batteries-included indlejring direkte i Rust-binæren via `include_str!`.
  - 7 specialiserede JIT fasedirektiver med distinkte AI-personaer og token-effektive nudges (<45 tokens).
  - Cross-platform harness discovery for Linux, macOS og Windows.
  - Afkobling af forretnings-ADR'er med neutral skabelon (`docs/adr/template.md`).
  - Udvidet `xgauntlet doctor` med dedikeret `Harnesses` diagnostisk kategori.

### 🛡️ Hardened & Refactored
- **Brand-Harmonisering & CLI Parameter-Fleksibilitet**:
  - Komplet kodebase-dækkende harmonisering fra det historiske `agent-gauntlet` navn til `xgauntlet`.
  - Dobbelt syntaks-understøttelse i `xgauntlet hook`: accepterer både `--harness <HARNESS>` og positionelt `[HARNESS]`.
  - Automatisk udvælgelse af hook-format via `--harness <HARNESS>` i `xgauntlet telemetry`.
  - Fuldt formateret testsuite og 100% compliance med `cargo fmt --check`.

## [0.4.0] - 2026-09-09

### 🚀 Added
- **Multi-Harness Dynamic HUD Adapters (`Task 016`, `Task 017`, `Task 018`)**:
  - **Claude Code**: `PostToolUse` event-interception der udskriver live Variant B Unicode box-drawing telemetri-kort efter tool-kald.
  - **OpenAI Codex**: `PostToolUse` integration og response-wrapping (`wrap_response`) med adaptiv terminal/agent-telemetri.
  - **Google Antigravity**: `PreInvocation` hook-interceptor via `xgauntlet hook antigravity`, der injicerer dynamisk Blockquote HUD forud for agent-prompts.
  - CLI subcommand flags: Tilføjet `--harness <HARNESS>` til `verify` og `checkpoint` med automatisk telemetri-indkapsling.

### 🛡️ Hardened & Refactored
- **HUD & Harness Adapter Hardening & OS Resilience (`Task 019`)**:
  - Garanteret fast 64-tegns linjebredde i `render_box_card` via deterministisk felt-budgettering og `truncate_with_ellipsis`.
  - Sti-normalisering af Windows backslashes (`\`) til standard POSIX (`/`) i telemetri og task-referencer.
  - DRY konsolidering af fælles `PostToolUse` JSON hook merge-logik mellem Claude Code og Codex.
  - Typesikker `HarnessKind` med understøttelse af gængse aliaser (`claude`, `claude_code`, `codex`, `openai`, `openai_codex`, `antigravity`, `google_antigravity`).
  - Robust fejlhåndtering (`?`) og registrering af scaffoldede filer under `xgauntlet init`.
  - Automatisk opgradering af forældede python-hooks i `.agents/hooks.json` til `xgauntlet hook antigravity`.

## [0.3.0] - 2026-09-07

### 📚 Documentation & Universal Distribution
- **Universal Global Installation Model**:
  - Streamlined global CLI installation (`npm install -g xgauntlet`) as the single canonical standard across all tech stacks (Python, Go, Rust, TypeScript).
  - Clarified sub-3ms PreToolUse gatekeeper hook execution latency in `$PATH` without Node.js startup overhead.
- **Poly-Harness Sameksistens & Scaffolding Scope**:
  - Documented in `README.md` and `packages/cli/README.md` the clear boundary between the 7 universal in-repo governance files and the 2 harness-specific agent configs (`CLAUDE.md`, `.agents/`).
- **Surgical In-Repo Cleanup & Zero Lock-in**:
  - Added dedicated de-installation guide distinguishing between surgical in-repo governance cleanup (`rm -f gauntlet.toml .agents/hooks.json ...`) without touching developer tasks or ADRs, and global CLI uninstallation (`npm uninstall -g xgauntlet`).

## [0.2.0] - 2026-09-07

### 🚀 Added
- **Dynamic Cockpit HUD & TDD Checkpoint Protocol (`Task 012`)**:
  - Live 5-line status card HUD for agent responses with phase, criteria progress, git telemetry, and quick-links.
  - Strict local git commit protocol on TDD phase transitions (`SPEC`, `RED`, `GREEN`, `REFACTOR`, `DONE`) adhering to ADR 0003.
- **Task Lifecycle & Intent Scaffolding (`Task 013`)**:
  - CLI subcommand `xgauntlet task init <title>` with automatic slugification, number allocation, and OKF frontmatter template.
  - CLI subcommand `xgauntlet task list` displaying active/done status, intent, criteria progress, and metadata.
- **Phase Bound TDD Checkpoint Engine (`Task 014`)**:
  - CLI subcommand `xgauntlet checkpoint <phase>` enforcing phase transitions, clean working tree, test state verification, and atomic git commit creation.

### 🛡️ Hardened & Security
- **P0 Security & Policy Boundary Hardening (`Task 015`)**:
  - Integrated `WasmPolicyEngine` directly into harness adapters (`HarnessAdapter::evaluate_invocation`) executing in-memory Wasmtime evaluation with fail-closed semantics.
  - Path traversal and workspace containment defense via `WorkspaceRelativePath` blocking escape attempts across all adapters.
  - Command chaining defense (`&&`, `;`, `|`) blocking privilege escalation behind whitelisted command prefixes.
  - Fail-closed task binding eliminating synthetic default-task fallbacks in the gauntlet pipeline.
  - Typesafe serde deserialization inside the WebAssembly policy engine security boundary.
  - Expanded anti-tamper manifest scopes (`.agents/`, `.github/`, `CONTEXT.md`, `CODING_STANDARDS.md`) and exact digest matching.

## [0.1.0] - 2026-09-06

### 🚀 Added
- **Zero-Daemon WASM Policy Verifier (`Task 002`)**:
  - Embedded zero-ambient-authority WebAssembly policy engine (`crates/gauntlet-policy-engine`) evaluated via in-memory Wasmtime host (`WasmRuntimeHost`).
  - Strict denials for destructive commands (`git push`, `rm -rf`, `git reset --hard`) and task-bound write authorizations.
- **Specification, Task & Aristotelian Glossary Gatekeeper (`Task 003`)**:
  - Shift-left CLI subcommand `check-spec` validating Open Knowledge Format (OKF) task packages and glossary invariants.
- **Evidence & Workspace Manifest Engine (`Task 004`)**:
  - Deterministic Canonical Workspace Manifest with Git-blob SHA-256 multi-digests and CRLF/LF line-ending immunity.
  - Two-Tier Trust Model (`LOCAL_UNATTESTED` vs `CI_ATTESTED`) and CLI subcommand `check-evidence`.
- **Gauntlet Execution Engine (`Task 005`)**:
  - Multi-layer fail-closed verification pipeline (`xgauntlet verify`) with timeouts and self-mutation invariant enforcement.
- **Vertical Slice Harness Adapters (`Task 006`)**:
  - Universal Pre-Tool Use interceptors and plugin validators for Google Antigravity, Claude Code, and OpenAI Codex.
- **Declarative Configuration & Multi-Stack Profiles (`Task 007`)**:
  - Schema validation for `gauntlet.toml` with presets for Rust, TypeScript/Node.js, Python, and Go via `check-config`.
- **Safe Non-Destructive Project Bootstrap Engine (`Task 008`)**:
  - Safe scaffolding subcommand `xgauntlet init` generating in-repo governance files without overwriting existing assets unless `--force` is given.
- **Fast Environment, Git & Toolchain Diagnostics (`Task 009`)**:
  - High-speed subcommand `xgauntlet doctor` performing sub-50ms host, git, governance, toolchain, and engine inspections.
- **Release Readiness Gatekeeper & Manifest Harmony Engine (`Task 010`)**:
  - Mechanical release gate `xgauntlet check-release` validating version harmony across manifests (`Cargo.toml`, `package.json`, `pyproject.toml`), `CHANGELOG.md` synchronization, and ADR coverage across `README.md` and `spec.md`.

### 🛡️ Hardened & Refactored
- **Code Review & Architecture Audit Remediation (`Task 011`)**:
  - Eliminated `.unwrap()` calls in production code across evidence manifest and release engine.
  - Consolidated ISO 8601 UTC timestamp generation into `features/evidence/time.rs`.
  - Fixed rustdoc intra-doc link warnings in `features/release/models.rs`.
  - Removed deprecated daemon documentation (`docs/supervisor-systemd.md`) aligning strictly with Zero-Daemon architecture (ADR 0007).
