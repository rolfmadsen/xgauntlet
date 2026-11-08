# Living Slice Specification: `release`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Slice Path**: `crates/xgauntlet-core/src/features/release/`  
**Type**: `Stateless Pipeline-Slice`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0003](../docs/adr/0003-local-only-git-checkpoints.md), [ADR 0005](../docs/adr/0005-two-tier-trust-and-sigstore-attestation.md), [ADR 0012](../docs/adr/0012-vertical-slice-boundaries-and-adr-lifecycle.md)

---

## 🎯 Purpose & Capability Boundary
The `release` slice enforces mechanical release readiness (`check-release`) and transactional SemVer version bumping (`release bump` / `release prepare`):
- **Public Facade (`mod.rs` / `engine.rs`)**: `check_release_readiness(options)` $\to$ `Result<ReleaseReadinessReport, ReleaseError>` and `update_manifest_versions` / `prepare_changelog_release`.
- **Manifest Harmony**: Synchronizes and verifies version parity across `Cargo.toml`, crate manifests, `package.json`, `packages/cli/package.json`, `.agents/plugins/xgauntlet/plugin.json`, `.agents/plugins/xgauntlet/policy.json`, and embedded `gauntlet_policy.wasm`.
- **Changelog & ADR Coverage**: Verifies Keep a Changelog section `[<version>]` and ensures all `accepted` ADRs in `docs/adr/` are referenced in `README.md` or `spec.md`.
- **NPM README Parity & Zero-Drift Evidence**: Verifies `packages/cli/README.md` matches root `README.md` and `verification-report.json` matches the current workspace manifest with zero drift.

## 🚫 Invariants
- Never invokes destructive Git commands (`git reset --hard`, `git clean -f`) during release rollback; uses isolated disk-staging backup.
- Never executes `git push` automatically.
