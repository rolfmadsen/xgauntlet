---
type: Architectural Decision Record
title: 'ADR 0014: Worktree-Isolated Multi-Agent & Subagent Supervision'
status: accepted
tags:
- architecture
- adr
- multi-agent
- worktree
- adapters
- policy
- telemetry
generated:
  by: antigravity/1.0
  at: '2026-11-14T22:05:00Z'
verified: []
---

# 14. Worktree-Isolated Multi-Agent & Subagent Supervision

**Status**: `accepted` (Extends [ADR 0003](0003-surgical-gatekeeper-and-no-remote-push.md), [ADR 0006](0006-multi-harness-policy-adapter-contract.md), [ADR 0007](0007-local-transparent-supervisor-and-wasm-verifier.md), and [ADR 0008](0008-sovereign-agentic-sdlc-task-scoping-and-design-tokens.md))  
**Date**: `2026-11-14`  

## Context
Moderne AI-harnesses — herunder Google Antigravity v2.32.1 (`invoke_subagent`, `swarm`, `send_message`, `define_subagent`, `manage_subagents`, `run_workflow`), Claude Code (`Task`/`Agent` med `isolation: "worktree"`), OpenAI Codex og Mistral Vibe — understøtter i stigende grad parallelle sessioner og delegerede subagenter.

Under xGauntlets nuværende single-checkout design opstår der imidlertid fire strukturelle blokeringer og blinde vinkler på tværs af harnesses:

1. **Fail-Closed Blokering af Orkestreringsværktøjer (`4038`)**:
   - `AntigravityAdapter::normalize_tool_call` anvender `matcher: "*"` i `.agents/hooks.json`, så ethvert subagent-/swarm-kald rammer `PreToolUse` og falder igennem til `ToolActionType::Other` (`reason_code: 4038`).
   - `ClaudeCodeAdapter`, `CodexAdapter` og `MistralAdapter` har snævrere default-matchers (`Bash|Edit|Write`), hvorved subagent-kald i dag går uden om `PreToolUse`, men ville fejle med `4038` ved bredere matchers.
2. **Enkelt Aktiv Task per Checkout (`4037`)**:
   - `resolve_active_task_path` vælger udelukkende den højeste nummererede aktive task i `tasks/**`. To parallelle sessioner på hver sin opgave (f.eks. `088` og `089`) kolliderer på den højeste task og blokeres uden for dennes `scope`.
3. **Manglende Worktree-CWD Opløsning i Øvrige Adapters**:
   - Kun `AntigravityAdapter::resolve_effective_workspace` inspicerer `workspacePaths` / `cwd` fra hook-payloaden. Når `claude_code`, `codex` eller `mistral` kører et hook fra parent-repoets CWD for en subagent i et `git worktree`, evalueres `EnforcementContext` mod det forkerte bibliotek.
4. **Delt Singleton-Telemetri på Tværs af Sessioner**:
   - Antigravitys `PreInvocation`-dedup-cache (`/tmp/xgauntlet_telemetry_dedup_<USER>.json`) deles globalt pr. OS-bruger på tværs af alle workspaces og sessioner, og `.xgauntlet/session_state.json` har ingen `session_id`-adskillelse.

Samtidig ville det bryde xGauntlets fundamentale garantier i [ADR 0003](0003-surgical-gatekeeper-and-no-remote-push.md) og [ADR 0007](0007-local-transparent-supervisor-and-wasm-verifier.md) at tillade flere agenter at skrive samtidigt i *samme* working tree:
- **Self-Mutation Invariant** (`xgauntlet verify`) fejler øjeblikkeligt som tampering, hvis Agent B ændrer filer under Agent A's testkørsel.
- **Atomiske Phase Checkpoints** (`RED` → `GREEN` → `REFACTOR`) kan ikke adskille sammenflettede ændringer i ét fælles working tree.
- **Zero-Daemon Engine** forbyder baggrundsdæmoner og fildelingslåse.

## Decision
Vi indfører en **Worktree-Isoleret Multi-Agent & Subagent Arkitektur** baseret på fem ufravigelige principper, der gælder ensartet på tværs af alle fire understøttede harnesses (`antigravity`, `claude_code`, `codex`, `mistral`):

### 1. Single-Writer-Per-Worktree Invariant
- **Parallele skrivende agenter/sessioner SKAL arbejde i hver sin isolerede `git worktree`** (eller klonede working copy). Samtidig skrivning fra flere agenter i samme working tree forbliver eksplicit forbudt.
- **Læse-kun subagenter (`research` / `explore` / read-only `review`)** må gerne dele parent-workspacet (`inherit`), da læseoperationer ikke muterer filsystemet eller bryder Self-Mutation Invarianten.
- **Muterende delegation** (`swarm` eller subagenter med skriveadgang) skal anvende isoleret worktree (`share`/`branch`) og må ikke dele skrivning i `inherit`-workspacet, når en aktiv session allerede ejer det.

### 2. Kanonisk Klassifikation af Agent Delegation på Tværs af Harnesses
- Vi udvider adapter- og policy-kontrakten (`ToolActionType` / `CapabilityRequest`) til eksplicit at repræsentere agent-orkestrering i to klare kategorier:
  - **Read-Only Delegation** (f.eks. læsende `research`-subagent, status/listning via `manage_subagents`, statusforespørgsler): Tillades på linje med ikke-muterende inspektion — også i `SPEC`-fasen uden aktiv task.
  - **Mutating / Isolated Worker Delegation** (f.eks. `swarm`, skrivende subagent i `share`/`branch`-worktree, `run_workflow`): Kræver en aktiv task (eller eksplicit koordinatorkontekst) og afvises fail-closed, hvis der forsøges samtidig skrivning i et uisoleret (`inherit`) working tree.
- Alle fire adapters (`antigravity`, `claude_code`, `codex`, `mistral`) normaliserer deres respektive subagent-/orkestreringsværktøjer gennem denne fælles kontrakt uden at svække [ADR 0006](0006-multi-harness-policy-adapter-contract.md) (caller-payloads kan aldrig overskrive `EnforcementContext`).

### 3. Worktree-Lokal Task-Binding (`.xgauntlet/active_task`)
- Hver `git worktree` (og hoved-checkout) kan binde sig lokalt til en konkret aktiv opgave via `.xgauntlet/active_task` (gitignoreret runtime-binding, styret via CLI som `xgauntlet task bind <id>`).
- `resolve_active_task_path` / `EnforcementContext::from_workspace` prioriterer:
  1. Worktree-lokal `.xgauntlet/active_task` (hvis den peger på en gyldig `ACTIVE` task i workspacet).
  2. Fallback til nuværende adfærd (højeste nummererede `ACTIVE` task i `tasks/**`).
- Herved kan Worktree A (`088`) og Worktree B (`089`) superviseres og verificeres uafhængigt på samme tid, selv om begge task-filer ligger på base-branchen.

### 4. Fælles Payload-Bevidst Worktree-Opløsning i Alle Adapters
- `resolve_effective_workspace(workspace, payload)` løftes fra at være Antigravity-specifik til at være en fælles adapter-primitiv under `adapters/`, så også `claude_code`, `codex` og `mistral` sikkert kan opløse en valid `git worktree`-sti fra hook-payloadens `cwd` / `workspacePaths`, når processens CWD peger på parent-repoet.
- Opløsningen forbliver strengt underlagt `is_valid_workspace` og stivalidering jf. [ADR 0006](0006-multi-harness-policy-adapter-contract.md).

### 5. Workspace- & Sessions-Scopet Telemetri (Zero-Daemon Bevaret)
- Antigravitys `PreInvocation`-dedup-cache nøgles med SHA-256 af `(canonical_workspace, session_id)` i stedet for globalt pr. OS-bruger (`<USER>`), så parallelle sessioner aldrig undertrykker hinandens cockpit-telemetri.
- `.xgauntlet/session_state.json` understøtter sessions-keyed state (når `session_id`/`conversation_id` er tilgængelig i hook-payloaden) med bagudkompatibel fallback til enkelt-session state.

### 6. Samlet Integrations-Gate på Koordinator-Branchen
- Hver worktree gennemfører sin egen lokale `RED → GREEN → REFACTOR → verify → checkpoint`-løkke mod sin bundne opgaves `scope`.
- Når én eller flere worker-branches flettes tilbage i koordinatorens integrationsbranch, SKAL der gennemføres en samlet `xgauntlet verify` og `xgauntlet check-architecture` før endelig `DONE`-forsegling, da disjunkte scopes stadig kan påvirke hinanden ved kompilering eller arkitekturgrænser.

## Consequences
- **Positiv**: Bevarer 100% Zero-Daemon-arkitektur ([ADR 0007](0007-local-transparent-supervisor-and-wasm-verifier.md)), Self-Mutation Invariant og atomiske Git checkpoints ([ADR 0003](0003-surgical-gatekeeper-and-no-remote-push.md)).
- **Positiv**: Muliggør både parallelle selvstændige tasks i worktrees og koordinator + subagent/swarm-arbejdsgange på tværs af Antigravity, Claude Code, Codex og Mistral Vibe.
- **Positiv**: Eliminerer CWD- og telemetri-kollisioner mellem samtidige sessioner.
- **Negativ / Trade-off**: Kræver opdatering af WIT-kontrakt (`wit/gauntlet_policy.wit`), rekompilering af den indlejrede WASM-policy-komponent samt opdatering af alle fire adapter-slices og deres kontrakt-tests.
- **Negativ / Trade-off**: Samtidig skrivning fra flere agenter i *ét* fælles working tree forbliver bevidst uunderstøttet som standard (`allow_coordinated_inherit = false`).

## Addendum (2026-11-15, Task 093): Praktisk Antigravity-Sameksistens & Opt-In Coordinated Inherit
Efter praktiske observationer i større Rust-workspaces (Kant) og Antigravitys orkestrator-dokumentation (`/swarm`, `/boost`, `Subagents`) præciseres fire supplerende regler under dette ADR:
1. **Bredere Read-Only Subagent Klassifikation:** Ud over `research` klassificeres `browser`, `image-generator` og custom `.agents/agents/<name>.md`-subagenter uden muterende fil-/shell-værktøjer i `tools` som `read_only`.
2. **`Workspace: "branch"`-Klon Opløsning:** `resolve_effective_workspace` understøtter ud over linked `git worktree` (`.git`-fil) også sikre lokale `git clone`-workspaces (bekræftet via `.git/objects/info/alternates` eller matching `remote.origin.url` mod parent-repoet).
3. **Opt-In Coordinated `inherit` (`[multi_agent] allow_coordinated_inherit = true`):** Da Antigravitys `/boost` og standard-`/swarm` bruger `inherit` med eksplicit fil-ejerskab og serialiseret `@verifier`-join, kan et projekt i `gauntlet.toml` aktivere `allow_coordinated_inherit = true`. Standard forbliver `false` (streng `4040`-spærre).
4. **Shared `CARGO_TARGET_DIR` & Race-Free `.xgauntlet/sessions/`:** Linked worktrees kan via `[multi_agent] share_cargo_target_dir = true` dele primary-repoets `target/`-mappe, og samtidige `PreInvocation`-sessioner isoleres atomisk i `.xgauntlet/sessions/<hash>.json`.

