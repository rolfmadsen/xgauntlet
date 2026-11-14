# Living Slice Specification: `tasks`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Slice Path**: `crates/xgauntlet-core/src/features/tasks/`  
**Type**: `Stateless Task Specification, Scaffolding & Lifecycle Slice`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0008](../docs/adr/0008-sovereign-agentic-sdlc-task-scoping-and-design-tokens.md), [ADR 0012](../docs/adr/0012-vertical-slice-boundaries-and-adr-lifecycle.md)

---

## 🎯 Purpose & Capability Boundary
The `tasks` slice owns OKF v0.2 task parsing, specification readiness validation (`check-spec`), Aristotelian glossary drift validation (`check-spec --context`), task telemetry, and task package scaffolding (`task new`):
- **Parser (`parser.rs` & `okf.rs`)**: Parses OKF v0.2 YAML frontmatter (`scope`, `test_refs`, `domain`/`subdomain`, `slice`, `spec_ref`) and task statuses (`DRAFT`, `ACTIVE`, `DONE`), and recursively discovers tasks across `tasks/**` (excluding `README.md`/`TEMPLATE.md`) with numeric sorting (`001`..`999` → `1000+`) and exact/prefix-prioritized ID lookup.
- **Validator (`validator.rs`)**: Enforces active task completeness, scope/test-ref boundaries, anti-monolith test file ceilings (`<= 1000` lines), scope-aware slice mapping, and `CONTEXT.md` glossary synchronization across `tasks/**`.
- **Scaffolder & Archiver (`lifecycle.rs`)**: Generates sequential task packages (`tasks/<id>-<slug>.md` or optional `--subdir`), computes global `max(id) + 1` across `tasks/**`, archives `DONE` tasks to `tasks/archive/` while updating relative links in `specs/**/*.md`, and maintains living slice specifications (`specs/<slice>.md` or `specs/<subdomain>/<slice>.md`) with two-way traceability links.

## 🚫 Invariants
- Completed tasks (`status: done`) remain 100% exempt from active slice-mapping changes for full backward compatibility.
- Cross-cutting tasks whose `scope` does not touch `architecture.slice_root` MUST NOT be forced to fabricate `specs/*.md` files.
- Task IDs MUST never be reused even when earlier tasks reside in `tasks/archive/` or other subdirectories, and active/draft tasks MUST NOT be archived.

## 📋 Relaterede Tasks
- [Task 057: Deterministic Slice Architecture Gates, Living Slice Specs & ADR Lifecycle](../tasks/archive/057-slice-architecture-gates-and-adr-lifecycle.md)
- [Task 069: Correct DDD/VSA Taxonomy, 1- and 2-Level Slice Isolation, and Dogfooding](../tasks/archive/069-ddd-slice-taxonomy-and-dogfooding.md)
- [Task 078: Scope-Aware Slice Spec Mapping & Two-Way Spec-Feature-Task Traceability](../tasks/archive/078-scope-aware-slice-spec-and-traceability.md)
- [Task 079: Recursive Task Subdirectories & Auto-Archiving in tasks/](../tasks/archive/079-recursive-task-subdirectories-and-archiving.md)

