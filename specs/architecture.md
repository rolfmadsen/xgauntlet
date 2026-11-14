# Living Feature Specification: `architecture`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Feature Path**: `crates/xgauntlet-core/src/features/architecture/`  
**Type**: `Stateless Pipeline-Feature`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0010](../docs/adr/0010-adr-bound-architectural-exemptions-and-monotonic-ratchets.md), [ADR 0013](../docs/adr/0013-feature-architecture-boundaries-and-living-specs.md)

---

## 🎯 Purpose & Capability Boundary
The `architecture` feature enforces deterministic structural boundaries, monotonic complexity ratchets, and Package-by-Feature isolation across polyglot workspaces (`Rust`, `TypeScript/React`, `Java`, `Python`, `Go`):
- **Public Facade (`mod.rs` / `checker.rs`)**: `run_architecture_check(workspace_root, config)` $\to$ `Result<ArchitectureReport, ArchitectureError>`.
- **Gate A (`LayerDirectionViolation`)**: Enforces top-down dependency flow across the 4 universal layers (`app -> features -> core -> shared`).
- **Gate B (`CrossFeatureImportViolation`)**: Auto-detects 1-level (`<feature_root>/<feature>/<file>`) and 2-level (`<feature_root>/<subdomain>/<feature>/<file>`) feature directories and rejects direct cross-feature imports unless explicitly exempted via `allow_imports`.
- **Gate C (`ForbiddenCoreImportViolation`)**: Prevents UI, CLI, or framework imports inside the pure `core` layer (`Shared Kernel`).
- **Monotonic Ratchets**: Enforces `max_file_lines`, `max_test_file_lines`, and ADR-bound `[[architecture.exemptions]]`.

## 🚫 Invariants
- Leaf files inside a feature folder (`models.rs`, `validator.rs`, `checker.rs`, etc.) are always internal to their owning feature and must never trigger `CrossFeatureImportViolation` when importing sibling modules within the same feature.
- Every `[[architecture.exemptions]]` entry must reference an accepted ADR in `docs/adr/` and supports both exact relative paths and `/**` directory glob prefixes for `allow_imports`.

## 📋 Relaterede Tasks
- [Task 057: Deterministic Slice Architecture Gates, Living Slice Specs & ADR Lifecycle](../tasks/archive/057-slice-architecture-gates-and-adr-lifecycle.md)
- [Task 069: Correct DDD/VSA Taxonomy, 1- and 2-Level Slice Isolation, and Dogfooding](../tasks/archive/069-ddd-slice-taxonomy-and-dogfooding.md)
- [Task 081: Replace Slice Terminology with Feature Architecture](../tasks/archive/081-replace-slice-terminology-with-feature.md)
