---
type: Task Package
title: "Task 022: Mistral Vibe Harness Adapter & Hooks Integration"
description: "Etablere Mistral Vibe autonom vertikal adapter-slice med normalisering af værktøjskald, pre_tool fail-closed gatekeeping, post_tool telemetri-injektion og .vibe/hooks.toml scaffolding jf. spec.md, ADR 0001, ADR 0004 og ADR 0006"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-09-13T15:36:00Z" }
tags: [mistral, vibe, adapters, hooks, toml, gatekeeper, telemetry, hud, vscode, cli, adr-0004, adr-0006]
---

# Task 022: Mistral Vibe Harness Adapter & Hooks Integration

**Status**: `DONE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-09-13`  

## 🎯 Formål
Etablere en autonom vertikal harness-adapter slice for **Mistral Vibe** i `crates/xgauntlet-core/src/features/adapters/mistral/` i fuld overensstemmelse med Mistral AI's officielle hook-specifikation ([docs.mistral.ai/vibe/code/cli/hooks](https://docs.mistral.ai/vibe/code/cli/hooks)), [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **Autonom Vertikal Feature-Slice (`features/adapters/mistral/`)**:
   - Oprette `MistralAdapter` der implementerer `HarnessAdapter`-traitet.
   - Kanonisk harness-identifikator `mistral` med aliaserne `mistral_vibe`, `mistral-vibe`, `vibe`.
   - Registrere i `SUPPORTED_HARNESSES` og `HarnessKind`.

2. **Normalisering af Værktøjskald (`normalize_tool_call`)**:
   - Oversætte Mistral Vibe værktøjer til strongly-typed `ToolActionType`:
     * `bash` / `exec` / shell ➔ `ToolActionType::RunCommand`
     * `write_file` / `write` ➔ `ToolActionType::WriteFile`
     * `edit` / `str_replace` ➔ `ToolActionType::WriteFile`
     * `read` / `view` ➔ `ToolActionType::ReadFile`
     * `grep` / `find` ➔ `ToolActionType::SearchCode`
     * `task` / subagent ➔ `ToolActionType::DelegateTask`
     * `ask_user_question` ➔ `ToolActionType::AskQuestion`

3. **Pre-Tool Lifecycle Hook Gatekeeper (`pre_tool`)**:
   - `handle_hook`: Læse JSON på stdin med `tool_name`, `tool_call_id`, `tool_input`.
   - Evaluerer kaldet mod in-memory Wasm policy motoren og sanitiserer relative workspace-stier.
   - Ved tilladelse: Returnerer exit code 0 og `{"decision": "allow"}`.
   - Ved afvisning: Returnerer exit code 0 og `{"decision": "deny", "reason": "<diagnostisk fejlmeddelelse>"}` (eller non-zero exit code ved fatale runtime-fejl), så Mistral Vibe agenten ser den præcise årsag til blokeringen.

4. **Post-Tool Telemetri & Cockpit HUD Injektion (`post_tool`)**:
   - Formaterer telemetri som JSON på stdout til `post_tool` hooket:
     ```json
     {
       "hook_specific_output": {
         "additional_context": "<Unicode Box-Drawing Telemetry Card / HUD>"
       }
     }
     ```
   - Tilføje formatet `mistral-hook` til `xgauntlet telemetry --format mistral-hook`.

5. **Scaffolding & Konfigurations-merging (`.vibe/hooks.toml`)**:
   - Implementere `generate_hooks_toml` og `scaffold_hooks`:
     * Genererer eller merger `./.vibe/hooks.toml` med `pre_tool` (gatekeeper) og `post_tool` (HUD telemetri).
     * Bevarer eksisterende brugerdefinerede hooks i TOML-filen.

6. **CLI & Init Udvidelse**:
   - Understøtte `--harness mistral` i `xgauntlet hook`, `xgauntlet init`, `xgauntlet doctor` og `xgauntlet telemetry`.

7. **Multi-Layer Conformance & Regressionstests**:
   - Teste normalisering, evaluering, hook-håndtering, telemetri-formatering og TOML-scaffolding i `crates/xgauntlet-core/tests/harness_adapters_test.rs`.

---

## 📋 Acceptance Criteria
- [x] `SUPPORTED_HARNESSES` indeholder `"mistral"` og `HarnessKind::parse_alias` genkender `"mistral"`, `"mistral_vibe"`, `"mistral-vibe"` og `"vibe"`.
- [x] `get_adapter("mistral")` returnerer en gyldig `Box<dyn HarnessAdapter>`.
- [x] `MistralAdapter::normalize_tool_call` mapper `bash`, `write_file`, `edit`, `read`, `grep`, `task` og `ask_user_question` korrekt til `ToolActionType`.
- [x] `MistralAdapter::handle_hook` håndterer `pre_tool` hooks:
  - Returnerer `{"decision": "allow"}` for godkendte handlinger.
  - Returnerer `{"decision": "deny", "reason": "..."}` for politisk afviste handlinger (f.eks. path traversal eller destructive shell commands).
- [x] `MistralAdapter::format_post_tool_use_payload` genererer gyldig JSON med `hook_specific_output.additional_context`.
- [x] `xgauntlet telemetry --format mistral-hook` genererer JSON-payload klar til Mistral Vibes `post_tool` hook.
- [x] `MistralAdapter::scaffold_hooks` opretter eller merger `./.vibe/hooks.toml` uden at ødelægge eksisterende TOML-konfiguration.
- [x] CLI subcommand `xgauntlet hook --harness mistral` modtager og evaluerer Mistral Vibe stdin JSON.
- [x] CLI subcommand `xgauntlet init --harness mistral` stilladserer `./.vibe/hooks.toml`.
- [x] Alle nye og eksisterende enhedstests i `crates/xgauntlet-core/tests/harness_adapters_test.rs` består med 100% grøn status.

---

## 🚫 Must NOT
- **Zero-Daemon**: Må ikke køre baggrundstjenester eller starte sockets.
- **Zero Ambient Authority**: Ingen netværkskald til `api.mistral.ai` eller afhængighed af eksterne API-nøgler under verifikation.
- **Isoleret Feature-Slice (ADR 0004)**: Må ikke forurene eller introducere kobling til `antigravity`, `claude_code` eller `codex`.
- **Fail-Closed**: Ugyldig eller korrupt JSON skal altid afvises, aldrig godkendes stiltiende.
- **Ingen Destruktive Git Operationer**: ADR 0003 skal overholdes under hele forløbet.

---

## 📝 Revisions
- 2026-09-13: Opgave initialiseret på baggrund af officiel Mistral Vibe hook-dokumentation ([docs.mistral.ai/vibe/code/cli/hooks](https://docs.mistral.ai/vibe/code/cli/hooks)).

---

## 🧪 Verifikation
- `cargo test --test harness_adapters_test`
- `cargo run -p xgauntlet-cli -- check-spec`
- `cargo run -p xgauntlet-cli -- telemetry --harness mistral -f mistral-hook`
- `cargo run -p xgauntlet-cli -- verify`
