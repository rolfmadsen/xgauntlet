//! Strict parser and validator for Open Knowledge Format (OKF v0.2) frontmatter.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OkfError {
    #[error("Unclosed YAML frontmatter block (missing terminating '---')")]
    UnclosedFrontmatter,

    #[error("Missing or empty required frontmatter field: 'type'")]
    MissingType,

    #[error("Invalid actor convention: '{0}'")]
    InvalidActor(String),

    #[error("Invalid timestamp '{0}': {1}")]
    InvalidTimestamp(String, String),

    #[error("Frontmatter parse error: {0}")]
    ParseError(String),
}

/// Actor identity adhering to OKF v0.2 specifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub kind: String,
    pub identifier: String,
    pub raw: String,
    pub namespace: Option<String>,
}

impl Actor {
    /// Parses an actor string into a structured Actor object.
    ///
    /// Valid formats:
    /// - `human:<id>`
    /// - `process:<id>`
    /// - `<agent>/<version>`
    pub fn parse(actor_str: &str) -> Result<Self, OkfError> {
        let clean = actor_str.trim();
        if clean.is_empty() {
            return Err(OkfError::InvalidActor(
                "actor must not be empty".to_string(),
            ));
        }

        if let Some(rest) = clean.strip_prefix("human:") {
            let id = rest.trim();
            if id.is_empty() || id.contains('/') {
                return Err(OkfError::InvalidActor(format!(
                    "invalid human actor: '{clean}'"
                )));
            }
            return Ok(Self {
                kind: "human".to_string(),
                identifier: id.to_string(),
                raw: clean.to_string(),
                namespace: None,
            });
        }

        if let Some(rest) = clean.strip_prefix("process:") {
            let id = rest.trim();
            if id.is_empty() {
                return Err(OkfError::InvalidActor(format!(
                    "invalid process actor: '{clean}'"
                )));
            }
            return Ok(Self {
                kind: "process".to_string(),
                identifier: id.to_string(),
                raw: clean.to_string(),
                namespace: None,
            });
        }

        if clean.contains('/') {
            let parts: Vec<&str> = clean.splitn(2, '/').collect();
            let ns = parts[0].trim();
            let id = parts[1].trim();
            if ns.is_empty() || id.is_empty() {
                return Err(OkfError::InvalidActor(format!(
                    "invalid agent actor: '{clean}'"
                )));
            }
            return Ok(Self {
                kind: "agent".to_string(),
                identifier: id.to_string(),
                raw: clean.to_string(),
                namespace: Some(ns.to_string()),
            });
        }

        Err(OkfError::InvalidActor(format!(
            "invalid actor convention: '{clean}'. Must be 'human:<id>', 'process:<id>', or '<producer>/<version>'"
        )))
    }
}

/// Generation metadata in OKF v0.2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedEntry {
    pub by: String,
    pub at: Option<String>,
}

/// Verification record in OKF v0.2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedEntry {
    pub by: String,
    pub at: String,
}

/// Source reference in OKF v0.2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEntry {
    pub resource: String,
    pub id: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
}

/// Structured metadata extracted from OKF v0.2 YAML frontmatter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OkfMetadata {
    pub doc_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub tags: Vec<String>,
    pub generated: Option<GeneratedEntry>,
    pub verified: Vec<VerifiedEntry>,
    pub sources: Vec<SourceEntry>,
    pub stale_after: Option<String>,
}

/// Validates strict ISO 8601 UTC timestamp (e.g. `2026-09-06T16:15:00Z`).
pub fn validate_iso_timestamp(val: &str) -> Result<(), OkfError> {
    let clean = val.trim().trim_matches('"').trim_matches('\'');
    // Format must be YYYY-MM-DDTHH:MM:SS... with explicit Z or offset
    let has_date_sep =
        clean.len() >= 10 && clean.as_bytes()[4] == b'-' && clean.as_bytes()[7] == b'-';
    let has_time_sep =
        clean.len() >= 19 && (clean.as_bytes()[10] == b'T' || clean.as_bytes()[10] == b' ');
    let has_tz = clean.ends_with('Z')
        || clean.contains('+')
        || (clean.len() > 19 && clean[19..].contains('-'));

    if !has_date_sep || !has_time_sep || !has_tz {
        return Err(OkfError::InvalidTimestamp(
            clean.to_string(),
            "timestamp must match ISO 8601 UTC format (e.g. '2026-09-06T12:00:00Z')".to_string(),
        ));
    }

    Ok(())
}

/// Extracts YAML frontmatter and markdown body from file content.
pub fn parse_frontmatter(content: &str) -> Result<(Option<OkfMetadata>, &str), OkfError> {
    let trimmed_start = content.trim_start();
    if !trimmed_start.starts_with("---") {
        return Ok((None, content));
    }

    let mut lines = content.lines();
    // First line must be ---
    let first = lines.next().unwrap_or("").trim();
    if first != "---" {
        return Ok((None, content));
    }

    let mut frontmatter_lines = Vec::new();
    let mut found_closing = false;
    let mut body_start_idx = 0;

    let mut current_offset = 0;
    // Account for first line and newline
    if let Some(pos) = content.find("---") {
        current_offset = pos + 3;
        if current_offset < content.len() && content.as_bytes()[current_offset] == b'\r' {
            current_offset += 1;
        }
        if current_offset < content.len() && content.as_bytes()[current_offset] == b'\n' {
            current_offset += 1;
        }
    }

    let rest = &content[current_offset..];
    let mut line_start = 0;

    for line in rest.lines() {
        let line_len = line.len();
        if line.trim() == "---" {
            found_closing = true;
            body_start_idx = current_offset + line_start + line_len;
            if body_start_idx < content.len() && content.as_bytes()[body_start_idx] == b'\r' {
                body_start_idx += 1;
            }
            if body_start_idx < content.len() && content.as_bytes()[body_start_idx] == b'\n' {
                body_start_idx += 1;
            }
            break;
        }
        frontmatter_lines.push(line);
        line_start += line_len + 1; // approximate for iteration
    }

    if !found_closing {
        return Err(OkfError::UnclosedFrontmatter);
    }

    let body = if body_start_idx <= content.len() {
        &content[body_start_idx..]
    } else {
        ""
    };

    let meta = parse_yaml_metadata(&frontmatter_lines)?;
    Ok((Some(meta), body))
}

fn strip_quotes(s: &str) -> &str {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

fn parse_yaml_metadata(lines: &[&str]) -> Result<OkfMetadata, OkfError> {
    let mut doc_type: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut status: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut generated: Option<GeneratedEntry> = None;
    let verified: Vec<VerifiedEntry> = Vec::new();
    let sources: Vec<SourceEntry> = Vec::new();
    let mut stale_after: Option<String> = None;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim();

            match key {
                "type" => {
                    let cleaned = strip_quotes(val);
                    if !cleaned.is_empty() {
                        doc_type = Some(cleaned.to_string());
                    }
                }
                "title" => {
                    title = Some(strip_quotes(val).to_string());
                }
                "description" => {
                    description = Some(strip_quotes(val).to_string());
                }
                "status" => {
                    status = Some(strip_quotes(val).to_string());
                }
                "stale_after" => {
                    let s = strip_quotes(val);
                    if !s.is_empty() {
                        validate_iso_timestamp(s)?;
                        stale_after = Some(s.to_string());
                    }
                }
                "tags" => {
                    if val.starts_with('[') && val.ends_with(']') {
                        let inner = &val[1..val.len() - 1];
                        for item in inner.split(',') {
                            let item_clean = strip_quotes(item.trim());
                            if !item_clean.is_empty() {
                                tags.push(item_clean.to_string());
                            }
                        }
                    } else if val.is_empty() {
                        // Check subsequent indented lines
                        while i + 1 < lines.len() {
                            let next = lines[i + 1];
                            let next_trim = next.trim();
                            if let Some(stripped) = next_trim.strip_prefix("- ") {
                                let item = strip_quotes(stripped);
                                if !item.is_empty() {
                                    tags.push(item.to_string());
                                }
                                i += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
                "generated" => {
                    if val.starts_with('{') && val.ends_with('}') {
                        let inner = &val[1..val.len() - 1];
                        let mut by = String::new();
                        let mut at = None;
                        for pair in inner.split(',') {
                            if let Some((pk, pv)) = pair.split_once(':') {
                                let pk = pk.trim();
                                let pv = strip_quotes(pv.trim());
                                if pk == "by" {
                                    Actor::parse(pv)?;
                                    by = pv.to_string();
                                } else if pk == "at" {
                                    validate_iso_timestamp(pv)?;
                                    at = Some(pv.to_string());
                                }
                            }
                        }
                        if !by.is_empty() {
                            generated = Some(GeneratedEntry { by, at });
                        }
                    } else if val.is_empty() {
                        let mut by = String::new();
                        let mut at = None;
                        while i + 1 < lines.len() {
                            let next = lines[i + 1].trim();
                            if let Some((pk, pv)) = next.split_once(':') {
                                let pk = pk.trim();
                                let pv = strip_quotes(pv.trim());
                                if pk == "by" {
                                    Actor::parse(pv)?;
                                    by = pv.to_string();
                                    i += 1;
                                    continue;
                                } else if pk == "at" {
                                    validate_iso_timestamp(pv)?;
                                    at = Some(pv.to_string());
                                    i += 1;
                                    continue;
                                }
                            }
                            break;
                        }
                        if !by.is_empty() {
                            generated = Some(GeneratedEntry { by, at });
                        }
                    }
                }
                "verified" if val.starts_with('[') && val.ends_with(']') => {
                    // inline list of maps if any
                }
                _ => {}
            }
        }
        i += 1;
    }

    let doc_type = doc_type.ok_or(OkfError::MissingType)?;

    Ok(OkfMetadata {
        doc_type,
        title,
        description,
        status,
        tags,
        generated,
        verified,
        sources,
        stale_after,
    })
}
