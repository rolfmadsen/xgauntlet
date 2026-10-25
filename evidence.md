# Verification Report

**Task ID**: `026-in-repo-migration-engine-and-doctor-auto-remediation`  
**Task Title**: Task 026: In-Repo Migration Engine and Doctor Auto-Remediation  
**Verdict**: `PARTIAL`  
**Execution Origin**: `LOCAL`  
**Source Manifest Digest**: `5559c9f7318ea321742822e1180338c88f93e6f8c10308fd146e18d43db35ea9`  
**Timestamp**: `2026-10-25T17:01:29Z`  
**Head**: `fdbda2a`  
**Commit**: `fdbda2a`  

## Acceptance Criteria

- [x] **Dybde-inspektion i `xgauntlet doctor`**:
- [x] Udvid `governance_agent_harness` i doctor-motoren til at parse og inspicere indholdet af `.agents/hooks.json`.
- [x] Flag `[WARN]` hvis `.agents/hooks.json` mangler `PreInvocation` telemetri-hook eller benytter det forældede `"agent-gauntlet-gatekeeper"` namespace.
- [x] Flag `[WARN]` hvis `gauntlet.toml` refererer forældet `evidence_file = "evidence.json"` i stedet for `"verification-report.json"`.
- [x] Give actionable remediation-meddelelse: `Action: Run 'xgauntlet migrate' (or 'xgauntlet doctor --fix')`.
- [x] **Kirurgisk Migrations-motor (`xgauntlet migrate`)**:
- [x] Etablere CLI subcommand `xgauntlet migrate` i `crates/xgauntlet-cli` og kerne-logik i `crates/xgauntlet-core/src/features/migration/`.
- [x] Kirurgisk opdatering af `.agents/hooks.json`: Omdøb `"agent-gauntlet-gatekeeper"` til `"xgauntlet"` og injicer `"PreInvocation"` hook uden at overskrive brugerens egne `PreToolUse` matcher-regler.
- [x] Kirurgisk opdatering af `gauntlet.toml`: Modernisér `evidence_file` til `"verification-report.json"` uden at berøre eksisterende bruger-definerede `[[layers]]`.
- [x] Automatisk scaffolding af manglende template-filer (f.eks. `docs/adr/template.md`).
- [x] Flag-understøttelse: `--dry-run` (viser planlagte ændringer uden diskskrivning), `--workspace <path>` og `--json`.
- [x] **Auto-remediation i `xgauntlet doctor --fix`**:
- [x] Tilføje `--fix` flag til `xgauntlet doctor`, som automatisk kalder migrationsmotoren for identificerede governance-advarsler.
- [x] **Runtime Bagudkompatibilitet**:
- [x] `xgauntlet hook` accepterer fortsat legacy namespace `"agent-gauntlet-gatekeeper"` i `.agents/hooks.json` med en opgraderings-anbefaling i stderr.
- [x] **Multi-layer Verifikation**:
- [x] Tilføje omfattende integrationstests i `crates/xgauntlet-core/tests/migration_engine_test.rs` der beviser in-place migrering mod ældre projektopsætninger.
- [x] Verificere at samtlige tests og invariant-tjek passerer (`cargo test --workspace`, `cargo clippy`, `xgauntlet check-spec`).

---

## Verification Checks

| Check Name | Status | Exit Code | Duration (s) |
|---|---|---|---|
| `architecture` | `PASSED` | `0` | `0.071s` |
| `format` | `PASSED` | `0` | `0.238s` |
| `lint` | `PASSED` | `0` | `0.131s` |
| `types` | `PASSED` | `0` | `0.104s` |
| `unit` | `PASSED` | `0` | `19.561s` |
| `npm` | `PASSED` | `0` | `0.831s` |
| `invariants` | `PASSED` | `0` | `0.072s` |
| `mutation-testing-gauntlet` | `FAILED` | `101` | `0.010s` |

---

### 📜 Multi-Session Audit Lineage

| Session | Rolle | Fase | Commit | Tidsstempel | Status |
|---|---|---|---|---|---|
| `sess-0e5ff17b` | `System Architect` | `SPEC` | `0e5ff17` | `2026-10-25T14:51:15Z` | `VALID` |
| `sess-589985b0` | `TDD Craftsman` | `RED` | `589985b` | `2026-10-25T15:00:27Z` | `VALID` |
| `sess-89220d3e` | `TDD Craftsman` | `GREEN` | `89220d3` | `2026-10-25T15:05:09Z` | `VALID` |
| `sess-0de064f4` | `TDD Craftsman` | `REFACTOR` | `0de064f` | `2026-10-25T15:06:38Z` | `VALID` |
| `sess-26bd5b99` | `Software Engineer` | `DONE` | `26bd5b9` | `2026-10-25T14:16:13Z` | `VALID` |
| `sess-4a8a2566` | `System Architect` | `SPEC` | `4a8a256` | `2026-10-25T14:43:26Z` | `VALID` |
| `sess-4a8a2566` | `TDD Craftsman` | `RED` | `c1b2b2d` | `2026-10-25T14:50:03Z` | `VALID` |
| `sess-7793d1b7` | `TDD Craftsman` | `GREEN` | `7793d1b` | `2026-10-25T15:08:20Z` | `VALID` |
| `sess-58ba84b2` | `TDD Craftsman` | `REFACTOR` | `58ba84b` | `2026-10-25T15:08:57Z` | `VALID` |
| `sess-cfddffd7` | `Software Engineer` | `DONE` | `cfddffd` | `2026-10-25T15:12:16Z` | `VALID` |
| `sess-59e83396` | `System Architect` | `SPEC` | `59e8339` | `2026-10-25T15:21:43Z` | `VALID` |
| `sess-e36bbb3c` | `TDD Craftsman` | `RED` | `e36bbb3` | `2026-10-25T15:22:47Z` | `VALID` |
| `sess-3f63030e` | `TDD Craftsman` | `GREEN` | `3f63030` | `2026-10-25T15:28:34Z` | `VALID` |
| `sess-21582552` | `TDD Craftsman` | `REFACTOR` | `2158255` | `2026-10-25T15:32:13Z` | `VALID` |
| `sess-a70236ba` | `Software Engineer` | `DONE` | `a70236b` | `2026-10-25T15:33:55Z` | `VALID` |
| `sess-f50fd4dc` | `Software Engineer` | `DONE` | `f50fd4d` | `2026-10-25T15:34:59Z` | `VALID` |
| `sess-1d7f44ec` | `TDD Craftsman` | `RED` | `1d7f44e` | `2026-10-25T16:01:02Z` | `VALID` |
| `sess-9dc6319c` | `TDD Craftsman` | `GREEN` | `9dc6319` | `2026-10-25T16:06:50Z` | `VALID` |
| `sess-2c9ed01a` | `TDD Craftsman` | `REFACTOR` | `2c9ed01` | `2026-10-25T16:07:41Z` | `VALID` |
| `sess-fc17224e` | `Software Engineer` | `DONE` | `fc17224` | `2026-10-25T16:10:58Z` | `VALID` |
| `sess-8050b79d` | `System Architect` | `SPEC` | `8050b79` | `2026-10-25T16:19:49Z` | `VALID` |
| `sess-67315fa5` | `TDD Craftsman` | `RED` | `67315fa` | `2026-10-25T16:21:56Z` | `VALID` |
| `sess-8c983637` | `TDD Craftsman` | `GREEN` | `8c98363` | `2026-10-25T16:27:56Z` | `VALID` |
| `sess-24ac25ac` | `TDD Craftsman` | `REFACTOR` | `24ac25a` | `2026-10-25T16:28:37Z` | `VALID` |
| `sess-1884ade7` | `Software Engineer` | `DONE` | `1884ade` | `2026-10-25T16:29:59Z` | `VALID` |
| `sess-05fd66fd` | `System Architect` | `SPEC` | `05fd66f` | `2026-10-25T16:41:15Z` | `VALID` |
| `sess-de771be3` | `TDD Craftsman` | `RED` | `de771be` | `2026-10-25T16:47:22Z` | `VALID` |
| `sess-0c135645` | `TDD Craftsman` | `GREEN` | `0c13564` | `2026-10-25T16:55:51Z` | `VALID` |
| `sess-fdbda2ae` | `TDD Craftsman` | `REFACTOR` | `fdbda2a` | `2026-10-25T16:59:02Z` | `VALID` |
| `sess-aa21fa7f` | `Software Engineer` | `DONE` | `aa21fa7` | `2026-10-25T17:01:29Z` | `VALID` |
| `sess-88b26411` | `TDD Craftsman` | `RED` | `88b2641` | `2026-10-25T17:19:44Z` | `VALID` |
| `sess-e40eb6c5` | `TDD Craftsman` | `GREEN` | `e40eb6c` | `2026-10-25T17:23:59Z` | `VALID` |

---

### 🧪 Test Provenance & Born-Green Verification

| Test Identifier | Status | Baseline Failure Digest | Verified At |
|---|---|---|---|
| `test_doctor_warns_on_legacy_hooks_namespace_and_missing_preinvocation` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_doctor_warns_on_legacy_evidence_file_in_gauntlet_toml` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_doctor_passes_when_hooks_and_evidence_are_modern` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_doctor_fix_flag_auto_remediates_governance_issues` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_migration_engine_dry_run_does_not_modify_disk` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_migration_engine_modernizes_hooks_preserving_custom_matchers` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_migration_engine_modernizes_gauntlet_toml_preserving_custom_layers` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_migration_engine_scaffolds_missing_adr_template` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_runtime_backward_compatibility_with_legacy_hooks_namespace` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_session_lineage_trail_genesis_and_chaining` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_session_lineage_tamper_detection` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_verification_report_backward_compatibility` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_evidence_markdown_renders_lineage_table` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_checkpoint_engine_appends_lineage` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_adr_with_complete_okf_header_validates_ok` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_adr_without_generated_gives_generated_missing_error` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_adr_with_invalid_actor_gives_invalid_actor_error` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_adr_with_invalid_timestamp_gives_invalid_timestamp_error` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_adr_with_missing_verified_gives_warning_not_error` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_check_all_adrs_skips_template_and_readme` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_scaffold_template_includes_generated_and_verified` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_xgauntlet_own_adrs_validate_cleanly` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_models_and_conventions` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_compose_commit_message_variations` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_checkpoint_fails_without_active_task` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_spec_phase_checkpoint` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_red_phase_fails_when_test_passes` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_red_phase_succeeds_when_test_fails` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_autonomous_red_to_green_transition_checkpoint` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_green_phase_fails_when_test_fails` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_green_phase_succeeds_when_test_passes` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_refactor_phase_succeeds_when_green` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_skip_verify_override_bypasses_preflight` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_done_phase_succeeds_and_creates_chore_commit` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_cli_checkpoint_red_and_green_invariants` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_extract_test_identifiers_polyglot` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_synchronous_case` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_asynchronous_worker` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_inline_definition` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_detect_born_green_tests_in_git_diff` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_initial_requirement` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_sneaky_born_green_requirement` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_inverse_preflight_detects_tautological_test` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_inverse_preflight_verifies_legitimate_test` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_checkpoint_green_fails_closed_on_born_green_with_strict_provenance` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_collect_task_test_provenance_and_verification_report` | `InversePreflightVerified` | `c46f890b2d9b738e...` | `2026-10-25T15:07:56Z` |
| `test_doctor_warns_on_tsc_no_emit_in_composite_typescript_repo` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_doctor_passes_when_composite_typescript_uses_tsc_b` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_detect_composite_typescript_tsconfig_content` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_detect_composite_typescript_in_workspace` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_scaffold_init_composite_typescript_generates_tsc_b` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_scaffold_init_standard_typescript_generates_tsc_no_emit` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_architecture_config_parsing_from_toml` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_passes_on_compliant_workspace` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_fails_on_monolithic_production_file` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_fails_on_monolithic_test_file` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_honors_explicit_exemption` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_fails_if_exemption_ceiling_exceeded` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_catches_forbidden_tokens` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_catches_forbidden_imports` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_fails_when_exemption_lacks_adr` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_fails_when_exemption_adr_not_found` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_check_fails_when_exemption_adr_is_not_accepted` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |
| `test_architecture_config_parsing_from_toml_with_adr` | `RedCheckpointVerified` | `c436f3579eb4e5b6...` | `2026-10-25T18:19:44+01:00` |

---
