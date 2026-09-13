---
type: Task Package
title: "Task 024: Fix Windows CI CRLF Frontmatter Assertion and Gitattributes"
description: "Fix Windows CI failure in plugin_distribution_test caused by CRLF frontmatter assertion and enforce LF line endings via .gitattributes"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-09-13T19:19:58Z" }
tags: [task-lifecycle, intent, scaffolding, rust]
---

# Task 024: Fix Windows CI CRLF Frontmatter Assertion and Gitattributes

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-09-13`

## 🎯 Formål
Fix Windows CI failure in plugin_distribution_test caused by CRLF frontmatter assertion and enforce LF line endings via .gitattributes

## 📋 Acceptance Criteria
- [x] Gøre YAML frontmatter assertion i `plugin_distribution_test.rs` robust overfor både LF (`---\n`) og CRLF (`---\r\n`).
- [x] Tilføje regressions/reproduktionstest for CRLF frontmatter genkendelse.
- [x] Oprette `.gitattributes` i roden med `* text=auto eol=lf` og binære filtyper for at sikre ensartede LF line endings på Windows CI.
- [x] Verificere at samtlige tests og invariant-tjek passerer (`cargo test --workspace`, `cargo clippy`, `check-spec`).

## 🚫 Must NOT
- Må IKKE bryde eksisterende arkitektur-invarianter eller API-kontrakter.
- Må IKKE foretage remote publication handlinger (`git push`).
- Må IKKE introducere baggrunds-dæmoner jf. Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-13: Task oprettet som ACTIVE for Fix Windows CI CRLF Frontmatter Assertion and Gitattributes.

## 🧪 Verifikation
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-spec -t 024-fix-windows-crlf-frontmatter`
