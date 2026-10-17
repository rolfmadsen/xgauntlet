//! Structured command tokenization and security evaluation for gauntlet-policy-engine.

#[derive(Debug, PartialEq, Eq)]
pub enum TokenizerError {
    UnclosedQuote,
}

pub fn tokenize_command_line(cmd: &str) -> Result<Vec<Vec<String>>, TokenizerError> {
    let mut segments: Vec<Vec<String>> = Vec::new();
    let mut current_segment: Vec<String> = Vec::new();
    let mut current_token = String::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    let chars: Vec<char> = cmd.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if escaped {
            current_token.push(ch);
            escaped = false;
            i += 1;
            continue;
        }

        if ch == '\\' && !in_single_quote {
            escaped = true;
            i += 1;
            continue;
        }

        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }

        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            i += 1;
            continue;
        }

        if in_single_quote || in_double_quote {
            current_token.push(ch);
            i += 1;
            continue;
        }

        // Outside quotes: check for operators: &&, ||, ;, |
        if ch == '&' && i + 1 < len && chars[i + 1] == '&' {
            if !current_token.is_empty() {
                current_segment.push(std::mem::take(&mut current_token));
            }
            if !current_segment.is_empty() {
                segments.push(std::mem::take(&mut current_segment));
            }
            i += 2;
            continue;
        }

        if ch == '|' && i + 1 < len && chars[i + 1] == '|' {
            if !current_token.is_empty() {
                current_segment.push(std::mem::take(&mut current_token));
            }
            if !current_segment.is_empty() {
                segments.push(std::mem::take(&mut current_segment));
            }
            i += 2;
            continue;
        }

        if ch == ';' || ch == '|' {
            if !current_token.is_empty() {
                current_segment.push(std::mem::take(&mut current_token));
            }
            if !current_segment.is_empty() {
                segments.push(std::mem::take(&mut current_segment));
            }
            i += 1;
            continue;
        }

        if ch.is_whitespace() {
            if !current_token.is_empty() {
                current_segment.push(std::mem::take(&mut current_token));
            }
            i += 1;
            continue;
        }

        current_token.push(ch);
        i += 1;
    }

    if in_single_quote || in_double_quote {
        return Err(TokenizerError::UnclosedQuote);
    }

    if !current_token.is_empty() {
        current_segment.push(current_token);
    }
    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    Ok(segments)
}

pub fn normalize_executable(exe: &str) -> String {
    let name = exe.rsplit(['/', '\\']).next().unwrap_or(exe);
    let lower = name.to_ascii_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

pub fn resolve_git_subcommand(args: &[String]) -> Option<(&str, &[String])> {
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-C"
            || arg == "-c"
            || arg == "--config-env"
            || arg == "--exec-path"
            || arg == "--git-dir"
            || arg == "--work-tree"
            || arg == "--namespace"
            || arg == "--super-prefix"
        {
            i += 2;
        } else if arg.starts_with('-') {
            i += 1;
        } else {
            return Some((arg.as_str(), &args[i + 1..]));
        }
    }
    None
}

pub fn is_forbidden_wrapper(norm_exe: &str) -> bool {
    matches!(
        norm_exe,
        "env"
            | "sudo"
            | "doas"
            | "su"
            | "eval"
            | "xargs"
            | "exec"
            | "command"
            | "builtin"
            | "nohup"
            | "timeout"
            | "nice"
            | "stdbuf"
            | "strace"
            | "ltrace"
    )
}

pub fn is_shell_executable(norm_exe: &str) -> bool {
    matches!(
        norm_exe,
        "bash" | "sh" | "zsh" | "dash" | "ksh" | "fish" | "csh" | "tcsh"
    )
}

pub fn is_safe_command_segment(tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let norm_exe = normalize_executable(&tokens[0]);
    match norm_exe.as_str() {
        "git" => {
            if let Some((sub, _)) = resolve_git_subcommand(&tokens[1..]) {
                matches!(sub, "status" | "diff" | "log")
            } else {
                false
            }
        }
        "ls" | "pwd" | "echo" | "which" | "pytest" => true,
        "cargo" => {
            if tokens.len() > 1 {
                matches!(tokens[1].as_str(), "check" | "test" | "clippy" | "fmt")
            } else {
                false
            }
        }
        "npm" => {
            if tokens.len() > 1 {
                tokens[1] == "test"
                    || (tokens[1] == "run" && tokens.len() > 2 && tokens[2] == "test")
            } else {
                false
            }
        }
        "xgauntlet" => {
            if tokens.len() > 1 {
                matches!(
                    tokens[1].as_str(),
                    "verify" | "doctor" | "check-evidence" | "check-spec"
                )
            } else {
                false
            }
        }
        "npx" if tokens.len() > 2 && tokens[1] == "xgauntlet" => {
            matches!(
                tokens[2].as_str(),
                "verify" | "doctor" | "check-evidence" | "check-spec"
            )
        }
        _ => false,
    }
}
