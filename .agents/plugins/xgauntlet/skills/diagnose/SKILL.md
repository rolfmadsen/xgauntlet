---
name: diagnose
description: Disciplined diagnosis loop for hard bugs and regressions. Reproduce -> minimise -> hypothesise -> instrument -> fix -> regression-test.
---

# Disciplined Diagnosis Loop (`diagnose`)

Use this disciplined diagnosis loop whenever investigating hard bugs, regressions, test failures, or crashes.

**Security first**: Redact every secret (`<REDACTED>`) before showing outputs or logs.

---

## Phase 1: Build a feedback loop

**This is the skill.** Everything else is mechanical. If you have a **tight** pass/fail signal for the bug (one that goes red on _this_ bug), you will find the cause; bisection, hypothesis-testing, and instrumentation all just consume it. If you don't have one, no amount of staring at code will save you.

Spend disproportionate effort here. **Be aggressive. Be creative. Refuse to give up.**

### Ways to construct one, in roughly this order:
1. **Failing test** at whatever seam reaches the bug: unit, integration, e2e.
2. **Curl / HTTP script** against a running server.
3. **CLI invocation** with a fixture input, diffing stdout against a known-good snapshot.
4. **Headless script / driver** that exercises the failure path.
5. **Replay a captured trace**: save a real payload/event to disk; replay through the code path in isolation.
6. **Throwaway harness**: spin up a minimal subset of the system exercising the bug with a single call.
7. **Property / fuzz loop**: if the bug is intermittent, loop 1000 random inputs.
8. **Bisection harness**: automate "boot at commit X, check, repeat" for `git bisect run`.
9. **Differential loop**: run same input through old vs new version and diff outputs.

### Completion criterion for Phase 1
Phase 1 is done when the loop is **tight** and **red-capable**:
- [ ] **Red-capable**: drives the bug code path and asserts the user's exact symptom.
- [ ] **Deterministic**: same verdict every run.
- [ ] **Fast**: seconds, not minutes.
- [ ] **Agent-runnable**: executable unattended without manual intervention.

*If you catch yourself reading code to build a theory before this command exists, stop! Jumping straight to a hypothesis is the exact failure this skill prevents.*

---

## Phase 2: Reproduce + minimise

Run the loop. Watch it go red as the bug appears.

Confirm:
- [ ] The loop reproduces the exact failure mode the user described.
- [ ] The failure is reproducible across multiple runs.
- [ ] You captured the exact symptom (error, output, trace).

### Minimise
Shrink the repro to the **smallest scenario that still goes red**. Cut inputs, callers, config, data, and steps **one at a time**, re-running the loop after each cut.
Done when **every remaining element is load-bearing**: removing any one of them makes the loop go green.

---

## Phase 3: Hypothesise

Generate **3–5 ranked hypotheses** before testing any of them. Single-hypothesis generation anchors on the first plausible idea.

Each hypothesis must be **falsifiable**:
> Format: *"If <X> is the cause, then <changing Y> will make the bug disappear / <changing Z> will make it worse."*

Show the ranked list to the user before testing.

---

## Phase 4: Instrument

Each probe must map to a specific prediction from Phase 3. **Change one variable at a time.**
- Tool preference: debugger inspection or targeted logs at boundaries.
- **Tag every debug log** with a unique prefix, e.g. `[DEBUG-a4f2]`. Cleanup at the end becomes a single grep.
- For performance regressions: establish a baseline measurement first, then bisect.

---

## Phase 5: Fix + regression test

Write the regression test **before the fix**, but only if there is a **correct seam** for it.
1. Turn the minimised repro into a failing test at that seam.
2. Watch it fail.
3. Apply the fix.
4. Watch it pass.
5. Re-run the Phase 1 feedback loop against the original (un-minimised) scenario.

---

## Phase 6: Cleanup

Required before declaring done:
- [ ] Original repro no longer reproduces (re-run Phase 1 loop).
- [ ] Regression test passes (run `xgauntlet verify`).
- [ ] All `[DEBUG-...]` instrumentation removed (`git grep` the prefix).
- [ ] Throwaway prototypes deleted.
- [ ] The correct hypothesis is documented in the commit message.
