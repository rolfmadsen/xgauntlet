//! Deterministic Architectural Boundary Gates (ADR 0012):
//! - Gate A (`LayerDirectionViolation`): Top-down layer dependency enforcement (`app -> features -> core -> shared`).
//! - Gate B (`CrossSliceImportViolation`): Black-box Vertical Slice isolation (`<slice_root>/<domain>/<slice>`).
//! - Gate C (`ForbiddenCoreImportViolation`): Clean Core protection against UI/framework leakage.

use super::models::{
    ArchitectureConfig, ArchitectureExemption, ArchitectureFinding, ArchitectureFindingCategory,
    ArchitectureReport,
};

/// Evaluates Gate A (Layer Direction), Gate B (Slice Isolation), and Gate C (Clean Core) for a single source file.
pub fn evaluate_file_boundaries(
    rel_path: &str,
    content: &str,
    config: &ArchitectureConfig,
    exemption: Option<&ArchitectureExemption>,
    report: &mut ArchitectureReport,
) {
    if config.layers.is_empty() && !config.isolate_slices && config.forbid_core_imports.is_empty() {
        return;
    }

    let imports = extract_file_imports(rel_path, content);
    if imports.is_empty() {
        return;
    }

    let src_layer = classify_path_layer(rel_path, &config.layers);
    let src_slice = extract_slice_identity(rel_path, &config.slice_root);
    let is_in_core = src_layer
        .as_ref()
        .is_some_and(|(_, name)| name.eq_ignore_ascii_case("core"))
        || path_has_segment(rel_path, "core");

    for (line_num, raw_import) in imports {
        if is_import_exempted(&raw_import, exemption) {
            continue;
        }

        // Gate C: Forbidden Core Imports (Clean Core)
        if is_in_core && !config.forbid_core_imports.is_empty() {
            if let Some(forbidden_pkg) =
                match_forbidden_core_import(&raw_import, &config.forbid_core_imports)
            {
                report.add_finding(ArchitectureFinding {
                    file_path: rel_path.to_string(),
                    line_number: Some(line_num),
                    category: ArchitectureFindingCategory::ForbiddenImportDetected,
                    message: format!(
                        "[ForbiddenCoreImportViolation] Core domain file '{}' must not import forbidden UI/framework package '{}' (via '{}') at line {}",
                        rel_path, forbidden_pkg, raw_import, line_num
                    ),
                    remediation_hint: format!(
                        "Keep 'core' pure and framework-agnostic. Move '{}' integration to 'features/' or 'app/'.",
                        forbidden_pkg
                    ),
                });
            }
        }

        // Resolve import into a synthetic workspace-relative path or segment list
        let resolved_target = resolve_import_to_pseudo_path(rel_path, &raw_import);

        // Gate A: Layer Direction (e.g. app [0] -> features [1] -> core [2] -> shared [3])
        if !config.layers.is_empty() {
            if let Some((src_idx, ref src_layer_name)) = src_layer {
                if let Some((target_idx, target_layer_name)) =
                    classify_import_layer(&resolved_target, &raw_import, &config.layers)
                {
                    if src_idx > target_idx {
                        report.add_finding(ArchitectureFinding {
                            file_path: rel_path.to_string(),
                            line_number: Some(line_num),
                            category: ArchitectureFindingCategory::ForbiddenImportDetected,
                            message: format!(
                                "[LayerDirectionViolation] Layer '{}' in '{}' must not import upward from higher layer '{}' (via '{}') at line {}",
                                src_layer_name, rel_path, target_layer_name, raw_import, line_num
                            ),
                            remediation_hint: format!(
                                "Enforce one-way downward dependencies ({}). Emit an explicit Action/Event/Result upward instead of importing '{}' from '{}'.",
                                config.layers.join(" -> "),
                                target_layer_name,
                                src_layer_name
                            ),
                        });
                    }
                }
            }
        }

        // Gate B: Cross-Slice Isolation (<slice_root>/<domain>/<slice>)
        if config.isolate_slices {
            if let Some((ref s_domain, ref s_slice)) = src_slice {
                if let Some((t_domain, t_slice)) =
                    extract_import_slice_identity(&resolved_target, &raw_import, &config.slice_root)
                {
                    if s_domain != &t_domain || s_slice != &t_slice {
                        report.add_finding(ArchitectureFinding {
                            file_path: rel_path.to_string(),
                            line_number: Some(line_num),
                            category: ArchitectureFindingCategory::ForbiddenImportDetected,
                            message: format!(
                                "[CrossSliceImportViolation] Slice '{}/{}' in '{}' must not import directly from peer slice '{}/{}' (via '{}') at line {}",
                                s_domain, s_slice, rel_path, t_domain, t_slice, raw_import, line_num
                            ),
                            remediation_hint: format!(
                                "Keep slice '{}/{}' black-box isolated. Coordinate across slices in the 'app' orchestrator via Actions/Events or extract shared domain types into 'core'.",
                                s_domain, s_slice
                            ),
                        });
                    }
                }
            }
        }
    }
}

fn is_import_exempted(import_spec: &str, exemption: Option<&ArchitectureExemption>) -> bool {
    let Some(ex) = exemption else {
        return false;
    };
    ex.allow_imports.iter().any(|allowed| {
        let a = allowed.trim();
        !a.is_empty() && (import_spec == a || import_spec.starts_with(a) || import_spec.contains(a))
    })
}

fn match_forbidden_core_import<'a>(
    import_spec: &str,
    forbidden_list: &'a [String],
) -> Option<&'a str> {
    let trimmed = import_spec.trim();
    for pkg in forbidden_list {
        let p = pkg.trim();
        if p.is_empty() {
            continue;
        }
        if trimmed == p
            || trimmed.starts_with(&format!("{p}::"))
            || trimmed.starts_with(&format!("{p}/"))
            || trimmed.starts_with(&format!("{p}."))
        {
            return Some(p);
        }
    }
    None
}

fn path_has_segment(path: &str, target: &str) -> bool {
    path.split(['/', '\\'])
        .any(|seg| seg.eq_ignore_ascii_case(target))
}

fn classify_path_layer(rel_path: &str, layers: &[String]) -> Option<(usize, String)> {
    let segments: Vec<&str> = rel_path
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();
    // Skip file name itself when matching directory layers
    let dir_segments = if segments.len() > 1 {
        &segments[..segments.len() - 1]
    } else {
        &segments[..]
    };

    for seg in dir_segments {
        for (idx, layer) in layers.iter().enumerate() {
            if seg.eq_ignore_ascii_case(layer) {
                return Some((idx, layer.clone()));
            }
            // Common convention: 'ui/app' maps to 'app' layer when 'ui' is not a separate layer
            if layer.eq_ignore_ascii_case("app")
                && seg.eq_ignore_ascii_case("ui")
                && !layers.iter().any(|l| l.eq_ignore_ascii_case("ui"))
            {
                return Some((idx, layer.clone()));
            }
        }
    }
    None
}

fn classify_import_layer(
    resolved_target: &str,
    raw_import: &str,
    layers: &[String],
) -> Option<(usize, String)> {
    if let Some(found) = classify_path_layer(resolved_target, layers) {
        return Some(found);
    }

    let tokens: Vec<&str> = raw_import
        .split([':', '/', '.', '\\'])
        .filter(|s| !s.is_empty() && *s != "crate" && *s != "self" && *s != "super" && *s != "@")
        .collect();

    for token in tokens {
        for (idx, layer) in layers.iter().enumerate() {
            if token.eq_ignore_ascii_case(layer) {
                return Some((idx, layer.clone()));
            }
        }
    }
    None
}

/// Extracts `(domain, slice)` if `rel_path` resides under `<slice_root>/<domain>/<slice>/...`
fn extract_slice_identity(rel_path: &str, slice_root: &str) -> Option<(String, String)> {
    let norm_path = rel_path.replace('\\', "/");
    let norm_root = slice_root.trim().trim_matches('/').replace('\\', "/");

    if !norm_root.is_empty() {
        if let Some(rest) = norm_path.strip_prefix(&format!("{norm_root}/")) {
            let parts: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
            if parts.len() >= 2 {
                let domain = parts[0].to_string();
                let slice = parts[1]
                    .strip_suffix(".rs")
                    .or_else(|| parts[1].strip_suffix(".ts"))
                    .or_else(|| parts[1].strip_suffix(".tsx"))
                    .or_else(|| parts[1].strip_suffix(".java"))
                    .unwrap_or(parts[1])
                    .to_string();
                return Some((domain, slice));
            }
        }
    }

    // Fallback: locate 'features/<domain>/<slice>' segment sequence
    let segments: Vec<&str> = norm_path.split('/').filter(|s| !s.is_empty()).collect();
    for (i, seg) in segments.iter().enumerate() {
        if *seg == "features" && i + 2 < segments.len() {
            let domain = segments[i + 1].to_string();
            let slice = segments[i + 2]
                .strip_suffix(".rs")
                .or_else(|| segments[i + 2].strip_suffix(".ts"))
                .or_else(|| segments[i + 2].strip_suffix(".tsx"))
                .or_else(|| segments[i + 2].strip_suffix(".java"))
                .unwrap_or(segments[i + 2])
                .to_string();
            return Some((domain, slice));
        }
    }
    None
}

fn extract_import_slice_identity(
    resolved_target: &str,
    raw_import: &str,
    slice_root: &str,
) -> Option<(String, String)> {
    if let Some(found) = extract_slice_identity(resolved_target, slice_root) {
        return Some(found);
    }

    let tokens: Vec<&str> = raw_import
        .split([':', '/', '.', '\\'])
        .filter(|s| !s.is_empty())
        .collect();
    for (i, tok) in tokens.iter().enumerate() {
        if *tok == "features" && i + 2 < tokens.len() {
            return Some((tokens[i + 1].to_string(), tokens[i + 2].to_string()));
        }
    }
    None
}

/// Resolves Rust (`crate::`, `super::`, `self::`), TS/JS relative (`./`, `../`, `@/`), and Java package imports
/// into a normalized slash-separated pseudo-path for deterministic layer and slice matching.
fn resolve_import_to_pseudo_path(rel_path: &str, raw_import: &str) -> String {
    let trimmed = raw_import.trim();

    // 1. Rust crate:: or TS @/
    if let Some(rest) = trimmed.strip_prefix("crate::") {
        let slash = rest.replace("::", "/");
        return format!("src/{slash}/_target");
    }
    if let Some(rest) = trimmed.strip_prefix("@/") {
        return format!("src/{rest}/_target");
    }

    // 2. Rust super:: or self::
    if trimmed.starts_with("super::") || trimmed.starts_with("self::") {
        let mut dir_parts: Vec<&str> = rel_path.split('/').filter(|s| !s.is_empty()).collect();
        let file_name = dir_parts.pop().unwrap_or("");
        let is_mod_root = matches!(file_name, "mod.rs" | "lib.rs" | "main.rs");

        let mut tokens = trimmed.split("::").peekable();
        let mut first_super_in_leaf = !is_mod_root;

        while let Some(tok) = tokens.peek() {
            if *tok == "self" {
                tokens.next();
            } else if *tok == "super" {
                tokens.next();
                if first_super_in_leaf {
                    // In `src/features/d/s/view.rs`, module parent `super` is `src/features/d/s` (already dir_parts)
                    first_super_in_leaf = false;
                } else {
                    dir_parts.pop();
                }
            } else {
                break;
            }
        }

        let mut resolved: Vec<String> = dir_parts.into_iter().map(|s| s.to_string()).collect();
        for tok in tokens {
            resolved.push(tok.to_string());
        }
        resolved.push("_target".to_string());
        return resolved.join("/");
    }

    // 3. Relative filesystem imports (./ or ../)
    if trimmed.starts_with("./") || trimmed.starts_with("../") {
        let mut dir_parts: Vec<&str> = rel_path.split('/').filter(|s| !s.is_empty()).collect();
        dir_parts.pop(); // Remove file name

        for seg in trimmed.split('/') {
            if seg == "." || seg.is_empty() {
                continue;
            } else if seg == ".." {
                dir_parts.pop();
            } else {
                dir_parts.push(seg);
            }
        }
        let mut out = dir_parts.join("/");
        out.push_str("/_target");
        return out;
    }

    // 4. Dot-separated or :: separated imports
    let normalized = trimmed.replace("::", "/").replace('.', "/");
    format!("{normalized}/_target")
}

/// Extracts `(1-based line_number, import_specifier)` across Rust, TS/JS, Java, Python, and Go.
pub fn extract_file_imports(rel_path: &str, content: &str) -> Vec<(usize, String)> {
    let ext = rel_path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut imports = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }

        match ext.as_str() {
            "rs" => {
                let stmt = trimmed
                    .strip_prefix("pub(crate) ")
                    .or_else(|| trimmed.strip_prefix("pub(super) "))
                    .or_else(|| trimmed.strip_prefix("pub "))
                    .unwrap_or(trimmed);
                if let Some(rest) = stmt.strip_prefix("use ") {
                    let clean = rest
                        .trim_end_matches(';')
                        .split('{')
                        .next()
                        .unwrap_or(rest)
                        .trim()
                        .trim_end_matches("::");
                    if !clean.is_empty() {
                        imports.push((line_num, clean.to_string()));
                    }
                }
            }
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => {
                if (trimmed.starts_with("import ") || trimmed.starts_with("export "))
                    && trimmed.contains("from ")
                {
                    if let Some((_, after_from)) = trimmed.split_once("from ") {
                        let spec = after_from
                            .trim()
                            .trim_end_matches(';')
                            .trim()
                            .trim_matches(['\'', '"']);
                        if !spec.is_empty() {
                            imports.push((line_num, spec.to_string()));
                        }
                    }
                } else if let Some(rest) = trimmed.strip_prefix("import ") {
                    if rest.starts_with('\'') || rest.starts_with('"') {
                        let spec = rest.trim_end_matches(';').trim().trim_matches(['\'', '"']);
                        if !spec.is_empty() {
                            imports.push((line_num, spec.to_string()));
                        }
                    }
                }
            }
            "java" => {
                if let Some(rest) = trimmed.strip_prefix("import ") {
                    let clean = rest
                        .strip_prefix("static ")
                        .unwrap_or(rest)
                        .trim_end_matches(';')
                        .trim();
                    if !clean.is_empty() {
                        imports.push((line_num, clean.to_string()));
                    }
                }
            }
            "py" => {
                if let Some(rest) = trimmed.strip_prefix("from ") {
                    if let Some((pkg, _)) = rest.split_once(" import ") {
                        let clean = pkg.trim();
                        if !clean.is_empty() {
                            imports.push((line_num, clean.to_string()));
                        }
                    }
                } else if let Some(rest) = trimmed.strip_prefix("import ") {
                    let clean = rest.split(',').next().unwrap_or(rest).trim();
                    if !clean.is_empty() {
                        imports.push((line_num, clean.to_string()));
                    }
                }
            }
            "go" if trimmed.contains('"') => {
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        let spec = &trimmed[start + 1..start + 1 + end];
                        if !spec.is_empty() {
                            imports.push((line_num, spec.to_string()));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    imports
}
