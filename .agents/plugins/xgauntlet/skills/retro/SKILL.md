---
name: retro
description: Conduct a retrospective on a coding session to improve the agent's environment, tooling, linters, and AGENTS.md steering.
---

# Session Retrospective (`retro`)

Conduct a retrospective on a coding session to suggest improvements to the agent's environment, automated checks, and steering instructions.

---

## Steps

1. Read the primary sources for the session (conversation transcript or session git history).
2. Look for candidates for improvement in these 7 categories:

- **Navigation**: how easy was it for the agent to find the right files? Are there hidden dependencies? Would a navigation pointer help?
- **Automated checks**: are there checks that could catch errors the agent made? (Linters, types, `xgauntlet verify`, spec tests).
- **Coding standards**: should the reviewer agent be given a new rule to enforce? Should an existing rule be clarified or removed?
- **Global AGENTS.md**: are there steering instructions that should be moved to coding standards or automated checks instead? (Keep AGENTS.md lean).
- **Tool economy**: did the agent make expensive or redundant tool calls that could be streamlined?
- **No-ops**: look for instructions in steering files that don't modify agent behavior.
- **Information access**: look for opportunities to increase the agent's access to information (telemetry, logs, readonly data).

3. Present candidates to the user in order of severity.

---

## Reference Principles

### Implementation vs Review
All work goes through two distinct stages: implementation and review.
- The **implementation agent** has the most **context pressure**: responsible for exploration, writing code, and debugging failures.
- The **review agent** has the least **context pressure**: receives a diff, no exploration needed.
- This means the review agent should be responsible for imposing coding standards, not the implementation agent.

### Files & Roles
- **`AGENTS.md` / `CLAUDE.md`**: Pushed into the context window of any agent. Must be kept lean, reserved for front-loaded navigation pointers and core loop invariants.
- **`CODING_STANDARDS.md`**: Read during review, not during implementation.
- **`tasks/` & `spec.md`**: Executable contracts and criteria.
- **Skills**: On-demand deep knowledge and workflows loaded only when invoked.
