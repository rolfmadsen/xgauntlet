# Verification Report

**Task ID**: `028-fix-windows-powershell-stdin-bom`  
**Task Title**: Task 028: Fix Windows PowerShell Hook Stdin BOM Immunity  
**Verdict**: `PARTIAL`  
**Execution Origin**: `LOCAL`  
**Source Manifest Digest**: `011928f86d7b1760ab55a5b0a8ccf2be0735ce18ceecce124084aee41943dc79`  
**Timestamp**: `2026-09-14T20:23:40Z`  
**Head**: `0309f10`  
**Commit**: `0309f10`  

## Acceptance Criteria

- [x] Implementere `clean_stdin` helper i `xgauntlet-core::features::adapters` der fjerner UTF-8 BOM (`\u{feff}`) og normaliserer whitespace.
- [x] Opdatere alle 4 harness-adaptere (`ClaudeCodeAdapter`, `CodexAdapter`, `AntigravityAdapter`, `MistralAdapter`) til at rense `stdin_content` for BOM før JSON parsing.
- [x] Tilføje unit-test for BOM-immunitet på tværs af alle 4 adaptere (`test_all_four_adapters_bom_immunity`).
- [x] Tilføje integrationstest for subprocess execution med rå UTF-8 BOM bytes (`\xef\xbb\xbf`) over stdin.
- [x] Sikre at `test_subprocess_hook_via_native_shell_piping` konfigurerer `$OutputEncoding = [System.Text.UTF8Encoding]::new($false)` og kører stabilt på Windows PowerShell.
- [x] Verificere at samtlige tests og gauntlet-lag passerer (`cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`).

---

## Verification Checks

| Check Name | Status | Exit Code | Duration (s) |
|---|---|---|---|
| `lint` | `PASSED` | `0` | `0.100s` |
| `types` | `PASSED` | `0` | `0.085s` |
| `unit` | `PASSED` | `0` | `15.211s` |
| `invariants` | `PASSED` | `0` | `0.353s` |
| `mutation-testing-gauntlet` | `FAILED` | `101` | `0.010s` |

---
