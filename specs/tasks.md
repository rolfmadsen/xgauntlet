# Living Feature Specification: `tasks`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Feature Path**: `crates/xgauntlet-core/src/features/tasks/`  
**Type**: `Stateless Task Specification, Scaffolding & Lifecycle Feature`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0008](../docs/adr/0008-sovereign-agentic-sdlc-task-scoping-and-design-tokens.md), [ADR 0013](../docs/adr/0013-feature-architecture-boundaries-and-living-specs.md)

---

## 🎯 Purpose & Capability Boundary
The `tasks` feature owns OKF v0.2 task parsing, specification readiness validation (`check-spec`), Aristotelian glossary drift validation (`check-spec --context`), task telemetry, and task package scaffolding (`task new`):
- **Parser (`parser.rs` & `okf.rs`)**: Parses OKF v0.2 YAML frontmatter (`scope`, `test_refs`, `domain`/`subdomain`, `feature` with legacy `slice` fallback, `spec_ref`) and task statuses (`DRAFT`, `ACTIVE`, `DONE`), recursively discovers tasks across `tasks/**` (excluding `README.md`/`TEMPLATE.md`) with numeric sorting (`001`..`999` → `1000+`), supports worktree-local task binding (`.xgauntlet/active_task` via `xgauntlet task bind` / `unbind`) that overrides highest-numbered active fallback with fail-closed validation, and provides exact/prefix-prioritized ID lookup.
- **Validator (`validator.rs`)**: Enforces active task completeness, scope/test-ref boundaries, anti-monolith test file ceilings (`<= 1000` lines), scope-aware feature mapping (exempting bounded small bugfixes with `intent: bug`, `scope.len() <= 3`, and no wildcards unless explicit `feature`/`spec_ref` is provided), and `CONTEXT.md` glossary synchronization across `tasks/**`.
- **Scaffolder & Archiver (`lifecycle.rs`)**: Generates sequential task packages (`tasks/<id>-<slug>.md` or optional `--subdir`) with one-liner `--scope`/`--test` support (`task new`), computes global `max(id) + 1` across `tasks/**`, archives terminal tasks (`DONE`, `COMPLETED`, `DEPRECATED`, `SUPERSEDED`, `REJECTED`) to `tasks/archive/` while updating relative links in `specs/**/*.md`, and maintains living feature specifications (`specs/<feature>.md` or `specs/<subdomain>/<feature>.md`) with two-way traceability links.

## 🚫 Invariants
- Completed tasks (`status: done`) remain 100% exempt from active feature-mapping changes for full backward compatibility.
- Cross-cutting tasks whose `scope` does not touch `architecture.feature_root` MUST NOT be forced to fabricate `specs/*.md` files.
- Task IDs MUST never be reused even when earlier tasks reside in `tasks/archive/` or other subdirectories, and non-terminal tasks (`DRAFT`, `ACTIVE`, `IN_PROGRESS`, `WIP`, `TODO`, `REOPENED`, `UNKNOWN`) MUST NOT be archived.

## 📋 Relaterede Tasks
- [Task 057: Deterministic Slice Architecture Gates, Living Slice Specs & ADR Lifecycle](../tasks/archive/057-slice-architecture-gates-and-adr-lifecycle.md)
- [Task 069: Correct DDD/VSA Taxonomy, 1- and 2-Level Slice Isolation, and Dogfooding](../tasks/archive/069-ddd-slice-taxonomy-and-dogfooding.md)
- [Task 078: Scope-Aware Slice Spec Mapping & Two-Way Spec-Feature-Task Traceability](../tasks/archive/078-scope-aware-slice-spec-and-traceability.md)
- [Task 079: Recursive Task Subdirectories & Auto-Archiving in tasks/](../tasks/archive/079-recursive-task-subdirectories-and-archiving.md)
- [Task 087: Archive Terminal Task Statuses (Deprecated, Superseded, Rejected)](../tasks/archive/087-archive-terminal-task-statuses.md)
- [Task 088: Worktree-Isolated Multi-Agent & Subagent Supervision](../tasks/archive/088-worktree-isolated-multi-agent-supervision.md)
- [Task 093: Multi-Agent Worktree Workflow, Coexistence Policy & Runtime/Evidence Hygiene](../tasks/093-multi-agent-worktree-and-runtime-hygiene.md)

