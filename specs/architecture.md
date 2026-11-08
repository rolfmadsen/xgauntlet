# Living Slice Specification: `architecture`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Slice Path**: `crates/xgauntlet-core/src/features/architecture/`  
**Type**: `Stateless Pipeline-Slice`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0010](../docs/adr/0010-adr-bound-architectural-exemptions-and-monotonic-ratchets.md), [ADR 0012](../docs/adr/0012-vertical-slice-boundaries-and-adr-lifecycle.md)

---

## 🎯 Purpose & Capability Boundary
The `architecture` slice enforces deterministic structural boundaries, monotonic complexity ratchets, and Vertical Slice Architecture (VSA) isolation across polyglot workspaces (`Rust`, `TypeScript/React`, `Java`, `Python`, `Go`):
- **Public Facade (`mod.rs` / `checker.rs`)**: `run_architecture_check(workspace_root, config)` $\to$ `Result<ArchitectureReport, ArchitectureError>`.
- **Gate A (`LayerDirectionViolation`)**: Enforces top-down dependency flow across the 4 universal layers (`app -> features -> core -> shared`).
- **Gate B (`CrossSliceImportViolation`)**: Auto-detects 1-level (`<slice_root>/<slice>/<file>`) and 2-level (`<slice_root>/<subdomain>/<slice>/<file>`) slice directories and rejects direct cross-slice imports unless explicitly exempted via `allow_imports`.
- **Gate C (`ForbiddenCoreImportViolation`)**: Prevents UI, CLI, or framework imports inside the pure `core` layer (`Shared Kernel`).
- **Monotonic Ratchets**: Enforces `max_file_lines`, `max_test_file_lines`, and ADR-bound `[[architecture.exemptions]]`.

## 🚫 Invariants
- Leaf files inside a slice folder (`models.rs`, `validator.rs`, `checker.rs`, etc.) are always internal to their owning slice and must never trigger `CrossSliceImportViolation` when importing sibling modules within the same slice.
- Every `[[architecture.exemptions]]` entry must reference an accepted ADR in `docs/adr/` and supports both exact relative paths and `/**` directory glob prefixes for `allow_imports`.
