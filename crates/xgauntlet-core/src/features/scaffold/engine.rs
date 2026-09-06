//! Scaffolding engine enforcing safe, non-destructive project bootstrapping.

use std::fs;

use crate::features::config::{detect_stack, get_stack_profile};
use crate::features::scaffold::models::{
    ScaffoldAction, ScaffoldError, ScaffoldFileReport, ScaffoldOptions, ScaffoldResult,
};
use crate::features::scaffold::templates::generate_templates;

/// Executes the scaffolding process according to provided options.
///
/// Under default execution, existing project files are strictly preserved (`Skipped`).
/// Supplying `force = true` permits overwriting existing template files.
/// Supplying `dry_run = true` predicts actions without executing any filesystem mutations.
pub fn run_scaffold(options: &ScaffoldOptions) -> Result<ScaffoldResult, ScaffoldError> {
    // 1. Resolve target workspace directory
    let workspace = &options.workspace;
    if !options.dry_run && !workspace.exists() {
        fs::create_dir_all(workspace)?;
    }

    // 2. Resolve programming stack profile
    let stack = match &options.stack {
        Some(explicit) => {
            let normalized = explicit.trim().to_ascii_lowercase();
            if get_stack_profile(&normalized).is_none() {
                return Err(ScaffoldError::InvalidStack(explicit.clone()));
            }
            normalized
        }
        None => detect_stack(workspace).to_string(),
    };

    // 3. Resolve project display name
    let project_name = options.project_name.clone().unwrap_or_else(|| {
        workspace
            .canonicalize()
            .ok()
            .and_then(|p| {
                p.file_name()
                    .and_then(|f| f.to_str().map(|s| s.to_string()))
            })
            .or_else(|| {
                workspace
                    .file_name()
                    .and_then(|f| f.to_str().map(|s| s.to_string()))
            })
            .unwrap_or_else(|| "project".to_string())
    });

    // 4. Generate candidate governance templates
    let templates = generate_templates(&stack, &project_name);

    let mut file_reports = Vec::new();
    let mut created_count = 0;
    let mut skipped_count = 0;
    let mut overwritten_count = 0;

    for template in templates {
        let dest = workspace.join(template.relative_path);
        let exists = dest.is_file();

        let (action, reason) = if exists {
            if options.force {
                if options.dry_run {
                    (ScaffoldAction::WouldOverwrite, None)
                } else {
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&dest, template.content)?;
                    (ScaffoldAction::Overwritten, None)
                }
            } else {
                let msg = Some("File already exists (use --force to overwrite)".to_string());
                if options.dry_run {
                    (ScaffoldAction::WouldSkip, msg)
                } else {
                    (ScaffoldAction::Skipped, msg)
                }
            }
        } else if options.dry_run {
            (ScaffoldAction::WouldCreate, None)
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, template.content)?;
            (ScaffoldAction::Created, None)
        };

        match action {
            ScaffoldAction::Created => created_count += 1,
            ScaffoldAction::Skipped => skipped_count += 1,
            ScaffoldAction::Overwritten => overwritten_count += 1,
            ScaffoldAction::WouldCreate
            | ScaffoldAction::WouldSkip
            | ScaffoldAction::WouldOverwrite => {}
        }

        file_reports.push(ScaffoldFileReport {
            path: template.relative_path.to_string(),
            action,
            reason,
        });
    }

    Ok(ScaffoldResult {
        workspace: workspace.clone(),
        stack,
        files: file_reports,
        created_count,
        skipped_count,
        overwritten_count,
        is_success: true,
    })
}
