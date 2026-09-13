---
name: to-tasks
description: Decompose features into verified task packages (tasks/<id>.md) with executable acceptance criteria (- [ ]) and Must NOT invariants.
---

# Task Decomposition (`to-tasks`)

Decompose high-level feature requirements into atomic, verified task packages adhering to OKF v0.2.

## Standard Task Package Structure
1. `# Task <number>: <Title>` with YAML frontmatter (`type: Task Package`, `status: active`, `tags: [...]`).
2. `## 🎯 Formål`: Concrete goal and operational boundaries.
3. `## 📋 Acceptance Criteria`: Executable `- [ ]` checkboxes with clear expected inputs and outputs.
4. `## 🚫 Must NOT`: Negative constraints and architectural invariants that must never be broken.
5. `## 📝 Revisions`: Append-only revision log for design changes.
6. `## 🧪 Verifikation`: Concrete CLI commands for testing and validation.

Validate with `cargo run -p xgauntlet-cli -- check-spec -t <id>`.
