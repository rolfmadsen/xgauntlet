//! Task scaffolding, template generation, and sequential lifecycle engine.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::features::evidence::time::current_iso_utc;
use crate::features::tasks::models::TaskStatus;
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

    /// Extracts optional `(subdomain, slice)` prefix if `name` uses `<subdomain>/<slice>:<slug>` or `<slice>:<slug>` syntax.
    pub fn split_slice_prefix(raw_name: &str) -> (Option<(String, String)>, &str) {
        if let Some((prefix, rest)) = raw_name.split_once(':') {
            if let Some((d, s)) = prefix.split_once('/') {
                let domain = Self::slugify(d);
                let slice = Self::slugify(s);
                if !domain.is_empty() && !slice.is_empty() {
                    return (Some((domain, slice)), rest);
                }
            } else {
                let slice = Self::slugify(prefix);
                if !slice.is_empty() {
                    return (Some((String::new(), slice)), rest);
                }
            }
        }
        (None, raw_name)
    }

    /// Generates markdown content for a new task file adhering to OKF v0.2 and check-spec.
    pub fn generate_task_content(options: &ScaffoldTaskOptions, task_number: u32) -> String {
        let padded_num = Self::format_task_number(task_number);
        let (slice_info, clean_name) = Self::split_slice_prefix(&options.name);
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

        let (slice_yaml, scope_yaml) = if let Some((ref d, ref s)) = slice_info {
            if d.is_empty() {
                (
                    format!("intent: {raw_intent}\nslice: {s}\nspec_ref: \"specs/{s}.md\"\n"),
                    format!("scope:\n  - \"src/features/{s}/**\"\n  - \"specs/{s}.md\""),
                )
            } else {
                (
                    format!(
                        "intent: {raw_intent}\ndomain: {d}\nslice: {s}\nspec_ref: \"specs/{d}/{s}.md\"\n"
                    ),
                    format!(
                        "scope:\n  - \"src/features/{d}/{s}/**\"\n  - \"specs/{d}/{s}.md\""
                    ),
                )
            }
        } else {
            (
                format!("intent: {raw_intent}\n"),
                "scope:\n  - \"src/**\"".to_string(),
            )
        };

        format!(
            r#"---
type: Task Package
title: "Task {padded_num}: {title}"
description: "{purpose}"
status: active
{slice_yaml}generated: {{ by: process:xgauntlet-task-init, at: "{now_iso}" }}
tags: [task-lifecycle, intent, scaffolding, rust]
{scope_yaml}
test_refs:
  - "tests/**"
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

    /// Scaffolds a new task package in `tasks/`, with optional explicit `(subdomain/domain, slice)` binding.
    pub fn scaffold_slice(
        options: &ScaffoldTaskOptions,
        domain: Option<&str>,
        slice: Option<&str>,
    ) -> Result<TaskScaffoldResult, TaskError> {
        Self::scaffold_in_subdir(options, domain, slice, None)
    }

    /// Scaffolds a new task package in `tasks/` (or an optional safe relative subdirectory under `tasks/`),
    /// with optional `(subdomain/domain, slice)` living spec binding.
    pub fn scaffold_in_subdir(
        options: &ScaffoldTaskOptions,
        domain: Option<&str>,
        slice: Option<&str>,
        subdir: Option<&str>,
    ) -> Result<TaskScaffoldResult, TaskError> {
        let mut effective = options.clone();
        if let Some(s) = slice.map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if !effective.name.contains(':') {
                if let Some(d) = domain.map(|d| d.trim()).filter(|d| !d.is_empty()) {
                    effective.name = format!("{d}/{s}:{}", effective.name);
                } else {
                    effective.name = format!("{s}:{}", effective.name);
                }
            }
        }
        Self::scaffold_internal(&effective, subdir)
    }

    /// Scaffolds a new task package in `tasks/`.
    pub fn scaffold(options: &ScaffoldTaskOptions) -> Result<TaskScaffoldResult, TaskError> {
        Self::scaffold_internal(options, None)
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
    ) -> Result<TaskScaffoldResult, TaskError> {
        let tasks_dir = options.workspace.join("tasks");
        fs::create_dir_all(&tasks_dir)?;
        let target_dir = Self::resolve_target_subdir(&tasks_dir, subdir)?;
        fs::create_dir_all(&target_dir)?;

        let (slice_info, clean_name) = Self::split_slice_prefix(&options.name);
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

        let content = Self::generate_task_content(options, task_number);
        fs::write(&file_path, content)?;

        let title = Self::format_title(&slug, options.title.as_deref());
        let padded_num = Self::format_task_number(task_number);
        let full_title = format!("Task {padded_num}: {title}");
        let task_id = format!("{padded_num}-{slug}");

        if let Some((ref d, ref s)) = slice_info {
            Self::sync_slice_spec_file(&options.workspace, d, s, &file_path, &full_title)?;
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

    /// Archives completed (`DONE` / `COMPLETED`) tasks into `tasks/archive/` and updates
    /// relative task links inside `specs/**/*.md`.
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
            if !matches!(info.status, TaskStatus::Done | TaskStatus::Completed) {
                return Err(TaskError::InvalidSpecification(format!(
                    "Task '{}' has status '{}' and cannot be archived (must be DONE or COMPLETED).",
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
                    if matches!(info.status, TaskStatus::Done | TaskStatus::Completed) {
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

    fn sync_slice_spec_file(
        workspace: &Path,
        domain: &str,
        slice: &str,
        task_file_path: &Path,
        full_title: &str,
    ) -> Result<(), TaskError> {
        let (spec_path, heading, metadata, layer_path) = if domain.is_empty() {
            (
                workspace.join("specs").join(format!("{slice}.md")),
                slice.to_string(),
                format!("- **Slice**: `{slice}`"),
                format!("features/{slice}"),
            )
        } else {
            (
                workspace
                    .join("specs")
                    .join(domain)
                    .join(format!("{slice}.md")),
                format!("{domain} / {slice}"),
                format!("- **Subdomain**: `{domain}`\n- **Slice**: `{slice}`"),
                format!("features/{domain}/{slice}"),
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
                "# Living Slice Specification: {heading}\n\n## 🎯 Capability & Facade Contract\n{metadata}\n- **Layer**: `{layer_path}`\n\n## 🔌 Public Input / Action Contract\n- Dokumenter slicens offentlige state/input og udgående `Action`/event/result her.\n\n## 🛡️ Slice Invariants\n- Må IKKE importere opad fra `app`.\n- Må IKKE importere direkte fra andre slices i `features/`.\n\n## 📋 Relaterede Tasks\n{task_link_line}\n"
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
