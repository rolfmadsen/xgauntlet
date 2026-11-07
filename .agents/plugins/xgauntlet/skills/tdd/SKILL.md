---
name: tdd
description: Streng Evidence-First Red-Green-Refactor test-drevet udvikling med lokale Git phase checkpoints, isolerede feature tests og forbud mod monolitiske testfiler (>1.000 linjer).
---

# Sovereign Skill: tdd

Fase 3 i xGauntlets Agentic SDLC: **Implementation via Test-Driven Development**.

`tdd` sikrer matematisk og empirisk beviselighed for al kode gennem den klassiske Red-Green-Refactor løkke koblet med lokale Git checkpoints.

---

## 🔄 TDD Faseløkken & Lokale Checkpoints

```text
SPEC (Human Gate) ➔ [RED ➔ GREEN ➔ REFACTOR (Autonomous Inner Loop)] ➔ GAUNTLET ➔ AUDIT (Human Gate)
```

### Autonomous TDD Inner Loop Princippet
Human-in-the-Loop gates tilhører opgavens ydre grænser:
1. **Gate 1: SPEC Godkendelse** – Mennesket godkender scope, kriterier og test-seams.
2. **Gate 2: AUDIT & HANDOFF Godkendelse** – Mennesket godkender review, evidens og endelig opgavelukning.

Den indre løkke `[RED ➔ GREEN ➔ REFACTOR]` er **100% autonom**:
Når en rød accepttest er skrevet og preflight-verificeret via `xgauntlet checkpoint --phase red`, må agenten **IKKE** standse op eller afvente brugergodkendelse. Agenten overgår umiddelbart til GREEN fasen i én sammenhængende eksekvering, da den mekaniske preflight og det atomiske Git-commit allerede garanterer sikkerhed og tilbagerulning.

### 1. SPEC Checkpoint
Når opgaven er godkendt af brugeren og valideret via `check-spec`:
```bash
git commit -m "task(<id>): initialize task specification and criteria"
```

### 2. RED Fase (Skriv den fejlede accepttest først)
- Skriv nye accepttests i den dedikerede testfil defineret under `test_refs` (IKKE i en monolitisk test-god-fil).
- Kør testen og **bevis**, at den fejler med den forventede årsag.
- Udfør lokalt checkpoint:
```bash
git commit -m "test(<id>): add failing acceptance test for <feature> [RED]"
```
- **Autonom Overgang**: Fortsæt straks til GREEN fasen uden turn-pause!

### 3. GREEN Fase (Minimal implementation)
- Skriv den *minimale* mængde produktionskode, der får testen til at passere.
- Hold dig strengt inden for opgavens deklarerede `scope: [...]`.
- Verificer at alle tests passerer.
- Udfør lokalt checkpoint:
```bash
git commit -m "feat(<id>): implement minimal logic to satisfy test [GREEN]"
```

### 4. REFACTOR Fase (Oprydning uden adfærdsændring)
- Refaktorer for læsbarhed, modularitet, typesikkerhed og fjernelse af duplikering.
- Sørg for at ingen filer overstiger `max_file_lines` (800 linjer).
- Sørg for at ingen rå styling eller forbudte tokens sniger sig ind.
- Kør testene igen for at bevise, at refaktoreringen ikke brød eksisterende adfærd.
- Udfør lokalt checkpoint:
```bash
git commit -m "refactor(<id>): clean up module boundaries and types [REFACTOR]"
```

---

## ⚡ Fast-Track Bugfix Lane (`intent: bug`)

For afgrænsede fejlrettelser understøttes en accelereret verifikationssti jf. [ADR 0011](docs/adr/0011-fast-track-worktree-inverse-proof-and-scoped-preflight.md):
- **Betingelser for Fast-Track**:
  1. Task intent er `bug` (`intent: bug` / `🐛 BUG FIX`).
  2. `scope` indeholder højst 3 kildefiler (`scope.len() <= 3`).
  3. Ingen berøring af `docs/adr/` eller `gauntlet.toml`.
- **2-trins Flow**: `checkpoint -p red` (kun `test_refs`) ➔ bugfix i `scope` ➔ `checkpoint -p done` (fuld verifikation og evidensforsegling). Separat `SPEC`- og `REFACTOR`-commit er valgfrie.
- **1-trins Atomisk Worktree Inverse Proof**: Hvis test og fix leveres i ét trin, udfører xGauntlet automatisk invers verifikation i en isoleret midlertidig git worktree mod uændret HEAD. Testen skal fejle mod HEAD for at bevise sin negative evne; hvis den passerer mod HEAD, afvises den prompte som `BORN_GREEN_TEST`.

---

## 🏗️ Konsolideret Test-Harness Mønster (Undgå Linker-Inflation)

I større Rust-workspaces fører snesevis af separate `[[test]]`-binarier til voldsomt linker-overhead. Anvend i stedet **modul-opdelte accepttests under én samlet harness**:
- Opret rod-harness i `tests/features.rs` (eller `tests/features_acceptance.rs`).
- Placer de enkelte feature-accepttests som moduler i `tests/features/<feature>_acceptance_test.rs`.
- Hver modulfil holdes skarpt afgrænset under 1.000 linjer jf. `max_test_file_lines`, men hele suiten kompileres og linkes som én samlet binærfil.
- Scoped preflight eksekverer målrettet via `cargo test --test features <submodule>`.

---

## 🧱 Black-Box Vertical Slice Indkapsling (Polyglot: Rust/Iced, TS/React, Java)

Under `GREEN` og `REFACTOR` overholdes de deterministiske grænse-gates i `xgauntlet check-architecture` (`app -> features -> core -> shared`, 0% intern mappestruktur-rigiditet indeni slicen):
- **Stateful UI-Slice (`features/<domain>/<slice>/`):** Ejer sin egen tilstand (`State`), sine egne interne beskeder/events (`Message`) og eksponerer en eksplicit udgående `Action`/callback til `app`-orchestratoren. Importerer **aldrig** `crate::app::Message` eller global app-store baglæns.
- **Stateless Pipeline-Slice (`features/<domain>/<slice>/`):** Ren funktionel transformation med eksplicit `Command`/`Input` og `Result`/`Output` over domænetyper fra `core`.

### Polyglot Eksempler (Rust/Iced, TypeScript/React, Java)
- **Rust / Iced (`src/features/diagram/canvas/mod.rs`):**
  ```rust
  pub struct CanvasState { pub zoom: f32 }
  #[derive(Clone, Debug)]
  pub enum CanvasMessage { ZoomChanged(f32), NodeClicked(NodeId) }
  pub enum CanvasAction { None, InspectNode(NodeId) }
  pub fn update(state: &mut CanvasState, msg: CanvasMessage) -> CanvasAction { ... }
  ```
- **TypeScript / React (`src/features/diagram/canvas/CanvasSlice.tsx`):**
  ```tsx
  import type { DiagramNode } from '@/core/diagram';
  export interface CanvasSliceProps {
    nodes: readonly DiagramNode[];
    onAction: (action: { type: 'INSPECT_NODE'; nodeId: string }) => void;
  }
  export function CanvasSlice({ nodes, onAction }: CanvasSliceProps) { ... }
  ```
- **Java (`com.acme.features.billing.invoice.InvoiceSlice`):**
  ```java
  package com.acme.features.billing.invoice;
  import com.acme.core.money.Money;
  public final class InvoiceSlice {
      public record IssueInvoiceCommand(String customerId, Money total) {}
      public record InvoiceIssuedEvent(String invoiceId) {}
      public InvoiceIssuedEvent issue(IssueInvoiceCommand cmd) { ... }
  }
  ```

---

## 🚫 TDD Invarianter
- Skriv ALDRIG produktionskode før en fejlet test har påvist behovet (RED-first).
- Tilføj ALDRIG tests til monolitiske testfiler over 1.000 linjer.
- Foretag ALDRIG remote publication (`git push`) under TDD løkken.
- Foretag ALDRIG destruktive Git-kommandoer (`git reset --hard` eller `git clean -f`).
- Nye tests må ALDRIG fødes grønne. Hvis et nyt kriterie eller kant-case opdages under GREEN, skal testen bevises rød via en ny RED-cyklus eller verificeres via xGauntlets inverse preflight.
