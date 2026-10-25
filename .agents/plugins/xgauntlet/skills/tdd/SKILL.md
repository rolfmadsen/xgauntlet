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
SPEC ➔ RED ➔ GREEN ➔ REFACTOR ➔ VERIFY / GAUNTLET
```

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

## 🚫 TDD Invarianter
- Skriv ALDRIG produktionskode før en fejlet test har påvist behovet (RED-first).
- Tilføj ALDRIG tests til monolitiske testfiler over 1.000 linjer.
- Foretag ALDRIG remote publication (`git push`) under TDD løkken.
- Foretag ALDRIG destruktive Git-kommandoer (`git reset --hard` eller `git clean -f`).
- Nye tests må ALDRIG fødes grønne. Hvis et nyt kriterie eller kant-case opdages under GREEN, skal testen bevises rød via en ny RED-cyklus eller verificeres via xGauntlets inverse preflight.
