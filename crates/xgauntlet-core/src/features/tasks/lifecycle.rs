//! Task scaffolding, template generation, and sequential lifecycle engine.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::features::evidence::time::current_iso_utc;
use crate::features::tasks::parser::{
    collect_task_files, extract_task_number, parse_task_file, resolve_task_file_path, TaskError,
};

/// Configuration options for scaffolding a new task package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldTaskOptions {
    pub name: String,
    pub title: Option<String>,
    pub intent: Option<String>,
    pub purpose: Option<String>,
    pub workspace: PathBuf,
    pub force: bool,
}

/// Result returned after scaffolding a new task package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskScaffoldResult {
    pub task_id: String,
    pub task_number: u32,
    pub path: PathBuf,
    pub title: String,
    pub intent: String,
    pub created: bool,
}

/// Configuration options for archiving completed (`DONE`) task packages into `tasks/archive/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskArchiveOptions {
    pub workspace: PathBuf,
    pub task_id: Option<String>,
    pub all_done: bool,
    pub dry_run: bool,
}

/// Details of a single archived (or would-be archived) task package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskArchiveEntry {
    pub task_id: String,
    pub from_path: PathBuf,
    pub to_path: PathBuf,
    pub updated_specs: Vec<String>,
}

/// Summary result returned by `TaskScaffolder::archive_tasks`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskArchiveResult {
    pub archived: Vec<TaskArchiveEntry>,
    pub skipped_count: usize,
    pub dry_run: bool,
}

/// Engine for task package scaffolding and numbering.
pub struct TaskScaffolder;

impl TaskScaffolder {
    /// Detects the next sequential task number by inspecting existing files across `tasks/**`.
    pub fn detect_next_task_number(tasks_dir: &Path) -> u32 {
        let max_id = collect_task_files(tasks_dir)
            .into_iter()
            .filter_map(|p| extract_task_number(&p))
            .max()
            .unwrap_or(0);

        max_id + 1
    }

    /// Formats a numeric task ID with leading zeroes (minimum 3 digits, e.g. 001, 014).
    pub fn format_task_number(number: u32) -> String {
        format!("{number:03}")
    }

    /// Normalizes a name or slug into kebab-case.
    pub fn slugify(name: &str) -> String {
        let mut result = String::new();
        let mut last_was_dash = false;

        for ch in name.chars() {
            if ch.is_alphanumeric() {
                result.push(ch.to_ascii_lowercase());
                last_was_dash = false;
            } else if (ch == '-' || ch == '_' || ch.is_whitespace())
                && !last_was_dash
                && !result.is_empty()
            {
                result.push('-');
                last_was_dash = true;
            }
        }

        while result.ends_with('-') {
            result.pop();
        }

        if result.is_empty() {
            "unnamed-task".to_string()
        } else {
            result
        }
    }

    /// Formats an engineering intent string with domain emoji and screaming uppercase.
    pub fn format_intent(intent: Option<&str>) -> String {
        let clean = intent
            .map(|s| s.trim().to_ascii_lowercase())
            .unwrap_or_default();
        match clean.as_str() {
            "bug" | "bugfix" | "fix" | "defect" => "🐛 BUG FIX".to_string(),
            "refactor" | "enhancement" | "perf" => "🔄 REFACTOR".to_string(),
            "query" | "investigation" | "diagnose" => "🔍 QUERY".to_string(),
            "feature" | "new" | "feat" | "" => "🚀 NEW FEATURE".to_string(),
            other => {
                let upper = other.to_ascii_uppercase().replace('-', " ");
                format!("🚀 {upper}")
            }
        }
    }

    /// Normalizes title, defaulting to title-casing the slug if title is not specified.
    pub fn format_title(slug: &str, title: Option<&str>) -> String {
        if let Some(t) = title {
            let trimmed = t.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        slug.split('-')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => {
                        let capitalized = first.to_ascii_uppercase();
                        format!("{capitalized}{}", chars.as_str())
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Extracts optional `(subdomain, feature)` prefix if `name` uses `<subdomain>/<feature>:<slug>` or `<feature>:<slug>` syntax.
    pub fn split_feature_prefix(raw_name: &str) -> (Option<(String, String)>, &str) {
        if let Some((prefix, rest)) = raw_name.split_once(':') {
            if let Some((d, f)) = prefix.split_once('/') {
                let domain = Self::slugify(d);
                let feature = Self::slugify(f);
                if !domain.is_empty() && !feature.is_empty() {
                    return (Some((domain, feature)), rest);
                }
            } else {
                let feature = Self::slugify(prefix);
                if !feature.is_empty() {
                    return (Some((String::new(), feature)), rest);
                }
            }
        }
        (None, raw_name)
    }

    /// Backward-compatible alias for [`Self::split_feature_prefix`].
    pub fn split_slice_prefix(raw_name: &str) -> (Option<(String, String)>, &str) {
        Self::split_feature_prefix(raw_name)
    }

    /// Generates markdown content for a new task file adhering to OKF v0.2 and check-spec.
    pub fn generate_task_content(options: &ScaffoldTaskOptions, task_number: u32) -> String {
        Self::generate_task_content_with_targets(options, task_number, &[], &[])
    }

    /// Generates markdown content for a new task file with optional explicit `scope` and `test_refs`.
    pub fn generate_task_content_with_targets(
        options: &ScaffoldTaskOptions,
        task_number: u32,
        explicit_scope: &[String],
        explicit_test_refs: &[String],
    ) -> String {
        let padded_num = Self::format_task_number(task_number);
        let (feature_info, clean_name) = Self::split_feature_prefix(&options.name);
        let raw_slug = Self::slugify(clean_name);
        let (_, slug) = Self::extract_slug_and_number(&raw_slug);
        let title = Self::format_title(&slug, options.title.as_deref());
        let intent_str = Self::format_intent(options.intent.as_deref());
        let raw_intent = options
            .intent
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("feature");
        let now_iso = current_iso_utc();
        let today = if now_iso.len() >= 10 {
            &now_iso[0..10]
        } else {
            "2026-09-07"
        };

        let purpose = match &options.purpose {
            Some(p) if !p.trim().is_empty() => p.trim().to_string(),
            _ => format!("Etablere {title} jf. spec.md og CONTEXT.md"),
        };

        let custom_scope: Vec<String> = explicit_scope
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| format!("  - \"{}\"", s.replace('"', "\\\"")))
            .collect();
        let custom_tests: Vec<String> = explicit_test_refs
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| format!("  - \"{}\"", s.replace('"', "\\\"")))
            .collect();

        let (feature_yaml, default_scope_yaml) = if let Some((ref d, ref f)) = feature_info {
            if d.is_empty() {
                (
                    format!("intent: {raw_intent}\nfeature: {f}\nspec_ref: \"specs/{f}.md\"\n"),
                    format!("scope:\n  - \"src/features/{f}/**\"\n  - \"specs/{f}.md\""),
                )
            } else {
                (
                    format!(
                        "intent: {raw_intent}\ndomain: {d}\nfeature: {f}\nspec_ref: \"specs/{d}/{f}.md\"\n"
                    ),
                    format!(
                        "scope:\n  - \"src/features/{d}/{f}/**\"\n  - \"specs/{d}/{f}.md\""
                    ),
                )
            }
        } else {
            (
                format!("intent: {raw_intent}\n"),
                "scope:\n  - \"src/**\"".to_string(),
            )
        };

        let scope_yaml = if custom_scope.is_empty() {
            default_scope_yaml
        } else {
            format!("scope:\n{}", custom_scope.join("\n"))
        };

        let test_refs_yaml = if custom_tests.is_empty() {
            "test_refs:\n  - \"tests/**\"".to_string()
        } else {
            format!("test_refs:\n{}", custom_tests.join("\n"))
        };

        format!(
            r#"---
type: Task Package
title: "Task {padded_num}: {title}"
description: "{purpose}"
status: active
{feature_yaml}generated: {{ by: process:xgauntlet-task-init, at: "{now_iso}" }}
tags: [task-lifecycle, intent, scaffolding, rust]
{scope_yaml}
{test_refs_yaml}
---

# Task {padded_num}: {title}

**Status**: `ACTIVE`
**Intent**: `{intent_str}`
**Oprettet**: `{today}`

## 🎯 Formål
{purpose}

## 📋 Acceptance Criteria
- [ ] Gennemføre primær implementering af {title}.
- [ ] Verificere at samtlige tests og invariant-tjek passerer.

## 🚫 Must NOT
- Må IKKE bryde eksisterende arkitektur-invarianter eller API-kontrakter.
- Må IKKE foretage remote publication handlinger (`git push`).
- Må IKKE introducere baggrunds-dæmoner jf. Zero-Daemon invarianten.

## 📝 Revisions
- {today}: Task oprettet som ACTIVE for {title}.

## 🧪 Verifikation
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-spec -t {padded_num}-{slug}`
"#
        )
    }

    fn extract_slug_and_number(raw_slug: &str) -> (Option<u32>, String) {
        let digits: String = raw_slug
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() && raw_slug.chars().nth(digits.len()) == Some('-') {
            let num: u32 = digits.parse().unwrap_or(0);
            let rest = &raw_slug[digits.len() + 1..];
            (Some(num), rest.to_string())
        } else {
            (None, raw_slug.to_string())
        }
    }

    /// Scaffolds a new task package in `tasks/`, with optional explicit `(subdomain/domain, feature)` binding.
    pub fn scaffold_feature(
        options: &ScaffoldTaskOptions,
        domain: Option<&str>,
        feature: Option<&str>,
    ) -> Result<TaskScaffoldResult, TaskError> {
        Self::scaffold_in_subdir(options, domain, feature, None)
    }

    /// Backward-compatible alias for [`Self::scaffold_feature`].
    pub fn scaffold_slice(
        options: &ScaffoldTaskOptions,
        domain: Option<&str>,
        slice: Option<&str>,
    ) -> Result<TaskScaffoldResult, TaskError> {
        Self::scaffold_feature(options, domain, slice)
    }

    /// Scaffolds a new task package in `tasks/` (or an optional safe relative subdirectory under `tasks/`),
    /// with optional `(subdomain/domain, feature)` living spec binding.
    pub fn scaffold_in_subdir(
        options: &ScaffoldTaskOptions,
        domain: Option<&str>,
        feature: Option<&str>,
        subdir: Option<&str>,
    ) -> Result<TaskScaffoldResult, TaskError> {
        Self::scaffold_with_targets(options, domain, feature, subdir, &[], &[])
    }

    /// Scaffolds a new task package with explicit `scope` and `test_refs` targets (AC3).
    pub fn scaffold_with_targets(
        options: &ScaffoldTaskOptions,
        domain: Option<&str>,
        feature: Option<&str>,
        subdir: Option<&str>,
        explicit_scope: &[String],
        explicit_test_refs: &[String],
    ) -> Result<TaskScaffoldResult, TaskError> {
        let mut effective = options.clone();
        if let Some(f) = feature.map(|f| f.trim()).filter(|f| !f.is_empty()) {
            if !effective.name.contains(':') {
                if let Some(d) = domain.map(|d| d.trim()).filter(|d| !d.is_empty()) {
                    effective.name = format!("{d}/{f}:{}", effective.name);
                } else {
                    effective.name = format!("{f}:{}", effective.name);
                }
            }
        }
        Self::scaffold_internal(&effective, subdir, explicit_scope, explicit_test_refs)
    }

    /// Scaffolds a new task package in `tasks/`.
    pub fn scaffold(options: &ScaffoldTaskOptions) -> Result<TaskScaffoldResult, TaskError> {
        Self::scaffold_internal(options, None, &[], &[])
    }

    fn resolve_target_subdir(tasks_dir: &Path, subdir: Option<&str>) -> Result<PathBuf, TaskError> {
        let Some(raw) = subdir else {
            return Ok(tasks_dir.to_path_buf());
        };
        let norm = raw.trim().replace('\\', "/");
        if norm.is_empty() {
            return Ok(tasks_dir.to_path_buf());
        }
        if norm.starts_with('/') || norm.contains(':') {
            return Err(TaskError::InvalidSpecification(format!(
                "Invalid task subdirectory '{raw}': must be a relative path inside tasks/"
            )));
        }
        let mut dir = tasks_dir.to_path_buf();
        for seg in norm.split('/') {
            if seg.is_empty() || seg == "." || seg == ".." {
                return Err(TaskError::InvalidSpecification(format!(
                    "Invalid task subdirectory '{raw}': path traversal or empty segment is prohibited"
                )));
            }
            dir = dir.join(seg);
        }
        Ok(dir)
    }

    fn scaffold_internal(
        options: &ScaffoldTaskOptions,
        subdir: Option<&str>,
        explicit_scope: &[String],
        explicit_test_refs: &[String],
    ) -> Result<TaskScaffoldResult, TaskError> {
        let tasks_dir = options.workspace.join("tasks");
        fs::create_dir_all(&tasks_dir)?;
        let target_dir = Self::resolve_target_subdir(&tasks_dir, subdir)?;
        fs::create_dir_all(&target_dir)?;

        let (feature_info, clean_name) = Self::split_feature_prefix(&options.name);
        let raw_slug = Self::slugify(clean_name);
        let (specified_num, slug) = Self::extract_slug_and_number(&raw_slug);

        // Check recursively if a task with this slug already exists anywhere in tasks/**
        let mut existing_file = None;
        for p in collect_task_files(&tasks_dir) {
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if stem == slug || stem.ends_with(&format!("-{slug}")) {
                let num = extract_task_number(&p);
                existing_file = Some((p, num));
                break;
            }
        }

        let (task_number, file_path) = match (existing_file, specified_num) {
            (Some((existing_path, num_opt)), _) => {
                if !options.force {
                    return Err(TaskError::InvalidSpecification(format!(
                        "Task file '{}' already exists. Use force to overwrite.",
                        existing_path.display()
                    )));
                }
                let num = num_opt.unwrap_or_else(|| Self::detect_next_task_number(&tasks_dir));
                (num, existing_path)
            }
            (None, Some(num)) => {
                let padded = Self::format_task_number(num);
                let p = target_dir.join(format!("{padded}-{slug}.md"));
                if p.exists() && !options.force {
                    return Err(TaskError::InvalidSpecification(format!(
                        "Task file '{}' already exists. Use force to overwrite.",
                        p.display()
                    )));
                }
                (num, p)
            }
            (None, None) => {
                let num = Self::detect_next_task_number(&tasks_dir);
                let padded = Self::format_task_number(num);
                let p = target_dir.join(format!("{padded}-{slug}.md"));
                if p.exists() && !options.force {
                    return Err(TaskError::InvalidSpecification(format!(
                        "Task file '{}' already exists. Use force to overwrite.",
                        p.display()
                    )));
                }
                (num, p)
            }
        };

        let content = Self::generate_task_content_with_targets(
            options,
            task_number,
            explicit_scope,
            explicit_test_refs,
        );
        fs::write(&file_path, content)?;

        let title = Self::format_title(&slug, options.title.as_deref());
        let padded_num = Self::format_task_number(task_number);
        let full_title = format!("Task {padded_num}: {title}");
        let task_id = format!("{padded_num}-{slug}");

        if let Some((ref d, ref f)) = feature_info {
            Self::sync_feature_spec_file(&options.workspace, d, f, &file_path, &full_title)?;
        }

        let intent = Self::format_intent(options.intent.as_deref());

        Ok(TaskScaffoldResult {
            task_id,
            task_number,
            path: file_path,
            title: full_title,
            intent,
            created: true,
        })
    }

    /// Computes a POSIX relative link path from `from_dir` to `to_file` within `workspace`.
    fn relative_link_from_dir(workspace: &Path, from_dir: &Path, to_file: &Path) -> String {
        let rel_from = from_dir.strip_prefix(workspace).unwrap_or(from_dir);
        let rel_to = to_file.strip_prefix(workspace).unwrap_or(to_file);
        let depth = rel_from.components().count();
        let mut prefix = String::new();
        for _ in 0..depth {
            prefix.push_str("../");
        }
        format!(
            "{prefix}{}",
            rel_to.display().to_string().replace('\\', "/")
        )
    }

    fn collect_markdown_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                Self::collect_markdown_files_recursive(&p, out);
            } else if p.is_file() && p.extension().is_some_and(|e| e == "md") {
                out.push(p);
            }
        }
    }

    /// Archives terminal tasks (`DONE`, `COMPLETED`, `DEPRECATED`, `SUPERSEDED`, `REJECTED`)
    /// into `tasks/archive/` and updates relative task links inside `specs/**/*.md`.
    pub fn archive_tasks(options: &TaskArchiveOptions) -> Result<TaskArchiveResult, TaskError> {
        let tasks_dir = options.workspace.join("tasks");
        let archive_dir = tasks_dir.join("archive");

        if !tasks_dir.is_dir() {
            if let Some(ref id) = options.task_id {
                return Err(TaskError::TaskNotFound(id.clone()));
            }
            return Ok(TaskArchiveResult {
                archived: Vec::new(),
                skipped_count: 0,
                dry_run: options.dry_run,
            });
        }

        let mut candidates: Vec<(String, PathBuf)> = Vec::new();
        let mut skipped_count = 0usize;

        if let Some(ref id) = options.task_id {
            let path = resolve_task_file_path(&options.workspace, Some(id))?;
            let info = parse_task_file(&path)?;
            if !info.status.is_terminal() {
                return Err(TaskError::InvalidSpecification(format!(
                    "Task '{}' has status '{}' and cannot be archived (must be DONE, COMPLETED, DEPRECATED, SUPERSEDED, or REJECTED).",
                    info.task_id,
                    info.status.as_str()
                )));
            }
            if path.starts_with(&archive_dir) {
                skipped_count += 1;
            } else {
                candidates.push((info.task_id, path));
            }
        } else {
            for path in collect_task_files(&tasks_dir) {
                if path.starts_with(&archive_dir) {
                    continue;
                }
                if let Ok(info) = parse_task_file(&path) {
                    if info.status.is_terminal() {
                        candidates.push((info.task_id, path));
                    }
                }
            }
        }

        let mut spec_files = Vec::new();
        let specs_dir = options.workspace.join("specs");
        if specs_dir.is_dir() {
            Self::collect_markdown_files_recursive(&specs_dir, &mut spec_files);
            spec_files.sort();
        }

        let mut archived = Vec::new();
        for (task_id, from_path) in candidates {
            let file_name = from_path
                .file_name()
                .ok_or_else(|| TaskError::InvalidSpecification("Invalid task filename".into()))?;
            let to_path = archive_dir.join(file_name);

            if to_path.exists() {
                return Err(TaskError::InvalidSpecification(format!(
                    "Archive destination '{}' already exists.",
                    to_path.display()
                )));
            }

            let mut updated_specs = Vec::new();
            for spec_path in &spec_files {
                let spec_dir = spec_path.parent().unwrap_or(&specs_dir);
                let old_rel =
                    Self::relative_link_from_dir(&options.workspace, spec_dir, &from_path);
                let new_rel = Self::relative_link_from_dir(&options.workspace, spec_dir, &to_path);
                if let Ok(content) = fs::read_to_string(spec_path) {
                    if content.contains(&old_rel) {
                        if !options.dry_run {
                            let updated = content.replace(&old_rel, &new_rel);
                            fs::write(spec_path, updated)?;
                        }
                        let rel_spec = spec_path
                            .strip_prefix(&options.workspace)
                            .unwrap_or(spec_path)
                            .display()
                            .to_string()
                            .replace('\\', "/");
                        updated_specs.push(rel_spec);
                    }
                }
            }

            if !options.dry_run {
                fs::create_dir_all(&archive_dir)?;
                fs::rename(&from_path, &to_path)?;
            }

            archived.push(TaskArchiveEntry {
                task_id,
                from_path,
                to_path,
                updated_specs,
            });
        }

        Ok(TaskArchiveResult {
            archived,
            skipped_count,
            dry_run: options.dry_run,
        })
    }

    fn sync_feature_spec_file(
        workspace: &Path,
        domain: &str,
        feature: &str,
        task_file_path: &Path,
        full_title: &str,
    ) -> Result<(), TaskError> {
        let (spec_path, heading, metadata, layer_path) = if domain.is_empty() {
            (
                workspace.join("specs").join(format!("{feature}.md")),
                format!("`{feature}`"),
                format!("- **Feature**: `{feature}`"),
                format!("features/{feature}"),
            )
        } else {
            (
                workspace
                    .join("specs")
                    .join(domain)
                    .join(format!("{feature}.md")),
                format!("`{domain}/{feature}`"),
                format!("- **Subdomain**: `{domain}`\n- **Feature**: `{feature}`"),
                format!("features/{domain}/{feature}"),
            )
        };

        let spec_dir = spec_path.parent().unwrap_or(workspace);
        let rel_task_link = Self::relative_link_from_dir(workspace, spec_dir, task_file_path);

        let task_link_line = format!("- [{full_title}]({rel_task_link})");
        if !spec_path.exists() {
            if let Some(parent) = spec_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let template = format!(
                "# Living Feature Specification: {heading}\n\n## 🎯 Capability & Facade Contract\n{metadata}\n- **Feature Path**: `{layer_path}`\n\n## 🔌 Public Input / Action Contract\n- Dokumenter featurens offentlige state/input og udgående `Action`/event/result her.\n\n## 🛡️ Feature Invariants\n- Må IKKE importere opad fra `app`.\n- Må IKKE importere direkte fra andre features i `features/`.\n\n## 📋 Relaterede Tasks\n{task_link_line}\n"
            );
            fs::write(&spec_path, template)?;
        } else if let Ok(existing) = fs::read_to_string(&spec_path) {
            if !existing.contains(&rel_task_link) {
                let updated = Self::append_related_task_link(&existing, &task_link_line);
                fs::write(&spec_path, updated)?;
            }
        }
        Ok(())
    }

    fn append_related_task_link(existing: &str, task_link_line: &str) -> String {
        let mut out = existing.trim_end().to_string();
        if existing.contains("## 📋 Relaterede Tasks") || existing.contains("## Relaterede Tasks")
        {
            out.push('\n');
            out.push_str(task_link_line);
            out.push('\n');
        } else {
            out.push_str("\n\n## 📋 Relaterede Tasks\n");
            out.push_str(task_link_line);
            out.push('\n');
        }
        out
    }
}

/// Marks the specified task file as `DONE` (`status: done` in frontmatter and `**Status**: \`DONE\``
/// in the body), returning its path and original content for transactional rollback.
pub fn mark_task_done(workspace: &Path, task_id: &str) -> Result<(PathBuf, String), TaskError> {
    let task_path = resolve_task_file_path(workspace, Some(task_id))?;
    let original = fs::read_to_string(&task_path)?;
    let line_ending = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    let mut in_frontmatter = false;
    let mut frontmatter_closed = false;

    for (idx, line) in lines.iter_mut().enumerate() {
        let trimmed = line.trim();
        if idx == 0 && trimmed == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter && trimmed == "---" {
            in_frontmatter = false;
            frontmatter_closed = true;
            continue;
        }
        if in_frontmatter && !frontmatter_closed && trimmed.starts_with("status:") {
            *line = "status: done".to_string();
        } else if !in_frontmatter && trimmed.starts_with("**Status**:") {
            *line = "**Status**: `DONE`".to_string();
        }
    }

    let mut updated = lines.join(line_ending);
    if original.ends_with('\n') {
        updated.push_str(line_ending);
    }
    fs::write(&task_path, &updated)?;
    Ok((task_path, original))
}
