---
type: Task Package
title: "Task 060: Brownfield VSA Migration Blueprint in README Documentation"
description: "Udvidet afsnit 7.4 i README.md og packages/cli/README.md med en detaljeret 4-trins køreplan for brownfield-migrering: Audit-røntgenbillede, Baseline-exemptions, rækkefølge (Gate C -> B -> A) samt Stateful UI-Slice mønstret."
status: done
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-07T21:23:00Z" }
tags: [documentation, brownfield, migration, vsa, architecture, blueprint]
scope:
  - "crates/xgauntlet-core/tests/**"
  - "README.md"
  - "packages/cli/README.md"
  - "tasks/060-brownfield-migration-blueprint-in-readme.md"
test_refs:
  - "crates/xgauntlet-core/tests/architecture_scaffold_template_test.rs"
---

# Task 060: Brownfield VSA Migration Blueprint in README Documentation

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-07`

---

## 🎯 Formål & Baggrund

Audit af afsnit 7.4 (*Trinvis Brownfield-Migrering*) i `README.md` afslørede, at afsnittet i dag kun viser den rå TOML-syntaks for `[[architecture.exemptions]]` uden at forklare den praktiske metodik og rækkefølge for, hvordan et komplekst produktionssystem (som f.eks. Kant med 128 tasks) refaktoreres sikkert og trygt til Vertical Slice Architecture.

Formålet med denne opgave er at udbygge afsnit 7.4 i både `README.md` og `packages/cli/README.md` med den fulde operationelle drejebog:
1. **Risikofrit Røntgenbillede (Audit):** Kørsel af `xgauntlet check-architecture` som baseline-scanning.
2. **Sikker Baseline uden Big-Bang:** Frysning af status quo via en midlertidig `allow_imports` exemption, så buildet forbliver grønt, mens alle nye moduler tvinges til VSA fra dag 1.
3. **Optimal Oprydningsrækkefølge:** Start med Gate C (rens domænekernen), dernæst Gate B (isoler slices indbyrdes), og afslut med Gate A (afkobl fra `app::Message` via lokale `Action`/`Event`-enums og facade jf. ADR 0012).
4. **Slice-for-Slice Afvikling:** Trinvis eliminering af exemptions task-for-task ved hjælp af AI-agenten.

---

## 📋 Acceptance Criteria

- [x] **AC1 — 4-Trins Drejebog i GitHub `README.md`:**
  - Udvid afsnit 7.4 i `README.md` med de 4 trin (Røntgenbillede/Audit, Baseline-exemptions, Gate C $\to$ B $\to$ A rækkefølge med lokale Actions, samt Slice-for-slice afvikling).
- [x] **AC2 — Paritet i npm `packages/cli/README.md`:**
  - Opdater afsnit 7.4 i `packages/cli/README.md` til fuld 100% paritet med `README.md`.
- [x] **AC3 — Automatisk Dokumentationstest:**
  - Udvid `crates/xgauntlet-core/tests/architecture_scaffold_template_test.rs` med assertions, der validerer at begge README-filer indeholder drejebogens nøglebegreber (Røntgenbillede/Audit, Baseline-exemptions, Gate C, Gate B, Gate A, samt Action/Event mønstret).

---

## 🚫 Must NOT

- Må IKKE fjerne eller forsimple de eksisterende kodeeksempler i afsnit 7.
- Må IKKE ændre ved `gauntlet.toml`-parseren eller bryde eksisterende TOML-kontrakter.
- Må IKKE udføre remote publication (`git push`) jf. ADR 0003.

---

## 📝 Revisions

- 2026-11-07: Oprettet Task 060 for berigelse af afsnit 7.4 med brownfield-migreringsdrejebog.

---

## 🧪 Verifikation

- `cargo run -p xgauntlet-cli -- check-spec -t 060`
- `cargo test --test architecture_scaffold_template_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
