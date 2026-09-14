---
type: Task Package
title: "Task 028: Fix Windows PowerShell Hook Stdin BOM Immunity"
description: "Fix Windows CI failure in harness_contracts_test caused by PowerShell UTF-8 BOM in native shell piping"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-14T22:15:00Z" }
tags: [harness, contracts, windows, powershell, bom, utf8, stdin, bug-fix]
---

# Task 028: Fix Windows PowerShell Hook Stdin BOM Immunity

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-09-14`

## 🎯 Formål
Løse Windows CI regressionsfejl i `harness_contracts_test.rs` (`test_subprocess_hook_via_native_shell_piping`), hvor PowerShells pipeline til native processer udsender et UTF-8 Byte Order Mark (`\u{FEFF}` / `0xEF, 0xBB, 0xBF`), hvilket fik `serde_json::from_str` til at fejle med `Corrupt JSON payload on stdin: expected value at line 1 column 1`.

## 📋 Acceptance Criteria
- [ ] Implementere `clean_stdin` helper i `xgauntlet-core::features::adapters` der fjerner UTF-8 BOM (`\u{feff}`) og normaliserer whitespace.
- [ ] Opdatere alle 4 harness-adaptere (`ClaudeCodeAdapter`, `CodexAdapter`, `AntigravityAdapter`, `MistralAdapter`) til at rense `stdin_content` for BOM før JSON parsing.
- [ ] Tilføje unit-test for BOM-immunitet på tværs af alle 4 adaptere (`test_all_four_adapters_bom_immunity`).
- [ ] Tilføje integrationstest for subprocess execution med rå UTF-8 BOM bytes (`\xef\xbb\xbf`) over stdin.
- [ ] Sikre at `test_subprocess_hook_via_native_shell_piping` konfigurerer `$OutputEncoding = [System.Text.UTF8Encoding]::new($false)` og kører stabilt på Windows PowerShell.
- [ ] Verificere at samtlige tests og gauntlet-lag passerer (`cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`).

## 🚫 Must NOT
- Må IKKE ændre eksisterende API-kontrakter eller returnerede exit-koder for gyldige/ugyldige payloads.
- Må IKKE fjerne eller slække på sikkerhedsvalideringen i Wasm-policy motoren eller gatekeeper-adapterne.
- Må IKKE foretage remote publication handlinger (`git push`).
- Må IKKE bryde Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-14: Task oprettet for at løse Windows CI fejl i `test_subprocess_hook_via_native_shell_piping`.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test harness_contracts_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
