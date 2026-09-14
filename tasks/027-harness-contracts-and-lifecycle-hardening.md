---
type: Task Package
title: "Task 027: Multi-Harness Hook Contracts, Schema Validation and Platform Hardening"
description: "Etablere præcise kontrakt- og schema/snapshot-tests for alle 4 harnesses (Antigravity, Claude Code, Codex, Mistral), mock-proces livscyklus på Windows/Unix, Wasm CRLF/Windows-path determinisme og platform-resilience"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-14T15:45:02Z" }
tags: [harness, contracts, schemas, claude-code, mistral-vibe, antigravity, codex, wasm, exit-codes, windows, unix, adr-0006]
---

# Task 027: Multi-Harness Hook Contracts, Schema Validation and Platform Hardening

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-09-14`  

## 🎯 Formål
Etablere en udtømmende test- og kontraktvalideringssuite for xgauntlets 4 harness-adaptere ([Antigravity](crates/xgauntlet-core/src/features/adapters/antigravity/), [Claude Code](crates/xgauntlet-core/src/features/adapters/claude_code/), [Codex](crates/xgauntlet-core/src/features/adapters/codex/), [Mistral Vibe](crates/xgauntlet-core/src/features/adapters/mistral/)) baseret på de officielle upstream-specifikationer jf. [spec.md](spec.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **Kontrakt- og Payload-Validering**: Validering af genererede konfigurationer og telemetri-/gatekeeper-payloads mod faste JSON schemas eller snapshot-fixtures på tværs af platforme.
2. **Claude Code Gatekeeper Opdatering**: Justering af `ClaudeCodeAdapter` til at benytte den officielle blokeringsmekanisme (Exit Code 2 og struktureret `hookSpecificOutput` med `permissionDecision = "deny"`).
3. **End-to-End Mock-Proces Livscyklus**: Simulering af reel harness-eksekvering med processpawning over native shell (`sh` på Unix, `cmd.exe`/PowerShell på Windows) for at teste stdin-piping, stdout/stderr parsing og exit-koder (0, 1, 2).
4. **Wasm Policy Engine Cross-Platform Determinisme**: Afvikling af Wasmtime-evalueringer med inputs indeholdende Windows-drev (`C:\...`), backslashes (`\`) og CRLF-linjeskift for at garantere identisk bit-for-bit output på Linux x86_64, ARM64, macOS og Windows.
5. **Filsystem- og Checkpoint-Resilience**: Afprøvning af lockfile-konflikter (`.git/index.lock`, `evidence.lock`) og fail-closed invariant-tjek under simulerede fillåse.
6. **HUD & Terminal Rendering Resilience**: Verifikation af at terminal-output (box-cards, ANSI-koder, Unicode box-drawing) håndterer uunderstøttede TTY- og console-kodetabeller uden at kaste panics.

---

## 📚 Harness Specifikationsreferencer & I/O Kontrakter (Primærkilder)

### 1. Google Antigravity IDE (`antigravity`)
- **Primærkilde**: Antigravity IDE Built-in Specifikation (`skills/agy-customizations/docs/hooks.md`).
- **Konfigurationsfil**: `.agents/hooks.json` (eller global `~/.gemini/config/hooks.json`).
- **Nøglestruktur**: JSON objekt med navngivne hook-blokke i protojson camelCase (`conversationId`, `workspacePaths`, `transcriptPath`, `artifactDirectoryPath`, `stepIdx`, `toolCall`).
- **Events**:
  - `PreInvocation`: Kører før modellen kaldes. Returnerer stdout JSON:
    ```json
    { "injectSteps": [{ "ephemeralMessage": "[XGAUNTLET COCKPIT TELEMETRY]\n..." }] }
    ```
  - `PreToolUse`: Kører før værktøj eksekveres. Modtager `{"toolCall": {"name": "...", "args": {...}}, ...}` på stdin.
    Returnerer stdout JSON:
    ```json
    {
      "decision": "allow" | "deny" | "ask" | "force_ask",
      "reason": "Forklaring",
      "overwrite": { "CommandLine": "..." }
    }
    ```
    Exit code 0 ved gyldig respons, exit code 1 ved parsefejl.
  - `PostToolUse`: Forventer tomt JSON-objekt `{}`.

### 2. Claude Code (`claude_code`)
- **Primærkilde**: `@anthropic-ai/claude-code` npm package & Claude Code official settings documentation.
- **Konfigurationsfil**: `.claude/settings.json` (eller global `~/.claude/settings.json`).
- **Miljøvariabler injiceret i hook processen**:
  - `CLAUDE_TOOL_NAME`: Navnet på værktøjet (`Bash`, `Edit`, `Write` osv.)
  - `CLAUDE_TOOL_INPUT`: Værktøjets argumenter serialiseret som JSON-streng
  - `CLAUDE_PROJECT_DIR`: Projektets rodsti
  - `CLAUDE_FILE_PATH`: Sti til berørt fil (ved fil-værktøjer)
  - `CLAUDE_SESSION_ID`: Aktivt sessions-ID
- **Nøglestruktur**: `{"hooks": {"PreToolUse": [...], "PostToolUse": [...]}}` med `matcher` (`"Bash"`, `"Edit|Write"`, `"*"`).
- **Events & Blokeringskontrakt**:
  - `PreToolUse`: Kører før værktøjskald. Modtager på stdin: `{"name": "...", "input": {...}}`.
    - **Blokering**: Returnerer **exit code 2** (officiel blocking code). Tekst på `stderr` sendes direkte til Claude som forklaring.
    - **Struktureret JSON på stdout**:
      ```json
      {
        "hookSpecificOutput": {
          "hookEventName": "PreToolUse",
          "permissionDecision": "deny",
          "permissionDecisionReason": "Afvisningsbegrundelse"
        }
      }
      ```
  - `PostToolUse`: Kører efter succesfuldt værktøj. Forventer exit code 0 og stdout JSON:
    ```json
    {
      "hookSpecificOutput": {
        "hookEventName": "PostToolUse",
        "additionalContext": "<Box-Card Telemetri>"
      }
    }
    ```

### 3. Mistral Vibe (`mistral`)
- **Primærkilde**: [mistralai/mistral-vibe](https://github.com/mistralai/mistral-vibe) (Apache-2.0) og [docs.mistral.ai/vibe/code/cli/hooks](https://docs.mistral.ai/vibe/code/cli/hooks).
- **Konfigurationsfil**: `.vibe/hooks.toml` (projekt-niveau) og `~/.vibe/hooks.toml` (bruger-niveau).
- **Nøglestruktur**: TOML `[[hooks]]` med felterne `name`, `type` (`pre_tool`, `post_tool`, `post_agent`), `command`, `match` (glob eller `re:` regex), `timeout` (float sekunder, default 60.0), `strict` (bool, default false).
- **Bemærkning om navngivning**: Vibe 2.21+ standardiserede hook-typer: `before_tool` $\rightarrow$ `pre_tool`, `after_tool` $\rightarrow$ `post_tool`, `post_agent_turn` $\rightarrow$ `post_agent`. Kommandoer afvikles shell-frit mod injection.
- **Events & Blokeringskontrakt**:
  - `pre_tool`: Modtager på stdin: `session_id`, `cwd`, `hook_event_name`, `tool_name`, `tool_call_id`, `tool_input`.
    - **Blokering**: Returnerer **exit code 0** med stdout JSON:
      ```json
      { "decision": "deny", "reason": "Afvisningsbegrundelse" }
      ```
    - `strict = true`: Non-zero exit betragtes som parse-/kørselsfejl og medfører hård afvisning.
  - `post_tool`: Kører kun hvis værktøjet kørte. Modtager `tool_status`, `tool_output`, `tool_output_text`, `duration_ms`.
    - Output på stdout JSON:
      ```json
      { "hook_specific_output": { "additional_context": "<Box-Card Telemetri>" } }
      ```
  - `post_agent`: Kører ved afslutning af assistent-turn.

### 4. OpenAI Codex / Tool Calling CLI (`codex`)
- **Primærkilde**: [openai/openai-openapi](https://github.com/openai/openai-openapi) (`openapi.yaml` komponenterne `ChatCompletionTool`, `FunctionObject`).
- **Tool-definition Schema**:
  ```json
  {
    "type": "function",
    "function": {
      "name": "string",
      "description": "string",
      "parameters": {
        "type": "object",
        "properties": {},
        "required": []
      },
      "strict": true
    }
  }
  ```
- **Konfigurationsfil**: `.codex/hooks.json` (eller aktiveret via `config.toml` med `codex_hooks = true`).
- **Instruktionsfiler**: `AGENTS.md` / `CODEX.md` indlæses som system- og developer-instruktioner.
- **Events**:
  - `PreToolUse`: Blokerer via exit code 2 eller `decision: "deny"`.
  - `PostToolUse`: Modtager `hookSpecificOutput.additionalContext`.

---

## 📋 Acceptance Criteria

### Klynge 1: Kontrakt-, Schema- og Snapshot-Validering
- [ ] Oprette deterministiske snapshot-fixtures eller JSON schema validering for de genererede filer for alle 4 adaptere:
  - [ ] `.agents/hooks.json` (Google Antigravity)
  - [ ] `.claude/settings.json` (Claude Code)
  - [ ] `.vibe/hooks.toml` (Mistral Vibe)
  - [ ] `.codex/hooks.json` (OpenAI Codex)
- [ ] Validere at serialiserede telemetri- og gatekeeper-payloads overholder felterne i ovenstående specifikationstabeller på tværs af Linux, macOS og Windows.

### Klynge 2: Claude Code Gatekeeper Opdatering & Exit Code 2
- [ ] Opdatere `ClaudeCodeAdapter::handle_hook` til:
  - [ ] Returnere exit code **2** ved afvisning af værktøjskald (i overensstemmelse med Claude Codes blokeringskontrakt).
  - [ ] Generere stdout JSON med `hookSpecificOutput.permissionDecision = "deny"` og `permissionDecisionReason`.
- [ ] Sikre at exit code 0 fortsat returneres ved tilladte handlinger (`"permissionDecision": "allow"`).

### Klynge 3: End-to-End Mock-Proces Livscyklus
- [ ] Oprette en integrationstest i `crates/xgauntlet-core/tests/` der spawner `xgauntlet hook` og `xgauntlet telemetry` som rigtige subprocesser.
- [ ] Teste piping af syntetiske tool-kald via stdin og modtagelse af stdout/stderr under:
  - [ ] Unix shell (`sh -c`)
  - [ ] Windows CMD (`cmd.exe /c` eller PowerShell hvis tilgængelig)
- [ ] Bekræfte identiske exit-koder (0 ved allow, 2/1 ved deny jf. harness) på tværs af platforme.

### Klynge 4: Wasm Policy Engine Sti- og Linjeskifts-Determinisme
- [ ] Tilføje integrationstest der evaluerer `gauntlet_policy.wasm` via Wasmtime med:
  - [ ] Windows absolutte stier (f.eks. `C:\repo\src\main.rs`) og relative Windows backslashes (`crates\core\lib.rs`).
  - [ ] CRLF (`\r\n`) linjeskift indlejret i `payload_json`.
- [ ] Verificere at Wasm-evalueringen leverer identiske verdicts og reason-koder uanset værts-OS.

### Klynge 5: Filsystem- og Checkpoint-Resilience
- [ ] Tilføje integrationstests for git checkpointing under filsystem-konflikter:
  - [ ] Opdagelse og sikker fejl-lukning hvis `.git/index.lock` er til stede.
  - [ ] Validering af at preflight-fejl ikke efterlader efterladte git stashing- eller commit-artefakter.

### Klynge 6: HUD & Terminal Rendering Resilience
- [ ] Sikre at `render_box_card` og `render_blockquote_hud` forbliver immune over for panics, hvis stdout er omdirigeret til non-TTY, fil eller et miljø uden ANSI escape-understøttelse.
- [ ] Verificere korrekt breddeberegning (64 synlige kolonner) ved UTF-8 multi-byte tegn.

---

## 🚫 Must NOT
- Må IKKE bryde Zero Ambient Authority for Wasm policy motoren (ingen filsystem- eller netværks-adgang indefra Wasm).
- Må IKKE introducere baggrunds-dæmoner jf. Zero-Daemon invarianten.
- Må IKKE foretage remote publication handlinger (`git push`).
- Må IKKE ændre eksisterende offentlige adapter-traits (`HarnessAdapter`) på en måde der bryder bagudkompatibilitet.
- Må IKKE maskere fejl eller slække på fail-closed princippet: Uparseable eller korrupt input på stdin SKAL altid resultere i afvisning.

---

## 📝 Revisions
- 2026-09-14: Task initialiseret med udtømmende specifikation, online harness-dokumentation og 6 konkrete test-klynger.

---

## 🧪 Verifikation
```bash
# Kør alle workspace integrationstests
cargo test --workspace

# Kør clippy uden advarsler
cargo clippy --workspace --all-targets -- -D warnings

# Verificer task-specifikationen mod Aristotelisk glossar
cargo run -p xgauntlet-cli -- check-spec -t 027-harness-contracts-and-lifecycle-hardening
```
