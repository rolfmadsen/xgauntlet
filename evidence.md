# Verification Report

**Task ID**: `027-harness-contracts-and-lifecycle-hardening`  
**Task Title**: Task 027: Multi-Harness Hook Contracts, Schema Validation and Platform Hardening  
**Verdict**: `PARTIAL`  
**Execution Origin**: `LOCAL`  
**Source Manifest Digest**: `cb80c1fceb13607620ab1b352452bddabae04be178737fc12388df24a46bdc6c`  
**Timestamp**: `2026-09-14T19:56:59Z`  
**Head**: `cf8d668`  
**Commit**: `cf8d668`  

## Acceptance Criteria

- [x] Oprette deterministiske snapshot-fixtures eller JSON schema validering for de genererede filer for alle 4 adaptere:
- [x] `.agents/hooks.json` (Google Antigravity)
- [x] `.claude/settings.json` (Claude Code)
- [x] `.vibe/hooks.toml` (Mistral Vibe)
- [x] `.codex/hooks.json` (OpenAI Codex)
- [x] Validere at serialiserede telemetri- og gatekeeper-payloads overholder felterne i ovenstående specifikationstabeller på tværs af Linux, macOS og Windows.
- [x] Opdatere `ClaudeCodeAdapter::handle_hook` til:
- [x] Returnere exit code **2** ved afvisning af værktøjskald (i overensstemmelse med Claude Codes blokeringskontrakt).
- [x] Generere stdout JSON med `hookSpecificOutput.permissionDecision = "deny"` og `permissionDecisionReason`.
- [x] Sikre at exit code 0 fortsat returneres ved tilladte handlinger (`"permissionDecision": "allow"`).
- [x] Oprette en integrationstest i `crates/xgauntlet-core/tests/` der spawner `xgauntlet hook` og `xgauntlet telemetry` som rigtige subprocesser.
- [x] Teste piping af syntetiske tool-kald via stdin og modtagelse af stdout/stderr under:
- [x] Unix shell (`sh -c`)
- [x] Windows CMD (`cmd.exe /c` eller PowerShell hvis tilgængelig)
- [x] Bekræfte identiske exit-koder (0 ved allow, 2/1 ved deny jf. harness) på tværs af platforme.
- [x] Tilføje integrationstest der evaluerer `gauntlet_policy.wasm` via Wasmtime med:
- [x] Windows absolutte stier (f.eks. `C:\repo\src\main.rs`) og relative Windows backslashes (`crates\core\lib.rs`).
- [x] CRLF (`\r\n`) linjeskift indlejret i `payload_json`.
- [x] Verificere at Wasm-evalueringen leverer identiske verdicts og reason-koder uanset værts-OS.
- [x] Tilføje integrationstests for git checkpointing under filsystem-konflikter:
- [x] Opdagelse og sikker fejl-lukning hvis `.git/index.lock` er til stede.
- [x] Validering af at preflight-fejl ikke efterlader efterladte git stashing- eller commit-artefakter.
- [x] Sikre at `render_box_card` og `render_blockquote_hud` forbliver immune over for panics, hvis stdout er omdirigeret til non-TTY, fil eller et miljø uden ANSI escape-understøttelse.
- [x] Verificere korrekt breddeberegning (64 synlige kolonner) ved UTF-8 multi-byte tegn.

---

## Verification Checks

| Check Name | Status | Exit Code | Duration (s) |
|---|---|---|---|
| `lint` | `PASSED` | `0` | `9.656s` |
| `types` | `PASSED` | `0` | `1.620s` |
| `unit` | `PASSED` | `0` | `17.220s` |
| `invariants` | `PASSED` | `0` | `0.339s` |
| `mutation-testing-gauntlet` | `FAILED` | `101` | `0.009s` |

---
