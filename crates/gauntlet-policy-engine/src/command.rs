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

/// Checks for unsafe shell metacharacters (`>`, `<`, `;`, `|`, `||`, single `&`, `$(`, `` ` ``)
/// while permitting `&&` outside quotes and literal characters inside quotes.
pub fn has_unsafe_shell_metachars(cmd: &str) -> bool {
    let chars: Vec<char> = cmd.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    while i < len {
        let ch = chars[i];
        if escaped {
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
        if in_single_quote {
            i += 1;
            continue;
        }
        if ch == '`' || (ch == '$' && i + 1 < len && chars[i + 1] == '(') {
            return true;
        }
        if in_double_quote {
            i += 1;
            continue;
        }
        if ch == '&' && i + 1 < len && chars[i + 1] == '&' {
            i += 2;
            continue;
        }
        if matches!(ch, '>' | '<' | ';' | '|' | '&') {
            return true;
        }
        i += 1;
    }
    false
}

/// Checks if any chained command segment invokes a shell or unsafe network/deletion binary.
pub fn has_unsafe_chained_executable(segments: &[Vec<String>]) -> bool {
    segments.iter().any(|seg| {
        seg.first().is_some_and(|exe| {
            let norm = normalize_executable(exe);
            matches!(norm.as_str(), "curl" | "wget" | "rm") || is_shell_executable(&norm)
        })
    })
}

fn is_safe_xgauntlet_subcommand(sub: &str) -> bool {
    matches!(
        sub,
        "status"
            | "telemetry"
            | "verify"
            | "doctor"
            | "check-evidence"
            | "check-spec"
            | "check-release"
            | "check-adr"
            | "check-architecture"
            | "handoff"
            | "lineage"
            | "checkpoint"
    )
}

fn is_safe_xgauntlet_invocation(args: &[String]) -> bool {
    let Some(sub) = args.first().map(|s| s.as_str()) else {
        return false;
    };
    if sub == "task" {
        return args
            .get(1)
            .is_some_and(|task_sub| matches!(task_sub.as_str(), "list" | "status" | "unbind" | "bind"));
    }
    is_safe_xgauntlet_subcommand(sub)
}

fn is_safe_cargo_run_xgauntlet(args: &[String]) -> bool {
    let mut i = 0;
    let mut targets_xgauntlet_cli = false;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" {
            return targets_xgauntlet_cli && is_safe_xgauntlet_invocation(&args[i + 1..]);
        }
        if arg == "-p" || arg == "--package" {
            let Some(pkg) = args.get(i + 1) else {
                return false;
            };
            if pkg != "xgauntlet-cli" {
                return false;
            }
            targets_xgauntlet_cli = true;
            i += 2;
        } else if let Some(pkg) = arg
            .strip_prefix("--package=")
            .or_else(|| arg.strip_prefix("-p"))
        {
            if pkg != "xgauntlet-cli" {
                return false;
            }
            targets_xgauntlet_cli = true;
            i += 1;
        } else if matches!(
            arg,
            "-q" | "--quiet"
                | "--release"
                | "--locked"
                | "--offline"
                | "--frozen"
                | "-v"
                | "--verbose"
        ) {
            i += 1;
        } else {
            return false;
        }
    }
    false
}

pub fn is_safe_command_segment(tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let norm_exe = normalize_executable(&tokens[0]);
    match norm_exe.as_str() {
        "git" => {
            if let Some((sub, sub_args)) = resolve_git_subcommand(&tokens[1..]) {
                matches!(sub, "status" | "diff" | "log" | "show")
                    && !sub_args
                        .iter()
                        .any(|a| a == "--output" || a.starts_with("--output="))
            } else {
                false
            }
        }
        "rg" => !tokens[1..].iter().any(|a| {
            a == "--pre"
                || a.starts_with("--pre=")
                || a == "--pre-glob"
                || a.starts_with("--pre-glob=")
        }),
        "grep" | "ls" | "pwd" | "echo" | "which" | "pytest" => true,
        "cargo" => match tokens.get(1).map(|s| s.as_str()) {
            Some("check" | "test" | "clippy" | "fmt") => true,
            Some("run") => is_safe_cargo_run_xgauntlet(&tokens[2..]),
            _ => false,
        },
        "npm" => {
            if tokens.len() > 1 {
                tokens[1] == "test"
                    || (tokens[1] == "run" && tokens.len() > 2 && tokens[2] == "test")
            } else {
                false
            }
        }
        "xgauntlet" => is_safe_xgauntlet_invocation(&tokens[1..]),
        "npx" if tokens.len() > 2 && tokens[1] == "xgauntlet" => {
            is_safe_xgauntlet_invocation(&tokens[2..])
        }
        _ => false,
    }
}
