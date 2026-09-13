---
name: old-coder
description: Evidence-first development (SPEC -> RED -> GREEN -> REFACTOR -> GAUNTLET -> EVIDENCE). Surround code with an executable spec and test gauntlet so line-by-line review becomes optional.
---

# Evidence-First TDD (`old-coder`)

Evidence-first development surrounds implementation with an executable spec and a gauntlet of constraints (tests, types, coverage, mutation) so line-by-line review becomes optional.

Every cycle follows the **red → green → refactor** loop strictly at pre-agreed public seams.

## What a good test is

Tests verify behavior through public interfaces, not implementation details. Code can change entirely; tests shouldn't. A good test reads like a specification: `"user can checkout with valid cart"` tells you exactly what capability exists, and it survives refactors because it doesn't care about internal structure.

See [tests.md](./tests.md) for examples and [mocking.md](./mocking.md) for mocking guidelines.

## Seams: where tests go

A **seam** is the public boundary you test at: the interface where you observe behavior without reaching inside. Tests live at seams, never against internals.

**Test only at pre-agreed seams.** Before writing any test, write down the seams under test and confirm them with the user. No test is written at an unconfirmed seam. You can't test everything, so agreeing the seams up front is how testing effort lands on the critical paths and complex logic instead of every edge case.

Ask: *"What's the public interface, and which seams should we test?"*

## Anti-patterns

- **Implementation-coupled**: mocks internal collaborators, tests private methods, or verifies through a side channel (querying the database instead of using the interface). The tell: the test breaks when you refactor but behavior hasn't changed.
- **Tautological**: the assertion recomputes the expected value the way the code does (`expect(add(a, b)).toBe(a + b)`), so it passes by construction and can never disagree with the code. Expected values must come from an independent source of truth: a known-good literal, a worked example, or the spec.
- **Horizontal slicing**: writing all tests first, then all implementation. Bulk tests verify _imagined_ behavior: you test the _shape_ of things rather than user-facing behavior. Work in **vertical slices** instead: one test → one implementation → repeat, each test a **tracer bullet** responding to what the last cycle taught you.

## Rules of the loop

1. **SPEC / GRILL**: Concrete executable criteria in `tasks/<task>.md` and `spec.md`.
2. **RED**: Write the failing acceptance/unit test first. Prove it fails with the expected failure mode.
3. **GREEN**: Write only the minimal implementation necessary to make the test pass. No speculative features.
4. **REFACTOR**: Clean up module boundaries, types, and duplication while assertions remain frozen.
5. **GAUNTLET**: Run multi-layer verification (`cargo run -p xgauntlet-cli -- verify` or `xgauntlet verify`).
6. **EVIDENCE**: Seal evidence into `verification-report.json` and `evidence.md`.

## 🔒 ADR 0003 Checkpoint Protocol
To prevent context rot and ensure rollback points, make local commits at each phase boundary:
- `SPEC`: `task(<id>): initialize task specification and criteria`
- `RED`: `test(<id>): add failing acceptance test for <feature> [RED]`
- `GREEN`: `feat(<id>): implement minimal logic to satisfy test [GREEN]`
- `REFACTOR`: `refactor(<id>): clean up module boundaries and types [REFACTOR]`
- `DONE`: `chore(<id>): seal evidence and mark task DONE`

*Never push to remote (`git push`) or run destructive resets (`git reset --hard`).*
