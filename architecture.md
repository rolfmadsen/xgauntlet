# xGauntlet Domain & Architecture Diagram

🌐 **[Open Interactive Diagram (Pele Playground)](https://pele.run/playground#source=classDiagram%0A++++direction+TB%0A%0A++++%25%25+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A++++%25%25+%F0%9F%8F%9B%EF%B8%8F+xGauntlet+Core+Domain+Model+%28Conceptual+Architecture+-+CONTEXT.md%29%0A++++%25%25+Reflects+domain+entities+and+aggregates.+Operational+errors+and+CLI+flags%0A++++%25%25+are+excluded+in+conceptual+view+%28inspect+via+--level+detailed%29.%0A++++%25%25+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A%0A++++class+AdapterHookVerdict+%7B%0A++++++++%2Ballowed%3A+bool%0A++++++++%2Bdecision%3A+DecisionVerdict%0A++++++++%2Breason%3A+String%0A++++++++%2Breason_code%3A+Option%7Eu32%7E%0A++++%7D%0A%0A++++class+AdapterValidationResult+%7B%0A++++++++%2Bvalid%3A+bool%0A++++++++%2Bissues%3A+Vec%7EValidationIssue%7E%0A++++%7D%0A%0A++++class+AdrReadinessReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Badr_id%3A+String%0A++++++++%2Bdiagnostics%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Binspected_files%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+AntigravityAdapter+%7B%0A++++%7D%0A%0A++++class+ArchitectureConfig+%7B%0A++++++++%2Bmax_file_lines%3A+Option%7Eusize%7E%0A++++++++%2Bmax_test_file_lines%3A+Option%7Eusize%7E%0A++++++++%2Bforbidden_tokens%3A+Vec%7EString%7E%0A++++++++%2Bforbidden_imports%3A+Vec%7EString%7E%0A++++++++%2Blayers%3A+Vec%7EString%7E%0A++++++++%2Bfeature_root%3A+String%0A++++++++%2Bisolate_features%3A+bool%0A++++++++%2Bforbid_core_imports%3A+Vec%7EString%7E%0A++++++++%2Bexemptions%3A+Vec%7EArchitectureExemption%7E%0A++++%7D%0A%0A++++class+ArchitectureExemption+%7B%0A++++++++%2Bpath%3A+String%0A++++++++%2Breason%3A+String%0A++++++++%2Badr%3A+String%0A++++++++%2Bmax_lines%3A+Option%7Eusize%7E%0A++++++++%2Ballow_imports%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+ArchitectureFinding+%7B%0A++++++++%2Bfile_path%3A+String%0A++++++++%2Bline_number%3A+Option%7Eusize%7E%0A++++++++%2Bcategory%3A+ArchitectureFindingCategory%0A++++++++%2Bmessage%3A+String%0A++++++++%2Bremediation_hint%3A+String%0A++++%7D%0A%0A++++class+ArchitectureFindingCategory+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++FileExceedsMaxLines%0A++++++++TestFileExceedsMaxLines%0A++++++++ExemptionCeilingExceeded%0A++++++++ForbiddenTokenDetected%0A++++++++ForbiddenImportDetected%0A++++++++ExemptionMissingAdr%0A++++++++ExemptionAdrNotFound%0A++++++++ExemptionAdrNotAccepted%0A++++++++RatchetCeilingInflated%0A++++++++RatchetBaselineUnreadable%0A++++%7D%0A%0A++++class+ArchitectureReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Bfiles_scanned%3A+usize%0A++++++++%2Bfindings%3A+Vec%7EArchitectureFinding%7E%0A++++++++%2Bnotices%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+AttestationReport+%7B%0A++++++++%2Bbundle_path%3A+String%0A++++++++%2Bartifact_path%3A+String%0A++++++++%2Bartifact_digest%3A+String%0A++++++++%2Bverdict%3A+VerificationVerdict%0A++++++++%2Battestation_status%3A+AttestationStatus%0A++++++++%2Btrust_decision%3A+TrustDecision%0A++++++++%2Breasons%3A+Vec%7EString%7E%0A++++++++%2Bmatched_subject%3A+Option%7EString%7E%0A++++++++%2Bclaims%3A+BTreeMap%7EString%0A++++++++%2BString%7E%0A++++%7D%0A%0A++++class+AttestationStatus+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Absent%0A++++++++Valid%0A++++++++Invalid%0A++++%7D%0A%0A++++class+BlastRadiusReport+%7B%0A++++++++%2Btarget%3A+String%0A++++++++%2Baffected_nodes%3A+Vec%7EString%7E%0A++++++++%2Bdirect_dependents%3A+Vec%7EString%7E%0A++++++++%2Bindirect_dependents%3A+Vec%7EString%7E%0A++++++++%2Bmax_depth_reached%3A+usize%0A++++++++%2Btotal_affected_count%3A+usize%0A++++%7D%0A%0A++++class+CapabilityRequest+%7B%0A++++++++%2Baction_type%3A+ToolActionType%0A++++++++%2Braw_tool_name%3A+String%0A++++++++%2Btarget_resource%3A+String%0A++++++++%2Bpayload_json%3A+String%0A++++%7D%0A%0A++++class+CheckStatus+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Passed%0A++++++++Failed%0A++++++++Error%0A++++++++Skipped%0A++++%7D%0A%0A++++class+CheckSummary+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bstatus%3A+CheckStatus%0A++++++++%2Bexit_code%3A+i32%0A++++++++%2Bduration_seconds%3A+f64%0A++++++++%2Boptional%3A+bool%0A++++++++%2Bpassed%3A+bool%0A++++++++%2Blog_digest%3A+String%0A++++%7D%0A%0A++++class+CheckpointConfig+%7B%0A++++++++%2Benforce_provenance%3A+bool%0A++++%7D%0A%0A++++class+CheckpointPhase+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Spec%0A++++++++Red%0A++++++++Green%0A++++++++Refactor%0A++++++++Done%0A++++%7D%0A%0A++++class+CheckpointResult+%7B%0A++++++++%2Bphase%3A+CheckpointPhase%0A++++++++%2Btask_id%3A+String%0A++++++++%2Bcommit_oid%3A+Option%7EString%7E%0A++++++++%2Bcommit_message%3A+String%0A++++++++%2Bstaged_files%3A+Vec%7EString%7E%0A++++++++%2Bpreflight_passed%3A+bool%0A++++++++%2Bdetails%3A+Option%7EString%7E%0A++++++++%2Bhandoff%3A+Option%7ESessionHandoff%7E%0A++++++++%2Bnext_phase%3A+Option%7EString%7E%0A++++++++%2Bautonomous_transition%3A+bool%0A++++%7D%0A%0A++++class+ClaudeCodeAdapter+%7B%0A++++%7D%0A%0A++++class+CodexAdapter+%7B%0A++++%7D%0A%0A++++class+ConfigValidationIssue+%7B%0A++++++++%2Bseverity%3A+ValidationSeverity%0A++++++++%2Bcode%3A+String%0A++++++++%2Bmessage%3A+String%0A++++++++%2Bfield%3A+Option%7EString%7E%0A++++++++%2Bremediation%3A+Option%7EString%7E%0A++++%7D%0A%0A++++class+ConfigValidationReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Bissues%3A+Vec%7EConfigValidationIssue%7E%0A++++%7D%0A%0A++++class+DecisionVerdict+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Allow%0A++++++++Deny%0A++++++++Ask%0A++++++++ForceAsk%0A++++%7D%0A%0A++++class+DiagnosticFinding+%7B%0A++++++++%2Bfinding_type%3A+FindingType%0A++++++++%2Btool_name%3A+String%0A++++++++%2Bfile_path%3A+String%0A++++++++%2Bline_number%3A+Option%7Eusize%7E%0A++++++++%2Bcolumn_number%3A+Option%7Eusize%7E%0A++++++++%2Bmessage%3A+String%0A++++++++%2Bremediation_hint%3A+String%0A++++++++%2Braw_context%3A+String%0A++++%7D%0A%0A++++class+DiagnosticReport+%7B%0A++++++++%2Blayer_name%3A+String%0A++++++++%2Bpassed%3A+bool%0A++++++++%2Bexit_code%3A+i32%0A++++++++%2Bfindings%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Bsummary%3A+String%0A++++%7D%0A%0A++++class+DiscoveredHarness+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bdetected%3A+bool%0A++++++++%2Bconfig_dir%3A+PathBuf%0A++++++++%2Bplugin_installed%3A+bool%0A++++++++%2Bplugin_dir%3A+PathBuf%0A++++%7D%0A%0A++++class+DoctorCategory+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Host%0A++++++++Git%0A++++++++Governance%0A++++++++Toolchains%0A++++++++Engine%0A++++++++Harnesses%0A++++%7D%0A%0A++++class+DoctorCheckItem+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bcategory%3A+DoctorCategory%0A++++++++%2Bstatus%3A+DoctorCheckStatus%0A++++++++%2Bmessage%3A+String%0A++++++++%2Bdetail%3A+Option%7EString%7E%0A++++++++%2Bremediation%3A+Option%7EString%7E%0A++++++++%2Bduration_ms%3A+u64%0A++++%7D%0A%0A++++class+DoctorCheckStatus+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Pass%0A++++++++Warn%0A++++++++Fail%0A++++++++Info%0A++++%7D%0A%0A++++class+DoctorReport+%7B%0A++++++++%2Bworkspace%3A+PathBuf%0A++++++++%2Btimestamp%3A+String%0A++++++++%2Bengine_version%3A+String%0A++++++++%2Btarget_platform%3A+String%0A++++++++%2Bchecks%3A+Vec%7EDoctorCheckItem%7E%0A++++++++%2Bpassed_count%3A+usize%0A++++++++%2Bwarn_count%3A+usize%0A++++++++%2Bfailed_count%3A+usize%0A++++++++%2Binfo_count%3A+usize%0A++++++++%2Bduration_total_ms%3A+u64%0A++++++++%2Bverdict%3A+DoctorVerdict%0A++++%7D%0A%0A++++class+DoctorVerdict+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Healthy%0A++++++++Degraded%0A++++++++Critical%0A++++%7D%0A%0A++++class+EdgeType+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Imports%0A++++++++Calls%0A++++++++DependsOn%0A++++++++Defines%0A++++++++Exports%0A++++++++Implements%0A++++%7D%0A%0A++++class+EnforcementContext+%7B%0A++++++++%2Bworkspace_id%3A+String%0A++++++++%2Bhas_active_task%3A+bool%0A++++++++%2Bactive_task_id%3A+String%0A++++++++%2Bread_only%3A+bool%0A++++++++%2Ballowed_scopes%3A+Vec%7EString%7E%0A++++++++%2Ballowed_test_targets%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+ExecutionMetadata+%7B%0A++++++++%2Bstarted_at%3A+String%0A++++++++%2Bfinished_at%3A+String%0A++++++++%2Btotal_duration_seconds%3A+f64%0A++++++++%2Benvironment%3A+BTreeMap%7EString%0A++++++++%2BString%7E%0A++++%7D%0A%0A++++class+ExecutionOrigin+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Local%0A++++++++CiUnprivileged%0A++++++++CiProtected%0A++++%7D%0A%0A++++class+FindingType+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++LintError%0A++++++++TypeMismatch%0A++++++++TestFailure%0A++++++++PropertyViolation%0A++++++++MutantSurvived%0A++++++++DriftDetected%0A++++++++InvariantViolation%0A++++++++GeneralError%0A++++++++Warning%0A++++%7D%0A%0A++++class+GauntletConfig+%7B%0A++++++++%2Bstack%3A+String%0A++++++++%2Bsave_evidence%3A+bool%0A++++++++%2Bevidence_file%3A+String%0A++++++++%2Bevidence_markdown_file%3A+String%0A++++++++%2Bpaths%3A+PathsConfig%0A++++++++%2Barchitecture%3A+ArchitectureConfig%0A++++++++%2Bcheckpoint%3A+CheckpointConfig%0A++++++++%2Bmulti_agent%3A+MultiAgentConfig%0A++++++++%2Blayers%3A+Vec%7ELayerConfig%7E%0A++++%7D%0A%0A++++class+GauntletReport+%7B%0A++++++++%2Bsuccess%3A+bool%0A++++++++%2Blayers%3A+Vec%7ELayerResult%7E%0A++++++++%2Btotal_duration_seconds%3A+f64%0A++++%7D%0A%0A++++class+GlossaryDriftReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Borphan_models%3A+Vec%7EString%7E%0A++++++++%2Bphantom_terms%3A+Vec%7EString%7E%0A++++++++%2Bavoid_violations%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Bdiagnostics%3A+Vec%7EDiagnosticFinding%7E%0A++++%7D%0A%0A++++class+HarnessAdapter+%7B%0A++++++++%3C%3Cinterface%3E%3E%0A++++%7D%0A%0A++++class+LayerConfig+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bcommand%3A+Vec%7EString%7E%0A++++++++%2Boptional%3A+bool%0A++++++++%2Btimeout_seconds%3A+f64%0A++++++++%2Bfail_on_output%3A+bool%0A++++%7D%0A%0A++++class+LayerDefinition+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bcommand%3A+Vec%7EString%7E%0A++++++++%2Boptional%3A+bool%0A++++++++%2Btimeout_seconds%3A+f64%0A++++++++%2Bfail_on_output%3A+bool%0A++++++++%2Brequirement%3A+LayerRequirement%0A++++%7D%0A%0A++++class+LayerExecutionStatus+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Passed%0A++++++++Failed%0A++++++++Skipped%0A++++++++Unavailable%0A++++++++TimedOut%0A++++++++Error%0A++++%7D%0A%0A++++class+LayerRequirement+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Required%0A++++++++Optional%0A++++%7D%0A%0A++++class+LayerResult+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bexit_code%3A+i32%0A++++++++%2Bpassed%3A+bool%0A++++++++%2Boutput%3A+String%0A++++++++%2Bstatus%3A+LayerExecutionStatus%0A++++++++%2Bduration_seconds%3A+f64%0A++++++++%2Brequirement%3A+LayerRequirement%0A++++%7D%0A%0A++++class+MigrationItem+%7B%0A++++++++%2Bfile%3A+PathBuf%0A++++++++%2Bdescription%3A+String%0A++++++++%2Bchanges%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+MigrationReport+%7B%0A++++++++%2Bis_success%3A+bool%0A++++++++%2Bitems%3A+Vec%7EMigrationItem%7E%0A++++++++%2Bfiles_modified%3A+Vec%7EPathBuf%7E%0A++++%7D%0A%0A++++class+MistralAdapter+%7B%0A++++%7D%0A%0A++++class+MultiAgentConfig+%7B%0A++++++++%2Ballow_coordinated_inherit%3A+bool%0A++++++++%2Bshare_cargo_target_dir%3A+bool%0A++++%7D%0A%0A++++class+NodeType+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Package%0A++++++++Module%0A++++++++File%0A++++++++Function%0A++++++++Struct%0A++++++++Interface%0A++++++++Class%0A++++++++Unknown%0A++++%7D%0A%0A++++class+NormalizedToolCall+%7B%0A++++++++%2Baction_type%3A+ToolActionType%0A++++++++%2Btarget_resource%3A+String%0A++++++++%2Braw_tool_name%3A+String%0A++++++++%2Bpayload%3A+Value%0A++++%7D%0A%0A++++class+PathsConfig+%7B%0A++++++++%2Btasks_dir%3A+String%0A++++++++%2Bspec_file%3A+String%0A++++++++%2Bcontext_file%3A+String%0A++++++++%2Bcoding_standards_file%3A+String%0A++++++++%2Bdiagram_file%3A+String%0A++++%7D%0A%0A++++class+PluginInstallReport+%7B%0A++++++++%2Btargets%3A+Vec%7EPluginTargetInstallReport%7E%0A++++++++%2Bdiscovered_harnesses%3A+Vec%7EDiscoveredHarness%7E%0A++++++++%2Bsuccess%3A+bool%0A++++++++%2Bis_dry_run%3A+bool%0A++++%7D%0A%0A++++class+PluginInstallStatus+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Installed%0A++++++++UpToDate%0A++++++++SkippedExisting%0A++++++++DryRunSimulated%0A++++%7D%0A%0A++++class+PluginTargetInstallReport+%7B%0A++++++++%2Bharness%3A+String%0A++++++++%2Btarget_dir%3A+PathBuf%0A++++++++%2Bstatus%3A+PluginInstallStatus%0A++++++++%2Bfiles_written%3A+Vec%7EPathBuf%7E%0A++++++++%2Bfiles_skipped%3A+Vec%7EPathBuf%7E%0A++++%7D%0A%0A++++class+PolicyDecision+%7B%0A++++++++%2Bverdict%3A+DecisionVerdict%0A++++++++%2Breason%3A+String%0A++++++++%2Breason_code%3A+u32%0A++++%7D%0A%0A++++class+ReleaseFinding+%7B%0A++++++++%2Bfile_path%3A+String%0A++++++++%2Bcategory%3A+ReleaseFindingCategory%0A++++++++%2Bmessage%3A+String%0A++++++++%2Bremediation_hint%3A+String%0A++++%7D%0A%0A++++class+ReleaseFindingCategory+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++ConfigVersionMismatch%0A++++++++MissingChangelog%0A++++++++ChangelogVersionMismatch%0A++++++++UnreferencedAdr%0A++++++++MissingManifest%0A++++++++DirtyWorktree%0A++++++++UnformattedCode%0A++++++++GeneralError%0A++++%7D%0A%0A++++class+ReleaseReadinessReport+%7B%0A++++++++%2Bis_ready%3A+bool%0A++++++++%2Bdeclared_version%3A+String%0A++++++++%2Bversions_by_source%3A+BTreeMap%7EString%0A++++++++%2BString%7E%0A++++++++%2Binspected_files%3A+Vec%7EString%7E%0A++++++++%2Bchangelog_versions%3A+Vec%7EString%7E%0A++++++++%2Bunreferenced_adrs%3A+Vec%7EString%7E%0A++++++++%2Bdiagnostics%3A+Vec%7EReleaseFinding%7E%0A++++++++%2Bduration_ms%3A+u64%0A++++%7D%0A%0A++++class+ScaffoldAction+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Created%0A++++++++Skipped%0A++++++++Overwritten%0A++++++++WouldCreate%0A++++++++WouldSkip%0A++++++++WouldOverwrite%0A++++%7D%0A%0A++++class+ScaffoldFileReport+%7B%0A++++++++%2Bpath%3A+String%0A++++++++%2Baction%3A+ScaffoldAction%0A++++++++%2Breason%3A+Option%7EString%7E%0A++++%7D%0A%0A++++class+ScaffoldResult+%7B%0A++++++++%2Bworkspace%3A+PathBuf%0A++++++++%2Bstack%3A+String%0A++++++++%2Bfiles%3A+Vec%7EScaffoldFileReport%7E%0A++++++++%2Bcreated_count%3A+usize%0A++++++++%2Bskipped_count%3A+usize%0A++++++++%2Boverwritten_count%3A+usize%0A++++++++%2Bis_success%3A+bool%0A++++%7D%0A%0A++++class+SessionHandoff+%7B%0A++++++++%2Btask_id%3A+String%0A++++++++%2Bbranch%3A+String%0A++++++++%2Bcommit_oid%3A+String%0A++++++++%2Bcommits_ahead%3A+usize%0A++++++++%2Bis_clean%3A+bool%0A++++++++%2Bnext_role%3A+String%0A++++++++%2Bformatted_card%3A+String%0A++++%7D%0A%0A++++class+SpecReadinessReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Btask_id%3A+String%0A++++++++%2Bdiagnostics%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Binspected_files%3A+Vec%7EString%7E%0A++++++++%2Bmust_not_rules%3A+Vec%7EString%7E%0A++++++++%2Bacceptance_criteria%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+StackProfile+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bdescription%3A+String%0A++++++++%2Bdetection_files%3A+Vec%7EString%7E%0A++++++++%2Bdefault_layers%3A+Vec%7ELayerConfig%7E%0A++++%7D%0A%0A++++class+TaskContract+%7B%0A++++++++%2Btask_id%3A+String%0A++++++++%2Btask_title%3A+String%0A++++++++%2Btask_digest%3A+String%0A++++++++%2Bacceptance_criteria%3A+Vec%7EString%7E%0A++++++++%2Bunresolved_criteria%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+TaskPackageInfo+%7B%0A++++++++%2Btask_id%3A+String%0A++++++++%2Btitle%3A+String%0A++++++++%2Bstatus%3A+TaskStatus%0A++++++++%2Bintent%3A+Option%7EString%7E%0A++++++++%2Bpurpose%3A+String%0A++++++++%2Bacceptance_criteria%3A+Vec%7EString%7E%0A++++++++%2Bunresolved_criteria%3A+Vec%7EString%7E%0A++++++++%2Bmust_not%3A+Vec%7EString%7E%0A++++++++%2Bscope%3A+Vec%7EString%7E%0A++++++++%2Btest_refs%3A+Vec%7EString%7E%0A++++++++%2Bdomain%3A+Option%7EString%7E%0A++++++++%2Bfeature%3A+Option%7EString%7E%0A++++++++%2Bspec_ref%3A+Option%7EString%7E%0A++++++++%2Bmetadata%3A+Option%7EOkfMetadata%7E%0A++++%7D%0A%0A++++class+TaskStatus+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Draft%0A++++++++Active%0A++++++++InProgress%0A++++++++Wip%0A++++++++Todo%0A++++++++Reopened%0A++++++++Done%0A++++++++Completed%0A++++++++Deprecated%0A++++++++Superseded%0A++++++++Rejected%0A++++++++Unknown%0A++++%7D%0A%0A++++class+ToolActionType+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++ReadFile%0A++++++++WriteFile%0A++++++++ExecuteCommand%0A++++++++Delegate%0A++++++++Other%0A++++%7D%0A%0A++++class+TopologyEdge+%7B%0A++++++++%2Bfrom%3A+String%0A++++++++%2Bto%3A+String%0A++++++++%2Bedge_type%3A+EdgeType%0A++++++++%2Bweight%3A+Option%7Eu32%7E%0A++++++++%2Bmetadata%3A+HashMap%7EString%0A++++++++%2BString%7E%0A++++%7D%0A%0A++++class+TopologyGraph+%7B%0A++++++++%2Bnodes%3A+HashMap%7EString%0A++++++++%2BTopologyNode%7E%0A++++++++%2Bedges%3A+Vec%7ETopologyEdge%7E%0A++++++++%2Bblast_radius%3A+Option%7EBlastRadiusReport%7E%0A++++%7D%0A%0A++++class+TopologyNode+%7B%0A++++++++%2Bid%3A+String%0A++++++++%2Bname%3A+String%0A++++++++%2Bnode_type%3A+NodeType%0A++++++++%2Bpath%3A+String%0A++++++++%2Bmetadata%3A+HashMap%7EString%0A++++++++%2BString%7E%0A++++%7D%0A%0A++++class+TrustDecision+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Accepted%0A++++++++PolicyRejected%0A++++%7D%0A%0A++++class+ValidationIssue+%7B%0A++++++++%2Bseverity%3A+ValidationSeverity%0A++++++++%2Bpath%3A+String%0A++++++++%2Bmessage%3A+String%0A++++%7D%0A%0A++++class+ValidationSeverity+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Error%0A++++++++Warning%0A++++%7D%0A%0A++++class+VcsMetadata+%7B%0A++++++++%2Bvcs_type%3A+String%0A++++++++%2Bhead%3A+String%0A++++++++%2Bcommit%3A+String%0A++++++++%2Bis_dirty%3A+bool%0A++++%7D%0A%0A++++class+VerificationReport+%7B%0A++++++++%2Bschema%3A+String%0A++++++++%2Bschema_version%3A+String%0A++++++++%2Bexecution_origin%3A+ExecutionOrigin%0A++++++++%2Bverdict%3A+VerificationVerdict%0A++++++++%2Btask_contract%3A+TaskContract%0A++++++++%2Bworkspace_state%3A+WorkspaceState%0A++++++++%2Bexecution_metadata%3A+ExecutionMetadata%0A++++++++%2Bchecks%3A+Vec%7ECheckSummary%7E%0A++++++++%2Bdiagnostics%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Blineage%3A+SessionLineageTrail%7E%0A++++++++%2Btest_provenance%3A+TestProvenanceEvidence%7E%7E%0A++++++++%2Bproof_mode%3A+Option%7EString%7E%0A++++++++%2Battestation%3A+Option%7EAttestationReport%7E%0A++++%7D%0A%0A++++class+VerificationVerdict+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Passed%0A++++++++Failed%0A++++++++Error%0A++++++++Incomplete%0A++++++++Partial%0A++++++++Skipped%0A++++%7D%0A%0A++++class+WorkspaceState+%7B%0A++++++++%2Bmanifest_version%3A+String%0A++++++++%2Bsource_content_digest%3A+String%0A++++++++%2Bsource_manifest_digest_pre%3A+String%0A++++++++%2Bsource_manifest_digest_post%3A+String%0A++++++++%2Bconfig_digest%3A+String%0A++++++++%2Btask_digest%3A+String%0A++++++++%2Bpolicy_digest%3A+String%0A++++++++%2Bcheck_definitions_digest%3A+String%0A++++++++%2Bincluded_files_count%3A+usize%0A++++++++%2Bvcs%3A+Option%7EVcsMetadata%7E%0A++++%7D%0A%0A++++ClaudeCodeAdapter+..%7C%3E+HarnessAdapter%0A++++AntigravityAdapter+..%7C%3E+HarnessAdapter%0A++++MistralAdapter+..%7C%3E+HarnessAdapter%0A++++CodexAdapter+..%7C%3E+HarnessAdapter%0A++++DoctorCheckItem+*--+DoctorCategory+%3A+category%0A++++DoctorCheckItem+*--+DoctorCheckStatus+%3A+status%0A++++MigrationReport+*--+MigrationItem+%3A+items%0A++++CapabilityRequest+*--+ToolActionType+%3A+action_type%0A++++PolicyDecision+*--+DecisionVerdict+%3A+verdict%0A++++PolicyDecision+*--+EnforcementContext+%3A+context%0A++++StackProfile+*--+LayerConfig+%3A+default_layers%0A++++PluginTargetInstallReport+*--+PluginInstallStatus+%3A+status%0A++++DiagnosticReport+*--+DiagnosticFinding+%3A+findings%0A++++ArchitectureConfig+*--+ArchitectureExemption+%3A+exemptions%0A++++CheckpointResult+*--+CheckpointPhase+%3A+phase%0A++++CheckpointResult+*--+SessionHandoff+%3A+handoff%0A++++ScaffoldResult+*--+ScaffoldFileReport+%3A+files%0A++++TopologyEdge+*--+EdgeType+%3A+edge_type%0A++++SpecReadinessReport+*--+DiagnosticFinding+%3A+diagnostics%0A++++GauntletReport+*--+LayerResult+%3A+layers%0A++++ArchitectureFinding+*--+ArchitectureFindingCategory+%3A+category%0A++++ReleaseFinding+*--+ReleaseFindingCategory+%3A+category%0A++++AdapterValidationResult+*--+ValidationIssue+%3A+issues%0A++++CheckSummary+*--+CheckStatus+%3A+status%0A++++TopologyGraph+*--+TopologyEdge+%3A+edges%0A++++TopologyGraph+*--+BlastRadiusReport+%3A+blast_radius%0A++++GlossaryDriftReport+*--+DiagnosticFinding+%3A+avoid_violations%0A++++GlossaryDriftReport+*--+DiagnosticFinding+%3A+diagnostics%0A++++ReleaseReadinessReport+*--+ReleaseFinding+%3A+diagnostics%0A++++ValidationIssue+*--+ValidationSeverity+%3A+severity%0A++++PluginInstallReport+*--+PluginTargetInstallReport+%3A+targets%0A++++PluginInstallReport+*--+DiscoveredHarness+%3A+discovered_harnesses%0A++++ConfigValidationReport+*--+ConfigValidationIssue+%3A+issues%0A++++GauntletConfig+*--+PathsConfig+%3A+paths%0A++++GauntletConfig+*--+ArchitectureConfig+%3A+architecture%0A++++GauntletConfig+*--+CheckpointConfig+%3A+checkpoint%0A++++GauntletConfig+*--+MultiAgentConfig+%3A+multi_agent%0A++++GauntletConfig+*--+LayerConfig+%3A+layers%0A++++WorkspaceState+*--+VcsMetadata+%3A+vcs%0A++++AdrReadinessReport+*--+DiagnosticFinding+%3A+diagnostics%0A++++AdapterHookVerdict+*--+DecisionVerdict+%3A+decision%0A++++ScaffoldFileReport+*--+ScaffoldAction+%3A+action%0A++++DoctorReport+*--+DoctorCheckItem+%3A+checks%0A++++DoctorReport+*--+DoctorVerdict+%3A+verdict%0A++++LayerResult+*--+LayerExecutionStatus+%3A+status%0A++++LayerResult+*--+LayerRequirement+%3A+requirement%0A++++NormalizedToolCall+*--+ToolActionType+%3A+action_type%0A++++DiagnosticFinding+*--+FindingType+%3A+finding_type%0A++++LayerDefinition+*--+LayerRequirement+%3A+requirement%0A++++AttestationReport+*--+VerificationVerdict+%3A+verdict%0A++++AttestationReport+*--+AttestationStatus+%3A+attestation_status%0A++++AttestationReport+*--+TrustDecision+%3A+trust_decision%0A++++VerificationReport+*--+ExecutionOrigin+%3A+execution_origin%0A++++VerificationReport+*--+VerificationVerdict+%3A+verdict%0A++++VerificationReport+*--+TaskContract+%3A+task_contract%0A++++VerificationReport+*--+WorkspaceState+%3A+workspace_state%0A++++VerificationReport+*--+ExecutionMetadata+%3A+execution_metadata%0A++++VerificationReport+*--+CheckSummary+%3A+checks%0A++++VerificationReport+*--+DiagnosticFinding+%3A+diagnostics%0A++++VerificationReport+*--+AttestationReport+%3A+attestation%0A++++ConfigValidationIssue+*--+ValidationSeverity+%3A+severity%0A++++TaskPackageInfo+*--+TaskStatus+%3A+status%0A++++TopologyNode+*--+NodeType+%3A+node_type%0A++++ArchitectureReport+*--+ArchitectureFinding+%3A+findings)** • 💻 `xgauntlet topology open`

> Automatisk synkroniseret fra kodebasens AST via `xgauntlet verify --save`.

## 🧩 Feature-Scoped Diagrams

- [`adapters`](docs/architecture/adapters.md)
- [`architecture`](docs/architecture/architecture.md)
- [`checkpoint`](docs/architecture/checkpoint.md)
- [`config`](docs/architecture/config.md)
- [`diagnostics`](docs/architecture/diagnostics.md)
- [`doctor`](docs/architecture/doctor.md)
- [`evidence`](docs/architecture/evidence.md)
- [`gauntlet`](docs/architecture/gauntlet.md)
- [`migration`](docs/architecture/migration.md)
- [`plugin`](docs/architecture/plugin.md)
- [`policy`](docs/architecture/policy.md)
- [`release`](docs/architecture/release.md)
- [`scaffold`](docs/architecture/scaffold.md)
- [`tasks`](docs/architecture/tasks.md)
- [`topology`](docs/architecture/topology.md)

## 🗺️ Full Domain Model

```mermaid
classDiagram
    direction TB

    %% ====================================================================
    %% 🏛️ xGauntlet Core Domain Model (Conceptual Architecture - CONTEXT.md)
    %% Reflects domain entities and aggregates. Operational errors and CLI flags
    %% are excluded in conceptual view (inspect via --level detailed).
    %% ====================================================================

    class AdapterHookVerdict {
        +allowed: bool
        +decision: DecisionVerdict
        +reason: String
        +reason_code: Option~u32~
    }

    class AdapterValidationResult {
        +valid: bool
        +issues: Vec~ValidationIssue~
    }

    class AdrReadinessReport {
        +is_valid: bool
        +adr_id: String
        +diagnostics: Vec~DiagnosticFinding~
        +inspected_files: Vec~String~
    }

    class AntigravityAdapter {
    }

    class ArchitectureConfig {
        +max_file_lines: Option~usize~
        +max_test_file_lines: Option~usize~
        +forbidden_tokens: Vec~String~
        +forbidden_imports: Vec~String~
        +layers: Vec~String~
        +feature_root: String
        +isolate_features: bool
        +forbid_core_imports: Vec~String~
        +exemptions: Vec~ArchitectureExemption~
    }

    class ArchitectureExemption {
        +path: String
        +reason: String
        +adr: String
        +max_lines: Option~usize~
        +allow_imports: Vec~String~
    }

    class ArchitectureFinding {
        +file_path: String
        +line_number: Option~usize~
        +category: ArchitectureFindingCategory
        +message: String
        +remediation_hint: String
    }

    class ArchitectureFindingCategory {
        <<enumeration>>
        FileExceedsMaxLines
        TestFileExceedsMaxLines
        ExemptionCeilingExceeded
        ForbiddenTokenDetected
        ForbiddenImportDetected
        ExemptionMissingAdr
        ExemptionAdrNotFound
        ExemptionAdrNotAccepted
        RatchetCeilingInflated
        RatchetBaselineUnreadable
    }

    class ArchitectureReport {
        +is_valid: bool
        +files_scanned: usize
        +findings: Vec~ArchitectureFinding~
        +notices: Vec~String~
    }

    class AttestationReport {
        +bundle_path: String
        +artifact_path: String
        +artifact_digest: String
        +verdict: VerificationVerdict
        +attestation_status: AttestationStatus
        +trust_decision: TrustDecision
        +reasons: Vec~String~
        +matched_subject: Option~String~
        +claims: BTreeMap~String
        +String~
    }

    class AttestationStatus {
        <<enumeration>>
        Absent
        Valid
        Invalid
    }

    class BlastRadiusReport {
        +target: String
        +affected_nodes: Vec~String~
        +direct_dependents: Vec~String~
        +indirect_dependents: Vec~String~
        +max_depth_reached: usize
        +total_affected_count: usize
    }

    class CapabilityRequest {
        +action_type: ToolActionType
        +raw_tool_name: String
        +target_resource: String
        +payload_json: String
    }

    class CheckStatus {
        <<enumeration>>
        Passed
        Failed
        Error
        Skipped
    }

    class CheckSummary {
        +name: String
        +status: CheckStatus
        +exit_code: i32
        +duration_seconds: f64
        +optional: bool
        +passed: bool
        +log_digest: String
    }

    class CheckpointConfig {
        +enforce_provenance: bool
    }

    class CheckpointPhase {
        <<enumeration>>
        Spec
        Red
        Green
        Refactor
        Done
    }

    class CheckpointResult {
        +phase: CheckpointPhase
        +task_id: String
        +commit_oid: Option~String~
        +commit_message: String
        +staged_files: Vec~String~
        +preflight_passed: bool
        +details: Option~String~
        +handoff: Option~SessionHandoff~
        +next_phase: Option~String~
        +autonomous_transition: bool
    }

    class ClaudeCodeAdapter {
    }

    class CodexAdapter {
    }

    class ConfigValidationIssue {
        +severity: ValidationSeverity
        +code: String
        +message: String
        +field: Option~String~
        +remediation: Option~String~
    }

    class ConfigValidationReport {
        +is_valid: bool
        +issues: Vec~ConfigValidationIssue~
    }

    class DecisionVerdict {
        <<enumeration>>
        Allow
        Deny
        Ask
        ForceAsk
    }

    class DiagnosticFinding {
        +finding_type: FindingType
        +tool_name: String
        +file_path: String
        +line_number: Option~usize~
        +column_number: Option~usize~
        +message: String
        +remediation_hint: String
        +raw_context: String
    }

    class DiagnosticReport {
        +layer_name: String
        +passed: bool
        +exit_code: i32
        +findings: Vec~DiagnosticFinding~
        +summary: String
    }

    class DiscoveredHarness {
        +name: String
        +detected: bool
        +config_dir: PathBuf
        +plugin_installed: bool
        +plugin_dir: PathBuf
    }

    class DoctorCategory {
        <<enumeration>>
        Host
        Git
        Governance
        Toolchains
        Engine
        Harnesses
    }

    class DoctorCheckItem {
        +name: String
        +category: DoctorCategory
        +status: DoctorCheckStatus
        +message: String
        +detail: Option~String~
        +remediation: Option~String~
        +duration_ms: u64
    }

    class DoctorCheckStatus {
        <<enumeration>>
        Pass
        Warn
        Fail
        Info
    }

    class DoctorReport {
        +workspace: PathBuf
        +timestamp: String
        +engine_version: String
        +target_platform: String
        +checks: Vec~DoctorCheckItem~
        +passed_count: usize
        +warn_count: usize
        +failed_count: usize
        +info_count: usize
        +duration_total_ms: u64
        +verdict: DoctorVerdict
    }

    class DoctorVerdict {
        <<enumeration>>
        Healthy
        Degraded
        Critical
    }

    class EdgeType {
        <<enumeration>>
        Imports
        Calls
        DependsOn
        Defines
        Exports
        Implements
    }

    class EnforcementContext {
        +workspace_id: String
        +has_active_task: bool
        +active_task_id: String
        +read_only: bool
        +allowed_scopes: Vec~String~
        +allowed_test_targets: Vec~String~
    }

    class ExecutionMetadata {
        +started_at: String
        +finished_at: String
        +total_duration_seconds: f64
        +environment: BTreeMap~String
        +String~
    }

    class ExecutionOrigin {
        <<enumeration>>
        Local
        CiUnprivileged
        CiProtected
    }

    class FindingType {
        <<enumeration>>
        LintError
        TypeMismatch
        TestFailure
        PropertyViolation
        MutantSurvived
        DriftDetected
        InvariantViolation
        GeneralError
        Warning
    }

    class GauntletConfig {
        +stack: String
        +save_evidence: bool
        +evidence_file: String
        +evidence_markdown_file: String
        +paths: PathsConfig
        +architecture: ArchitectureConfig
        +checkpoint: CheckpointConfig
        +multi_agent: MultiAgentConfig
        +layers: Vec~LayerConfig~
    }

    class GauntletReport {
        +success: bool
        +layers: Vec~LayerResult~
        +total_duration_seconds: f64
    }

    class GlossaryDriftReport {
        +is_valid: bool
        +orphan_models: Vec~String~
        +phantom_terms: Vec~String~
        +avoid_violations: Vec~DiagnosticFinding~
        +diagnostics: Vec~DiagnosticFinding~
    }

    class HarnessAdapter {
        <<interface>>
    }

    class LayerConfig {
        +name: String
        +command: Vec~String~
        +optional: bool
        +timeout_seconds: f64
        +fail_on_output: bool
    }

    class LayerDefinition {
        +name: String
        +command: Vec~String~
        +optional: bool
        +timeout_seconds: f64
        +fail_on_output: bool
        +requirement: LayerRequirement
    }

    class LayerExecutionStatus {
        <<enumeration>>
        Passed
        Failed
        Skipped
        Unavailable
        TimedOut
        Error
    }

    class LayerRequirement {
        <<enumeration>>
        Required
        Optional
    }

    class LayerResult {
        +name: String
        +exit_code: i32
        +passed: bool
        +output: String
        +status: LayerExecutionStatus
        +duration_seconds: f64
        +requirement: LayerRequirement
    }

    class MigrationItem {
        +file: PathBuf
        +description: String
        +changes: Vec~String~
    }

    class MigrationReport {
        +is_success: bool
        +items: Vec~MigrationItem~
        +files_modified: Vec~PathBuf~
    }

    class MistralAdapter {
    }

    class MultiAgentConfig {
        +allow_coordinated_inherit: bool
        +share_cargo_target_dir: bool
    }

    class NodeType {
        <<enumeration>>
        Package
        Module
        File
        Function
        Struct
        Interface
        Class
        Unknown
    }

    class NormalizedToolCall {
        +action_type: ToolActionType
        +target_resource: String
        +raw_tool_name: String
        +payload: Value
    }

    class PathsConfig {
        +tasks_dir: String
        +spec_file: String
        +context_file: String
        +coding_standards_file: String
        +diagram_file: String
    }

    class PluginInstallReport {
        +targets: Vec~PluginTargetInstallReport~
        +discovered_harnesses: Vec~DiscoveredHarness~
        +success: bool
        +is_dry_run: bool
    }

    class PluginInstallStatus {
        <<enumeration>>
        Installed
        UpToDate
        SkippedExisting
        DryRunSimulated
    }

    class PluginTargetInstallReport {
        +harness: String
        +target_dir: PathBuf
        +status: PluginInstallStatus
        +files_written: Vec~PathBuf~
        +files_skipped: Vec~PathBuf~
    }

    class PolicyDecision {
        +verdict: DecisionVerdict
        +reason: String
        +reason_code: u32
    }

    class ReleaseFinding {
        +file_path: String
        +category: ReleaseFindingCategory
        +message: String
        +remediation_hint: String
    }

    class ReleaseFindingCategory {
        <<enumeration>>
        ConfigVersionMismatch
        MissingChangelog
        ChangelogVersionMismatch
        UnreferencedAdr
        MissingManifest
        DirtyWorktree
        UnformattedCode
        GeneralError
    }

    class ReleaseReadinessReport {
        +is_ready: bool
        +declared_version: String
        +versions_by_source: BTreeMap~String
        +String~
        +inspected_files: Vec~String~
        +changelog_versions: Vec~String~
        +unreferenced_adrs: Vec~String~
        +diagnostics: Vec~ReleaseFinding~
        +duration_ms: u64
    }

    class ScaffoldAction {
        <<enumeration>>
        Created
        Skipped
        Overwritten
        WouldCreate
        WouldSkip
        WouldOverwrite
    }

    class ScaffoldFileReport {
        +path: String
        +action: ScaffoldAction
        +reason: Option~String~
    }

    class ScaffoldResult {
        +workspace: PathBuf
        +stack: String
        +files: Vec~ScaffoldFileReport~
        +created_count: usize
        +skipped_count: usize
        +overwritten_count: usize
        +is_success: bool
    }

    class SessionHandoff {
        +task_id: String
        +branch: String
        +commit_oid: String
        +commits_ahead: usize
        +is_clean: bool
        +next_role: String
        +formatted_card: String
    }

    class SpecReadinessReport {
        +is_valid: bool
        +task_id: String
        +diagnostics: Vec~DiagnosticFinding~
        +inspected_files: Vec~String~
        +must_not_rules: Vec~String~
        +acceptance_criteria: Vec~String~
    }

    class StackProfile {
        +name: String
        +description: String
        +detection_files: Vec~String~
        +default_layers: Vec~LayerConfig~
    }

    class TaskContract {
        +task_id: String
        +task_title: String
        +task_digest: String
        +acceptance_criteria: Vec~String~
        +unresolved_criteria: Vec~String~
    }

    class TaskPackageInfo {
        +task_id: String
        +title: String
        +status: TaskStatus
        +intent: Option~String~
        +purpose: String
        +acceptance_criteria: Vec~String~
        +unresolved_criteria: Vec~String~
        +must_not: Vec~String~
        +scope: Vec~String~
        +test_refs: Vec~String~
        +domain: Option~String~
        +feature: Option~String~
        +spec_ref: Option~String~
        +metadata: Option~OkfMetadata~
    }

    class TaskStatus {
        <<enumeration>>
        Draft
        Active
        InProgress
        Wip
        Todo
        Reopened
        Done
        Completed
        Deprecated
        Superseded
        Rejected
        Unknown
    }

    class ToolActionType {
        <<enumeration>>
        ReadFile
        WriteFile
        ExecuteCommand
        Delegate
        Other
    }

    class TopologyEdge {
        +from: String
        +to: String
        +edge_type: EdgeType
        +weight: Option~u32~
        +metadata: HashMap~String
        +String~
    }

    class TopologyGraph {
        +nodes: HashMap~String
        +TopologyNode~
        +edges: Vec~TopologyEdge~
        +blast_radius: Option~BlastRadiusReport~
    }

    class TopologyNode {
        +id: String
        +name: String
        +node_type: NodeType
        +path: String
        +metadata: HashMap~String
        +String~
    }

    class TrustDecision {
        <<enumeration>>
        Accepted
        PolicyRejected
    }

    class ValidationIssue {
        +severity: ValidationSeverity
        +path: String
        +message: String
    }

    class ValidationSeverity {
        <<enumeration>>
        Error
        Warning
    }

    class VcsMetadata {
        +vcs_type: String
        +head: String
        +commit: String
        +is_dirty: bool
    }

    class VerificationReport {
        +schema: String
        +schema_version: String
        +execution_origin: ExecutionOrigin
        +verdict: VerificationVerdict
        +task_contract: TaskContract
        +workspace_state: WorkspaceState
        +execution_metadata: ExecutionMetadata
        +checks: Vec~CheckSummary~
        +diagnostics: Vec~DiagnosticFinding~
        +lineage: SessionLineageTrail~
        +test_provenance: TestProvenanceEvidence~~
        +proof_mode: Option~String~
        +attestation: Option~AttestationReport~
    }

    class VerificationVerdict {
        <<enumeration>>
        Passed
        Failed
        Error
        Incomplete
        Partial
        Skipped
    }

    class WorkspaceState {
        +manifest_version: String
        +source_content_digest: String
        +source_manifest_digest_pre: String
        +source_manifest_digest_post: String
        +config_digest: String
        +task_digest: String
        +policy_digest: String
        +check_definitions_digest: String
        +included_files_count: usize
        +vcs: Option~VcsMetadata~
    }

    ClaudeCodeAdapter ..|> HarnessAdapter
    AntigravityAdapter ..|> HarnessAdapter
    MistralAdapter ..|> HarnessAdapter
    CodexAdapter ..|> HarnessAdapter
    DoctorCheckItem *-- DoctorCategory : category
    DoctorCheckItem *-- DoctorCheckStatus : status
    MigrationReport *-- MigrationItem : items
    CapabilityRequest *-- ToolActionType : action_type
    PolicyDecision *-- DecisionVerdict : verdict
    PolicyDecision *-- EnforcementContext : context
    StackProfile *-- LayerConfig : default_layers
    PluginTargetInstallReport *-- PluginInstallStatus : status
    DiagnosticReport *-- DiagnosticFinding : findings
    ArchitectureConfig *-- ArchitectureExemption : exemptions
    CheckpointResult *-- CheckpointPhase : phase
    CheckpointResult *-- SessionHandoff : handoff
    ScaffoldResult *-- ScaffoldFileReport : files
    TopologyEdge *-- EdgeType : edge_type
    SpecReadinessReport *-- DiagnosticFinding : diagnostics
    GauntletReport *-- LayerResult : layers
    ArchitectureFinding *-- ArchitectureFindingCategory : category
    ReleaseFinding *-- ReleaseFindingCategory : category
    AdapterValidationResult *-- ValidationIssue : issues
    CheckSummary *-- CheckStatus : status
    TopologyGraph *-- TopologyEdge : edges
    TopologyGraph *-- BlastRadiusReport : blast_radius
    GlossaryDriftReport *-- DiagnosticFinding : avoid_violations
    GlossaryDriftReport *-- DiagnosticFinding : diagnostics
    ReleaseReadinessReport *-- ReleaseFinding : diagnostics
    ValidationIssue *-- ValidationSeverity : severity
    PluginInstallReport *-- PluginTargetInstallReport : targets
    PluginInstallReport *-- DiscoveredHarness : discovered_harnesses
    ConfigValidationReport *-- ConfigValidationIssue : issues
    GauntletConfig *-- PathsConfig : paths
    GauntletConfig *-- ArchitectureConfig : architecture
    GauntletConfig *-- CheckpointConfig : checkpoint
    GauntletConfig *-- MultiAgentConfig : multi_agent
    GauntletConfig *-- LayerConfig : layers
    WorkspaceState *-- VcsMetadata : vcs
    AdrReadinessReport *-- DiagnosticFinding : diagnostics
    AdapterHookVerdict *-- DecisionVerdict : decision
    ScaffoldFileReport *-- ScaffoldAction : action
    DoctorReport *-- DoctorCheckItem : checks
    DoctorReport *-- DoctorVerdict : verdict
    LayerResult *-- LayerExecutionStatus : status
    LayerResult *-- LayerRequirement : requirement
    NormalizedToolCall *-- ToolActionType : action_type
    DiagnosticFinding *-- FindingType : finding_type
    LayerDefinition *-- LayerRequirement : requirement
    AttestationReport *-- VerificationVerdict : verdict
    AttestationReport *-- AttestationStatus : attestation_status
    AttestationReport *-- TrustDecision : trust_decision
    VerificationReport *-- ExecutionOrigin : execution_origin
    VerificationReport *-- VerificationVerdict : verdict
    VerificationReport *-- TaskContract : task_contract
    VerificationReport *-- WorkspaceState : workspace_state
    VerificationReport *-- ExecutionMetadata : execution_metadata
    VerificationReport *-- CheckSummary : checks
    VerificationReport *-- DiagnosticFinding : diagnostics
    VerificationReport *-- AttestationReport : attestation
    ConfigValidationIssue *-- ValidationSeverity : severity
    TaskPackageInfo *-- TaskStatus : status
    TopologyNode *-- NodeType : node_type
    ArchitectureReport *-- ArchitectureFinding : findings
```
