---
name: code-review
description: Review changes along two axes in parallel sub-agents: Standards (documented standards + Fowler code smells baseline) vs Spec (fidelity to the originating spec/issue).
---

# Two-Axis Code Review (`code-review`)

Two-axis review of the diff between `HEAD` and a fixed point the user supplies:
- **Standards**: does the code conform to this repo's documented standards and architectural invariants?
- **Spec**: does the code faithfully implement the originating task package / spec?

Both axes run as **parallel sub-agents** so they don't pollute each other's context, then this skill aggregates their findings.

---

## Process

### 1. Pin the fixed point

Whatever the user said is the fixed point (a commit SHA, branch name, tag, `main`, `HEAD~1`, etc.).
Capture the diff command: `git diff <fixed-point>...HEAD` (three-dot, comparison against merge-base).
Confirm the fixed point resolves: `git rev-parse <fixed-point>`.

### 2. Identify the spec source

Look for the originating spec/task in this order:
1. Task files under `tasks/` (e.g. `tasks/<id>.md`).
2. Macro specification in `spec.md`.
3. Issue reference in the commit messages (`#123`, `task(023): ...`).
4. An explicit path the user provided.

If no spec is found, the Spec sub-agent will skip and report *"no spec available"*.

### 3. Identify the standards sources & Smell baseline

Look for `AGENTS.md`, `docs/adr/`, `CODING_STANDARDS.md`, or compiler/linter configs.
On top of documented standards, the Standards axis always carries the **smell baseline** (Martin Fowler's *Refactoring*):

- **Mysterious Name**: function, variable, or type whose name doesn't reveal what it does.
- **Duplicated Code**: the same logic shape appears in multiple places.
- **Feature Envy**: a method that reaches into another object's data more than its own.
- **Data Clumps**: the same few fields or parameters always travel together.
- **Primitive Obsession**: primitives or raw strings standing in for domain concepts that deserve a type.
- **Repeated Switches**: identical `match` / `switch` branches recurring across the diff.
- **Shotgun Surgery**: one logical change forces scattered edits across many files.
- **Divergent Change**: one module is edited for several unrelated reasons.
- **Speculative Generality**: hooks, traits, or parameters added for needs the spec doesn't require.
- **Message Chains**: long `a.b().c().d()` navigation where the caller shouldn't know internal structure.
- **Middle Man**: a function or struct that mostly just delegates onward.
- **Refused Bequest**: a struct or class that ignores or overrides most of what it inherits.

### 4. Spawn both sub-agents in parallel

**Standards sub-agent**:
Report: (a) every place the diff violates a documented repo standard or architectural invariant (with file and line citation); (b) any baseline Fowler smell spotted (name it and quote the hunk). Distinguish hard violations from judgement calls. Under 400 words.

**Spec sub-agent**:
Report: (a) requirements the task/spec asked for that are missing or partial; (b) behavior in the diff that was not asked for (scope creep); (c) requirements that look implemented but where the implementation looks incorrect. Quote the spec line for each finding. Under 400 words.

### 5. Aggregate

Present the two reports verbatim under `## Standards` and `## Spec` headings.
Do **not** merge or rerank findings across axes. End with a one-line summary: total findings per axis, and the worst issue within each axis.

## Why two axes

A change can pass one axis and fail the other:
- Code that follows every standard but implements the wrong thing → **Standards pass, Spec fail.**
- Code that does exactly what the issue asked but breaks project conventions → **Spec pass, Standards fail.**

Reporting them separately stops one axis from masking the other.
