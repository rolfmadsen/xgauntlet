---
name: to-tasks
description: Break a plan, spec, or conversation into tracer-bullet task packages (tasks/<id>.md or tickets), each declaring blocking edges, acceptance criteria (- [ ]), and Must NOT invariants.
---

# To Tasks

Break a plan, spec, or conversation into a set of **tasks**: tracer-bullet vertical slices, each declaring the tasks that **block** it and verified acceptance criteria.

## Process

### 1. Gather context
Work from whatever is already in the conversation context, `spec.md`, or issue tracker.

### 2. Explore the codebase
Understand the current state of the code. Task titles and descriptions should use the project's domain glossary vocabulary (`CONTEXT.md`), and respect ADRs in `docs/adr/`.

Look for opportunities to prefactor the code to make the implementation easier: *"Make the change easy, then make the easy change."*

### 3. Draft vertical slices
Break the work into **tracer bullet** task packages:
- Each slice cuts a narrow but COMPLETE path through every layer: vertical, NOT a horizontal slice of one layer.
- A completed slice is demoable or verifiable on its own.
- Each slice is sized to fit in a single fresh context window.
- Any prefactoring should be done first.

Give each task its **blocking edges**: the other tasks that must complete before it can start. A task with no blockers can start immediately.

**Wide refactors are the exception to vertical slicing.** A **wide refactor** is one mechanical change (rename a column, retype a shared symbol) whose **blast radius** fans across the whole codebase, so a single edit breaks thousands of call sites at once and no vertical slice can land green. Don't force it into a tracer bullet; sequence it as **expand–contract**:
1. **Expand**: add the new form beside the old so nothing breaks.
2. **Migrate**: migrate call sites over in batches sized by blast radius (per package, per directory), each batch its own task blocked by the expand.
3. **Contract**: delete the old form once no caller remains, in a task blocked by every migrate batch.

### 4. Quiz the user
Present the proposed breakdown as a numbered list. For each task, show:
- **Title**: short descriptive name
- **Blocked by**: which other tasks (if any) must complete first
- **What it delivers**: the end-to-end behaviour this task makes work

Ask the user:
- Does the granularity feel right? (too coarse / too fine)
- Are the blocking edges correct: does each task only depend on tasks that genuinely gate it?
- Should any tasks be merged or split further?

Iterate until the user approves the breakdown.

### 5. Publish the task packages
In an xGauntlet repository, write one file per task package under `tasks/<number>-<slug>.md` in dependency order:

```markdown
---
type: Task Package
status: active
id: <number>
tags: [<feature-tags>]
---

# Task <number>: <Title>

## 🎯 Formål
The end-to-end behaviour this task makes work, from the user's perspective, not layer-by-layer implementation.

## 📋 Acceptance Criteria
- [ ] Criterion 1: Concrete observable input/output
- [ ] Criterion 2: Verified behavior

## 🚫 Must NOT
Negative constraints and architectural invariants that must never be broken under any circumstances.

## 📝 Revisions
Append-only log of revisions made during execution.

## 🧪 Verifikation
Concrete verification commands:
`cargo run -p xgauntlet-cli -- check-spec -t <number>`
```

Work the **frontier**: any task whose blockers are all done.
