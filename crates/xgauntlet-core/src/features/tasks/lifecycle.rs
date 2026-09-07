//! Task scaffolding, template generation, and sequential lifecycle engine.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::features::evidence::time::current_iso_utc;
use crate::features::tasks::parser::TaskError;

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

/// Engine for task package scaffolding and numbering.
pub struct TaskScaffolder;

impl TaskScaffolder {
    /// Detects the next sequential task number by inspecting existing files in `tasks/`.
    pub fn detect_next_task_number(tasks_dir: &Path) -> u32 {
        if !tasks_dir.is_dir() {
            return 1;
        }

        let entries = match fs::read_dir(tasks_dir) {
            Ok(e) => e,
            Err(_) => return 1,
        };

        let mut max_id = 0u32;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(num) = digits.parse::<u32>() {
                        if num > max_id {
                            max_id = num;
                        }
                    }
                }
            }
        }

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
            } else if ch == '-' || ch == '_' || ch.is_whitespace() {
                if !last_was_dash && !result.is_empty() {
                    result.push('-');
                    last_was_dash = true;
                }
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
        let clean = intent.map(|s| s.trim().to_ascii_lowercase()).unwrap_or_default();
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

    /// Generates markdown content for a new task file adhering to OKF v0.2 and check-spec.
    pub fn generate_task_content(options: &ScaffoldTaskOptions, task_number: u32) -> String {
        let padded_num = Self::format_task_number(task_number);
        let raw_slug = Self::slugify(&options.name);
        let (_, slug) = Self::extract_slug_and_number(&raw_slug);
        let title = Self::format_title(&slug, options.title.as_deref());
        let intent_str = Self::format_intent(options.intent.as_deref());
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

        format!(
r#"---
type: Task Package
title: "Task {padded_num}: {title}"
description: "{purpose}"
status: active
generated: {{ by: process:xgauntlet-task-init, at: "{now_iso}" }}
tags: [task-lifecycle, intent, scaffolding, rust]
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
        let digits: String = raw_slug.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() && raw_slug.chars().nth(digits.len()) == Some('-') {
            let num: u32 = digits.parse().unwrap_or(0);
            let rest = &raw_slug[digits.len() + 1..];
            (Some(num), rest.to_string())
        } else {
            (None, raw_slug.to_string())
        }
    }

    /// Scaffolds a new task package in `tasks/`.
    pub fn scaffold(options: &ScaffoldTaskOptions) -> Result<TaskScaffoldResult, TaskError> {
        let tasks_dir = options.workspace.join("tasks");
        fs::create_dir_all(&tasks_dir)?;

        let raw_slug = Self::slugify(&options.name);
        let (specified_num, slug) = Self::extract_slug_and_number(&raw_slug);

        // Check if a task with this slug already exists in tasks/
        let mut existing_file = None;
        if let Ok(entries) = fs::read_dir(&tasks_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|ext| ext == "md") {
                    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if stem == slug || stem.ends_with(&format!("-{slug}")) {
                        let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
                        let num = digits.parse::<u32>().ok();
                        existing_file = Some((p, num));
                        break;
                    }
                }
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
                let p = tasks_dir.join(format!("{padded}-{slug}.md"));
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
                let p = tasks_dir.join(format!("{padded}-{slug}.md"));
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
        let intent = Self::format_intent(options.intent.as_deref());
        let task_id = format!("{padded_num}-{slug}");

        Ok(TaskScaffoldResult {
            task_id,
            task_number,
            path: file_path,
            title: full_title,
            intent,
            created: true,
        })
    }
}
