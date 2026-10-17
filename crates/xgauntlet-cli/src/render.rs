//! CLI output renderers and presentation formatting helpers.

pub fn render_init_summary(result: &xgauntlet_core::ScaffoldResult, dry_run: bool) {
    println!("\n=== xGauntlet Project Initialization & Scaffold Engine ===\n");
    if dry_run {
        println!(
            "\x1b[33m[DRY RUN] Previewing file actions without modifying filesystem:\x1b[0m\n"
        );
    }

    println!("Workspace:     {}", result.workspace.display());
    println!("Stack profile: {}", result.stack);
    println!("\nFiles:");

    for file in &result.files {
        let (tag, color) = match file.action {
            xgauntlet_core::ScaffoldAction::Created => ("[+] CREATED", "\x1b[32m"),
            xgauntlet_core::ScaffoldAction::Skipped => ("[*] SKIPPED", "\x1b[33m"),
            xgauntlet_core::ScaffoldAction::Overwritten => ("[!] OVERWRITTEN", "\x1b[35m"),
            xgauntlet_core::ScaffoldAction::WouldCreate => ("[+] WOULD CREATE", "\x1b[32m"),
            xgauntlet_core::ScaffoldAction::WouldSkip => ("[*] WOULD SKIP", "\x1b[33m"),
            xgauntlet_core::ScaffoldAction::WouldOverwrite => ("[!] WOULD OVERWRITE", "\x1b[35m"),
        };

        let reason_str = match &file.reason {
            Some(r) => format!(" ({r})"),
            None => String::new(),
        };

        println!("  {}{:<17}\x1b[0m {}{}", color, tag, file.path, reason_str);
    }

    println!();
    if dry_run {
        let would_create = result
            .files
            .iter()
            .filter(|f| f.action == xgauntlet_core::ScaffoldAction::WouldCreate)
            .count();
        let would_skip = result
            .files
            .iter()
            .filter(|f| f.action == xgauntlet_core::ScaffoldAction::WouldSkip)
            .count();
        let would_overwrite = result
            .files
            .iter()
            .filter(|f| f.action == xgauntlet_core::ScaffoldAction::WouldOverwrite)
            .count();
        println!(
            "Summary: {} would create, {} would skip, {} would overwrite.",
            would_create, would_skip, would_overwrite
        );
    } else {
        println!(
            "Summary: {} created, {} skipped, {} overwritten.",
            result.created_count, result.skipped_count, result.overwritten_count
        );
        println!("\n\x1b[32m[SUCCESS] In-repo governance files initialized safely.\x1b[0m");
        println!("Next steps:");
        println!("  1. Inspect 'gauntlet.toml' and 'CONTEXT.md'.");
        println!("  2. Run 'xgauntlet check-spec' to validate specification contracts.");
        println!("  3. Run 'xgauntlet verify' to execute your verification gauntlet.\n");
    }
}

pub fn render_doctor_summary(report: &xgauntlet_core::DoctorReport, verbose: bool) {
    println!("\n=== xGauntlet Workspace Diagnostics & Doctor Engine ===\n");
    println!("Workspace: {}", report.workspace.display());
    println!(
        "Platform:  {} (Engine v{})",
        report.target_platform, report.engine_version
    );
    println!("Timestamp: {}", report.timestamp);
    println!();

    let categories = [
        xgauntlet_core::DoctorCategory::Host,
        xgauntlet_core::DoctorCategory::Git,
        xgauntlet_core::DoctorCategory::Governance,
        xgauntlet_core::DoctorCategory::Toolchains,
        xgauntlet_core::DoctorCategory::Engine,
        xgauntlet_core::DoctorCategory::Harnesses,
    ];

    for cat in &categories {
        let cat_checks: Vec<_> = report
            .checks
            .iter()
            .filter(|c| c.category == *cat)
            .collect();
        if cat_checks.is_empty() {
            continue;
        }

        println!("Category: \x1b[1m{}\x1b[0m", cat.as_str().to_uppercase());
        for check in cat_checks {
            let (tag, color) = match check.status {
                xgauntlet_core::DoctorCheckStatus::Pass => ("[PASS]", "\x1b[32m"),
                xgauntlet_core::DoctorCheckStatus::Warn => ("[WARN]", "\x1b[33m"),
                xgauntlet_core::DoctorCheckStatus::Fail => ("[FAIL]", "\x1b[31m"),
                xgauntlet_core::DoctorCheckStatus::Info => ("[INFO]", "\x1b[36m"),
            };

            println!(
                "  {}{:<6}\x1b[0m {:<28} {} ({:.1}ms)",
                color, tag, check.name, check.message, check.duration_ms
            );

            if verbose {
                if let Some(ref detail) = check.detail {
                    println!("         \x1b[90mDetail: {}\x1b[0m", detail);
                }
            }

            if let Some(ref remediation) = check.remediation {
                println!("         \x1b[33mAction: {}\x1b[0m", remediation);
            }
        }
        println!();
    }

    println!("--------------------------------------------------");
    println!(
        "Summary: {} passed, {} warning(s), {} failed, {} info. Total time: {}ms",
        report.passed_count,
        report.warn_count,
        report.failed_count,
        report.info_count,
        report.duration_total_ms
    );

    match report.verdict {
        xgauntlet_core::DoctorVerdict::Healthy => {
            println!("\x1b[32m[HEALTHY] Workspace environment and toolchains are ready.\x1b[0m\n");
        }
        xgauntlet_core::DoctorVerdict::Degraded => {
            println!(
                "\x1b[33m[DEGRADED] Workspace has warnings. Inspect remediation hints above.\x1b[0m\n"
            );
        }
        xgauntlet_core::DoctorVerdict::Critical => {
            println!(
                "\x1b[31m[CRITICAL] Workspace has failing checks. Remediation required.\x1b[0m\n"
            );
        }
    }
}

pub fn render_release_summary(report: &xgauntlet_core::ReleaseReadinessReport) {
    println!("\n=== \x1b[1m🛡️ xGauntlet Release Readiness Gatekeeper\x1b[0m ===\n");
    println!(
        "Target Declared Version: \x1b[1m{}\x1b[0m",
        report.declared_version
    );
    println!("Inspected Files:         {}", report.inspected_files.len());
    println!("Execution Duration:      {}ms\n", report.duration_ms);

    println!("Manifests & Versions:");
    if report.versions_by_source.is_empty() {
        println!("  \x1b[33m(no manifests found)\x1b[0m");
    } else {
        for (src, ver) in &report.versions_by_source {
            println!("  • {:<30} \x1b[32m{}\x1b[0m", src, ver);
        }
    }
    println!();

    println!("Changelog Releases:");
    if report.changelog_versions.is_empty() {
        println!("  \x1b[33m(no releases recorded in CHANGELOG.md)\x1b[0m");
    } else {
        let preview: Vec<_> = report.changelog_versions.iter().take(5).collect();
        println!(
            "  • Found {} release(s): {}",
            report.changelog_versions.len(),
            preview
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();

    let adr_count = report
        .inspected_files
        .iter()
        .filter(|f| f.starts_with("docs/adr/"))
        .count();
    let unref_count = report.unreferenced_adrs.len();
    let ref_count = adr_count.saturating_sub(unref_count);
    println!("Architecture Decision Records (ADRs):");
    if adr_count == 0 {
        println!("  \x1b[90m(no ADRs found in docs/adr/)\x1b[0m");
    } else {
        let pct = (ref_count * 100).checked_div(adr_count).unwrap_or(100);
        println!(
            "  • Coverage: {}/{} ADRs referenced in README.md/spec.md ({}%)",
            ref_count, adr_count, pct
        );
    }
    println!();

    if !report.diagnostics.is_empty() {
        println!("--------------------------------------------------");
        println!("\x1b[31m[!] Release Discrepancies & Blockers:\x1b[0m");
        for d in &report.diagnostics {
            println!(
                "  \x1b[31m• [{}] {}\x1b[0m: {}",
                d.category, d.file_path, d.message
            );
            println!("    \x1b[33mAction: {}\x1b[0m", d.remediation_hint);
        }
        println!("--------------------------------------------------\n");
    }

    if report.is_ready {
        println!("\x1b[32;1m✅ RELEASE READY\x1b[0m: All manifests in harmony, changelog synced, and ADR coverage complete.\n");
    } else {
        println!(
            "\x1b[31;1m❌ RELEASE BLOCKED\x1b[0m: Resolve the {} issue(s) detailed above before publishing.\n",
            report.diagnostics.len()
        );
    }
}

pub fn render_task_scaffold_summary(res: &xgauntlet_core::TaskScaffoldResult) {
    println!("✅ Scaffolding complete for task [{}]", res.task_id);
    println!("   Number: {:03}", res.task_number);
    println!("   Title:  {}", res.title);
    println!("   Intent: {}", res.intent);
    println!("   Path:   {}", res.path.display());
}

pub fn render_task_status_summary(telemetry: &xgauntlet_core::TaskTelemetry) {
    println!(
        "🛡️  Task Telemetry: [{}] {}",
        telemetry.task_id, telemetry.title
    );
    let intent_str = telemetry.intent.as_deref().unwrap_or("🚀 NEW FEATURE");
    println!(
        "   Status:   {} | Intent: {}",
        telemetry.status.as_str(),
        intent_str
    );
    println!(
        "   Progress: Criteria: {}/{} {} {}%",
        telemetry.criteria.completed,
        telemetry.criteria.total,
        telemetry.criteria.bar,
        telemetry.criteria.percentage
    );
    let git_status = if telemetry.git.is_clean {
        "clean".to_string()
    } else {
        format!("dirty: {} files", telemetry.git.dirty_count)
    };
    println!(
        "   Git:      {}@{} • {}",
        telemetry.git.branch, telemetry.git.head_oid, git_status
    );
    println!("   Path:     {}", telemetry.file_path);
}

pub fn render_task_list_summary(items: &[xgauntlet_core::TaskSummaryItem]) {
    if items.is_empty() {
        println!("No task packages discovered in tasks/.");
        return;
    }
    println!("📋 Task Packages ({} total):", items.len());
    println!("{:<6} {:<10} {:<18} TITLE", "ID", "STATUS", "PROGRESS");
    println!("{}", "-".repeat(75));
    for item in items {
        let num_str = item
            .task_number
            .map(|n| format!("{n:03}"))
            .unwrap_or_else(|| "---".to_string());
        let prog_str = format!(
            "{}/{} {} {:>3}%",
            item.criteria.completed,
            item.criteria.total,
            item.criteria.bar,
            item.criteria.percentage
        );
        println!(
            "{:<6} {:<10} {:<18} {}",
            num_str,
            item.status.as_str(),
            prog_str,
            item.title
        );
    }
}

pub fn render_checkpoint_summary(res: &xgauntlet_core::CheckpointResult) {
    println!("\n=== xGauntlet Phase Checkpoint Engine ===");
    let phase_badge = match res.phase {
        xgauntlet_core::CheckpointPhase::Spec => "\x1b[36m[SPEC]\x1b[0m",
        xgauntlet_core::CheckpointPhase::Red => "\x1b[31m[RED]\x1b[0m",
        xgauntlet_core::CheckpointPhase::Green => "\x1b[32m[GREEN]\x1b[0m",
        xgauntlet_core::CheckpointPhase::Refactor => "\x1b[33m[REFACTOR]\x1b[0m",
        xgauntlet_core::CheckpointPhase::Done => "\x1b[35m[DONE]\x1b[0m",
    };
    println!(
        "🛡️  Phase:      {} {}",
        res.phase.as_str().to_uppercase(),
        phase_badge
    );
    println!("📋 Task ID:    {}", res.task_id);
    let oid_short = res
        .commit_oid
        .as_deref()
        .map(|oid| if oid.len() >= 7 { &oid[..7] } else { oid })
        .unwrap_or("unknown");
    println!(
        "🔒 Commit OID: {} ({})",
        oid_short,
        res.commit_oid.as_deref().unwrap_or("none")
    );
    println!("💬 Message:    {}", res.commit_message);
    println!("📦 Staged:     {} file(s)", res.staged_files.len());
    for f in &res.staged_files {
        println!("   + {}", f);
    }
    println!();
    if let Some(ref handoff) = res.handoff {
        println!("{}", handoff.formatted_card);
    }
}

fn validate_wasm_toolchain(workspace: &std::path::Path) -> anyhow::Result<()> {
    let policy_crate = workspace.join("crates/gauntlet-policy-engine");
    if !policy_crate.is_dir() {
        return Ok(());
    }

    let cargo_check = std::process::Command::new("cargo")
        .arg("--version")
        .output();
    if cargo_check.is_err() || !cargo_check.as_ref().unwrap().status.success() {
        anyhow::bail!("Preflight failed: 'cargo' is not installed or available in PATH.");
    }

    let target_check = std::process::Command::new("rustc")
        .args([
            "--print",
            "target-libdir",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .output();
    match target_check {
        Ok(out) if out.status.success() => {
            let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !std::path::Path::new(&path_str).exists() {
                anyhow::bail!("Preflight failed: 'wasm32-unknown-unknown' target libdir does not exist ({path_str}). Run: rustup target add wasm32-unknown-unknown");
            }
        }
        _ => {
            anyhow::bail!("Preflight failed: Rust target 'wasm32-unknown-unknown' is not installed. Run: rustup target add wasm32-unknown-unknown");
        }
    }

    Ok(())
}

fn collect_candidate_files_for_backup(workspace: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let root_cargo = workspace.join("Cargo.toml");
    if root_cargo.is_file() {
        files.push(root_cargo);
    }
    let changelog = workspace.join("CHANGELOG.md");
    if changelog.is_file() {
        files.push(changelog);
    }
    let root_pkg = workspace.join("package.json");
    if root_pkg.is_file() {
        files.push(root_pkg);
    }

    let crates_dir = workspace.join("crates");
    if crates_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(crates_dir) {
            for entry in entries.flatten() {
                let sub_cargo = entry.path().join("Cargo.toml");
                if sub_cargo.is_file() {
                    files.push(sub_cargo);
                }
            }
        }
    }

    let packages_dir = workspace.join("packages");
    if packages_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(packages_dir) {
            for entry in entries.flatten() {
                let sub_pkg = entry.path().join("package.json");
                if sub_pkg.is_file() {
                    files.push(sub_pkg);
                }
            }
        }
    }

    let wasm_file = workspace.join("crates/xgauntlet-core/wasm/gauntlet_policy.wasm");
    if wasm_file.is_file() {
        files.push(wasm_file);
    }

    files
}

pub fn execute_release_workflow(
    workspace: &std::path::Path,
    current_version: &str,
    target_version: &str,
    dry_run: bool,
    json: bool,
) -> anyhow::Result<()> {
    let today = &xgauntlet_core::current_iso_utc()[..10];
    let branch = xgauntlet_core::features::checkpoint::git::get_current_branch(workspace)
        .unwrap_or_else(|_| "main".to_string());
    let git_instructions =
        xgauntlet_core::generate_release_git_instructions(target_version, &branch);

    let updated_files = if dry_run {
        let mut dry_files = vec![
            "Cargo.toml (dry-run)".to_string(),
            "CHANGELOG.md (dry-run)".to_string(),
        ];
        if workspace.join("crates/gauntlet-policy-engine").is_dir() {
            dry_files.push("crates/xgauntlet-core/wasm/gauntlet_policy.wasm (dry-run)".to_string());
        }
        dry_files
    } else {
        // 1. Preflight toolchain validation
        validate_wasm_toolchain(workspace)?;

        // 2. Disk-staging backup of all candidate manifest and artifact files
        let candidates = collect_candidate_files_for_backup(workspace);
        let backup_dir = workspace.join(format!(
            "target/.staging-backup-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        std::fs::create_dir_all(&backup_dir)?;

        let mut backed_up = Vec::new();
        for file in &candidates {
            if file.is_file() {
                if let Ok(rel) = file.strip_prefix(workspace) {
                    let dest = backup_dir.join(rel);
                    if let Some(parent) = dest.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if std::fs::copy(file, &dest).is_ok() {
                        backed_up.push((file.clone(), dest));
                    }
                }
            }
        }

        // 3. Execution closure with automatic rollback on error
        let result = (|| -> anyhow::Result<Vec<String>> {
            let mut modified = xgauntlet_core::update_manifest_versions(workspace, target_version)?;
            if xgauntlet_core::prepare_changelog_release(workspace, target_version, today)? {
                modified.push("CHANGELOG.md".to_string());
            }

            // 4. Full WASM build against updated release workspace
            let policy_crate = workspace.join("crates/gauntlet-policy-engine");
            if policy_crate.is_dir() {
                let status = std::process::Command::new("cargo")
                    .args([
                        "build",
                        "-p",
                        "gauntlet-policy-engine",
                        "--target",
                        "wasm32-unknown-unknown",
                        "--release",
                    ])
                    .current_dir(workspace)
                    .status()
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to spawn cargo build for wasm policy engine: {e}")
                    })?;

                if !status.success() {
                    anyhow::bail!(
                        "cargo build for gauntlet-policy-engine failed with exit code: {:?}",
                        status.code()
                    );
                }

                // 5. Bytecode validation and synchronization
                let source_wasm = workspace
                    .join("target/wasm32-unknown-unknown/release/gauntlet_policy_engine.wasm");
                if !source_wasm.is_file() {
                    anyhow::bail!(
                        "Compiled WASM binary does not exist at expected path: {}",
                        source_wasm.display()
                    );
                }

                let wasm_bytes = std::fs::read(&source_wasm)
                    .map_err(|e| anyhow::anyhow!("Failed to read compiled wasm artifact: {e}"))?;

                if wasm_bytes.len() < 8 || !wasm_bytes.starts_with(b"\0asm\x01\0\0\0") {
                    anyhow::bail!(
                        "Compiled artifact {} is not valid WebAssembly binary bytecode",
                        source_wasm.display()
                    );
                }

                let mut policy_engine =
                    xgauntlet_core::features::policy::WasmPolicyEngine::from_wasm_bytes(
                        &wasm_bytes,
                    )
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "Compiled WebAssembly policy bytecode failed module validation: {e}"
                        )
                    })?;

                let wasm_ver = policy_engine.get_policy_version().map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to query get_policy_version from compiled wasm module: {e}"
                    )
                })?;

                let clean_target = target_version.trim().trim_start_matches(['v', 'V']);
                if wasm_ver != clean_target {
                    anyhow::bail!("Compiled WebAssembly policy bytecode version ({wasm_ver}) does not match target release version ({clean_target})");
                }

                let dest_wasm = workspace.join("crates/xgauntlet-core/wasm/gauntlet_policy.wasm");
                if let Some(parent) = dest_wasm.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&source_wasm, &dest_wasm).map_err(|e| {
                    anyhow::anyhow!("Failed to synchronize gauntlet_policy.wasm: {e}")
                })?;

                modified.push("crates/xgauntlet-core/wasm/gauntlet_policy.wasm".to_string());
            }

            Ok(modified)
        })();

        match result {
            Ok(modified) => {
                // Slet deterministisk disk-backup ved succes
                let _ = std::fs::remove_dir_all(&backup_dir);
                modified
            }
            Err(err) => {
                // Transaktionel Rollback: Gendan alle filer til den uændrede tilstand
                eprintln!("\n🛑 \x1b[31;1mRelease bump operation failed: {err}\x1b[0m");
                eprintln!("🔄 \x1b[33mRolling back all modified manifests and artifacts from staging backup...\x1b[0m");
                for (orig, backup) in &backed_up {
                    if backup.is_file() {
                        let _ = std::fs::copy(backup, orig);
                    }
                }
                let _ = std::fs::remove_dir_all(&backup_dir);
                eprintln!("✅ \x1b[32mWorking tree successfully restored to clean pre-bump state without git reset.\x1b[0m\n");
                return Err(err);
            }
        }
    };

    if json {
        let res = serde_json::json!({
            "previous_version": current_version,
            "target_version": target_version,
            "branch": branch,
            "dry_run": dry_run,
            "updated_files": updated_files,
            "git_instructions": git_instructions,
        });
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("\n=== \x1b[1m🚀 xGauntlet Semantic Release Manager\x1b[0m ===");
        println!("Previous Version: \x1b[33m{}\x1b[0m", current_version);
        println!("Target Version:   \x1b[32;1mv{}\x1b[0m", target_version);
        println!("Target Branch:    {}", branch);
        if dry_run {
            println!("Mode:             \x1b[33mDRY RUN (no files modified)\x1b[0m\n");
        } else {
            println!("Updated Files:    {} file(s)", updated_files.len());
            for f in &updated_files {
                println!("   + {}", f);
            }
            println!();
        }

        println!(
            "📋 \x1b[1mUdfør følgende Git-kommandoer for at forsegle og publicere releasen:\x1b[0m"
        );
        println!("```bash\n{}\n```\n", git_instructions);
    }

    Ok(())
}
