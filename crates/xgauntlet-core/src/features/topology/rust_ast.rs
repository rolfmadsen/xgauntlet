//! Rust AST helper extraction for struct fields, enum variants, and type identifiers.

/// Extracts the balanced `{ ... }` body of a struct or enum starting from `start_idx`.
pub fn extract_brace_body(lines: &[&str], start_idx: usize) -> (String, usize) {
    let mut body = String::new();
    let mut brace_depth = 0;
    let mut entered = false;
    let mut j = start_idx;

    while j < lines.len() {
        let line_j = lines[j];
        let open_count = line_j.chars().filter(|&c| c == '{').count();
        let close_count = line_j.chars().filter(|&c| c == '}').count();
        brace_depth += open_count;
        brace_depth -= close_count;

        if open_count > 0 {
            entered = true;
        }

        if entered {
            if j == start_idx {
                let after_brace = line_j.split_once('{').map(|(_, r)| r).unwrap_or("");
                if brace_depth == 0 {
                    let inside = after_brace
                        .rsplit_once('}')
                        .map(|(l, _)| l)
                        .unwrap_or(after_brace);
                    body.push_str(inside);
                    body.push('\n');
                    return (body, j);
                } else {
                    body.push_str(after_brace);
                    body.push('\n');
                }
            } else if brace_depth == 0 {
                let inside = line_j.rsplit_once('}').map(|(l, _)| l).unwrap_or("");
                body.push_str(inside);
                body.push('\n');
                return (body, j);
            } else {
                body.push_str(line_j);
                body.push('\n');
            }
        } else if line_j.trim().ends_with(';') {
            break;
        }

        j += 1;
    }

    (body, j.min(lines.len().saturating_sub(1)))
}

/// Parses struct field declarations `field: Type` from struct body text.
pub fn parse_struct_fields(body: &str) -> Vec<String> {
    let mut without_comments = String::new();
    for line in body.lines() {
        let clean_line = if let Some((code, _)) = line.split_once("//") {
            code
        } else {
            line
        };
        without_comments.push_str(clean_line);
        without_comments.push(' ');
    }

    let mut without_attrs = String::new();
    let mut attr_depth = 0;
    let mut chars = without_comments.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '#' && chars.peek() == Some(&'[') {
            chars.next();
            attr_depth += 1;
            continue;
        }
        if attr_depth > 0 {
            if c == '[' {
                attr_depth += 1;
            } else if c == ']' {
                attr_depth -= 1;
            }
            continue;
        }
        without_attrs.push(c);
    }

    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut depth = 0;
    for c in without_attrs.chars() {
        if c == '<' || c == '(' || c == '[' {
            depth += 1;
            current_field.push(c);
        } else if c == '>' || c == ')' || c == ']' {
            if depth > 0 {
                depth -= 1;
            }
            current_field.push(c);
        } else if c == ',' && depth == 0 {
            let f = current_field.trim();
            if !f.is_empty() {
                if let Some((raw_name, raw_type)) = f.split_once(':') {
                    let field_ident = raw_name.split_whitespace().last().unwrap_or("").trim();
                    let clean_type = sanitize_mermaid_type(raw_type.trim());
                    if !field_ident.is_empty() && !clean_type.is_empty() {
                        fields.push(format!("{field_ident}: {clean_type}"));
                    }
                }
            }
            current_field.clear();
        } else {
            current_field.push(c);
        }
    }

    let f = current_field.trim();
    if !f.is_empty() {
        if let Some((raw_name, raw_type)) = f.split_once(':') {
            let field_ident = raw_name.split_whitespace().last().unwrap_or("").trim();
            let clean_type = sanitize_mermaid_type(raw_type.trim());
            if !field_ident.is_empty() && !clean_type.is_empty() {
                fields.push(format!("{field_ident}: {clean_type}"));
            }
        }
    }

    fields
}

/// Parses enum variant names from enum body text.
pub fn parse_enum_variants(body: &str) -> Vec<String> {
    let mut without_comments = String::new();
    for line in body.lines() {
        let clean_line = if let Some((code, _)) = line.split_once("//") {
            code
        } else {
            line
        };
        without_comments.push_str(clean_line);
        without_comments.push(' ');
    }

    let mut without_attrs = String::new();
    let mut attr_depth = 0;
    let mut chars = without_comments.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '#' && chars.peek() == Some(&'[') {
            chars.next();
            attr_depth += 1;
            continue;
        }
        if attr_depth > 0 {
            if c == '[' {
                attr_depth += 1;
            } else if c == ']' {
                attr_depth -= 1;
            }
            continue;
        }
        without_attrs.push(c);
    }

    let mut variants = Vec::new();
    let mut current_var = String::new();
    let mut depth = 0;
    for c in without_attrs.chars() {
        if c == '(' || c == '{' || c == '[' {
            depth += 1;
            current_var.push(c);
        } else if c == ')' || c == '}' || c == ']' {
            if depth > 0 {
                depth -= 1;
            }
            current_var.push(c);
        } else if c == ',' && depth == 0 {
            let v = current_var.trim();
            let var_ident: String = v
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !var_ident.is_empty() && !variants.contains(&var_ident) {
                variants.push(var_ident);
            }
            current_var.clear();
        } else {
            current_var.push(c);
        }
    }

    let v = current_var.trim();
    let var_ident: String = v
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if !var_ident.is_empty() && !variants.contains(&var_ident) {
        variants.push(var_ident);
    }

    variants
}

/// Sanitizes nested type representations for clean Mermaid rendering.
pub fn sanitize_mermaid_type(raw_type: &str) -> String {
    let mut clean = String::new();
    let parts: Vec<&str> = raw_type.split("::").collect();
    for (idx, part) in parts.iter().enumerate() {
        if idx == parts.len() - 1 {
            clean.push_str(part);
        } else {
            let non_ident: String = part
                .chars()
                .rev()
                .take_while(|c| !c.is_alphanumeric() && *c != '_')
                .collect();
            let prefix: String = non_ident.chars().rev().collect();
            clean.push_str(&prefix);
        }
    }
    clean.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extracts alphanumeric candidate identifiers from a complex type string.
pub fn extract_type_identifiers(type_str: &str) -> Vec<String> {
    let mut idents = Vec::new();
    let mut curr = String::new();
    for c in type_str.chars() {
        if c.is_alphanumeric() || c == '_' {
            curr.push(c);
        } else if !curr.is_empty() {
            idents.push(curr.clone());
            curr.clear();
        }
    }
    if !curr.is_empty() {
        idents.push(curr);
    }
    idents
}
