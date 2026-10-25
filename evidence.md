# Verification Report

**Task ID**: `048-canonical-antigravity-hook-in-migration`  
**Task Title**: Task 048: Canonical Antigravity Hook in Migration  
**Verdict**: `PARTIAL`  
**Execution Origin**: `LOCAL`  
**Source Manifest Digest**: `672f234760a4b83326bf5e8ebbe18e61fe71e814fcd7ffc7179acb5da5c954a9`  
**Timestamp**: `2026-10-25T19:48:47Z`  
**Head**: `9e9ed74`  
**Commit**: `9e9ed74`  

## Acceptance Criteria

- [x] **AC1 (Single source of truth)**: Én offentlig konstant for Antigravity PreInvocation-kommandoen (`xgauntlet telemetry --format antigravity-hook`) i `adapters/antigravity`, genbrugt af adapter, scaffold-template og migration.
- [x] **AC2 (Korrekt injektion)**: Migration af `hooks.json` uden telemetry-hook indsætter præcis den kanoniske kommando.
- [x] **AC3 (Normalisering)**: Migration af `hooks.json` med `--harness antigravity-hook` (eller `--harness antigravity`) erstatter den med den kanoniske kommando, rapporterer ændringen og er idempotent ved anden kørsel.
- [x] **AC4 (Ingen tavs overskrivning)**: Findes både `agent-gauntlet-gatekeeper` og `xgauntlet`, bevares den kanoniske gruppes hooks (merge uden dubletter), og ændringslisten nævner sammenfletningen.
- [x] **AC5 (doctor --fix er ærlig)**: Fejler migrationen under `doctor --fix`, rapporteres fejlen i doctor-resultatet i stedet for at blive kasseret.
- [x] **AC6 (Fixture-korrektion)**: `doctor_diagnostics_test.rs` bruger ikke længere den defekte streng som "korrekt" fixture.

---

## Verification Checks

| Check Name | Status | Exit Code | Duration (s) |
|---|---|---|---|
| `architecture` | `PASSED` | `0` | `0.072s` |
| `format` | `PASSED` | `0` | `0.212s` |
| `lint` | `PASSED` | `0` | `0.116s` |
| `types` | `PASSED` | `0` | `0.126s` |
| `unit` | `PASSED` | `0` | `21.049s` |
| `npm` | `PASSED` | `0` | `0.824s` |
| `invariants` | `PASSED` | `0` | `0.068s` |
| `mutation-testing-gauntlet` | `FAILED` | `101` | `0.014s` |

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
| `sess-2604f95e` | `TDD Craftsman` | `REFACTOR` | `2604f95` | `2026-10-25T17:24:43Z` | `VALID` |
| `sess-4ead69e2` | `Software Engineer` | `DONE` | `4ead69e` | `2026-10-25T17:26:10Z` | `VALID` |
| `sess-94e8a286` | `TDD Craftsman` | `RED` | `94e8a28` | `2026-10-25T19:19:28Z` | `VALID` |
| `sess-fc38f6bb` | `TDD Craftsman` | `GREEN` | `fc38f6b` | `2026-10-25T19:22:06Z` | `VALID` |
| `sess-499be1ee` | `TDD Craftsman` | `REFACTOR` | `499be1e` | `2026-10-25T19:22:38Z` | `VALID` |
| `sess-dc9e26a3` | `Software Engineer` | `DONE` | `dc9e26a` | `2026-10-25T19:26:06Z` | `VALID` |
| `sess-2f2e2454` | `TDD Craftsman` | `RED` | `2f2e245` | `2026-10-25T19:34:52Z` | `VALID` |
| `sess-30e4ad7d` | `TDD Craftsman` | `GREEN` | `30e4ad7` | `2026-10-25T19:41:18Z` | `VALID` |
| `sess-9e9ed74b` | `TDD Craftsman` | `REFACTOR` | `9e9ed74` | `2026-10-25T19:43:16Z` | `VALID` |

---

### 🧪 Test Provenance & Born-Green Verification

| Test Identifier | Status | Baseline Failure Digest | Verified At |
|---|---|---|---|
| `test_migration_injects_canonical_telemetry_format` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_migration_normalizes_defective_harness_hook_and_is_idempotent` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_migration_normalizes_legacy_harness_antigravity` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_migration_merges_simultaneous_legacy_and_modern_groups_without_data_loss` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_doctor_fix_reports_migration_errors_truthfully` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_preflight_leaves_working_tree_untouched_during_test_run` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_preflight_runs_in_isolated_worktree_and_cleans_up` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_preflight_does_not_resurrect_deleted_production_file` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_preflight_baseline_excludes_untracked_production_files` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_preflight_fails_closed_without_head` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
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
| `test_canonical_pre_invocation_cmd_single_source_of_truth` | `InversePreflightVerified` | `e63e0e0ec71fb4c6...` | `2026-10-25T19:39:49Z` |

---
