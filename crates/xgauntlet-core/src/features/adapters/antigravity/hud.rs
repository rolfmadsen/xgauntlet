//! Truthful Antigravity Cockpit HUD formatting and Smart Kadence rendering.

use serde_json::Value;

use crate::features::tasks::{
    CockpitSnapshot, GitTelemetry, IdleWorkspaceTelemetry, TaskTelemetry, VerificationFreshness,
};
use crate::features::telemetry::{render_jit_directive, PipelinePhase, SessionHealthStatus};

/// Wraps model telemetry and optional inline user HUD markdown into the authoritative
/// Antigravity `injectSteps` `systemMessage` payload schema.
pub fn format_pre_invocation_system_message(
    system_message: &str,
    user_markdown: Option<&str>,
) -> Value {
    let inner = match user_markdown {
        Some(md) if !md.trim().is_empty() => serde_json::json!({
            "systemMessage": system_message,
            "metadata": {
                "user_markdown": md
            }
        }),
        _ => serde_json::json!({
            "systemMessage": system_message
        }),
    };

    serde_json::json!({
        "injectSteps": [
            {
                "systemMessage": inner
            }
        ]
    })
}

fn format_git_state(git: &GitTelemetry) -> String {
    if git.is_clean {
        "clean".to_string()
    } else if git.dirty_count > 0 {
        format!("dirty: {} files", git.dirty_count)
    } else {
        "dirty".to_string()
    }
}

fn format_ahead_suffix(commits_ahead: Option<usize>) -> String {
    match commits_ahead {
        Some(n) if n > 0 => format!(" +{n}"),
        _ => String::new(),
    }
}

fn resolve_gauntlet_verdict(telemetry: &TaskTelemetry) -> (&'static str, Option<&str>) {
    if let Some(ref cockpit) = telemetry.cockpit {
        return (
            cockpit.verification.freshness.as_str(),
            cockpit.verification.reason.as_deref(),
        );
    }

    match telemetry.evidence.as_deref().map(str::trim) {
        Some(e) if e.eq_ignore_ascii_case("sealed") || e.eq_ignore_ascii_case("pass") => {
            ("PASS", None)
        }
        Some(e) if e.eq_ignore_ascii_case("failed") || e.eq_ignore_ascii_case("fail") => {
            ("FAIL", None)
        }
        Some(e) if e.eq_ignore_ascii_case("stale") => ("STALE", Some("stale")),
        _ => ("NONE", None),
    }
}

/// Formats structured, empirical `[XGAUNTLET COCKPIT TELEMETRY]` context for model injection.
pub fn format_ephemeral_telemetry(telemetry: &TaskTelemetry) -> String {
    let short_id = crate::features::checkpoint::extract_short_task_id(&telemetry.task_id);
    let task_id_str = if short_id.is_empty() {
        &telemetry.task_id
    } else {
        short_id
    };

    let phase = telemetry.phase.as_deref().unwrap_or("SPEC");
    let (verdict, stale_reason) = resolve_gauntlet_verdict(telemetry);
    let verdict_detail = match stale_reason {
        Some(r) if verdict == "STALE" || (verdict == "PASS" && r == "partial") => {
            format!("{verdict} ({r})")
        }
        _ => verdict.to_string(),
    };

    let invariants = telemetry.invariants.as_deref().unwrap_or("unverified");
    let mutation = telemetry
        .cockpit
        .as_ref()
        .and_then(|c| c.verification.mutation_status.as_deref())
        .unwrap_or("n/a");
    let scope = telemetry.scope.as_deref().unwrap_or("unscoped");
    let oos_suffix = telemetry
        .cockpit
        .as_ref()
        .filter(|c| c.scope_hygiene.dirty_out_of_scope > 0)
        .map(|c| {
            if c.scope_hygiene.out_of_scope_sample.is_empty() {
                format!(" | ⚠️ OOS dirty: {}", c.scope_hygiene.dirty_out_of_scope)
            } else {
                format!(
                    " | ⚠️ OOS dirty: {} ({})",
                    c.scope_hygiene.dirty_out_of_scope,
                    c.scope_hygiene.out_of_scope_sample.join(", ")
                )
            }
        })
        .unwrap_or_default();

    let git_state = format_git_state(&telemetry.git);
    let ahead_str = format_ahead_suffix(telemetry.cockpit.as_ref().and_then(|c| c.commits_ahead));
    let checkout = telemetry
        .cockpit
        .as_ref()
        .map(|c| c.worktree.checkout_label.as_str())
        .unwrap_or("primary");
    let drift = match telemetry.cockpit.as_ref().map(|c| c.verification.freshness) {
        Some(VerificationFreshness::Pass) => "clean".to_string(),
        Some(VerificationFreshness::Stale) => stale_reason
            .map(|r| format!("stale ({r})"))
            .unwrap_or_else(|| "stale".to_string()),
        Some(VerificationFreshness::Fail) => "failed".to_string(),
        _ => "unverified".to_string(),
    };
    let evidence = telemetry.evidence.as_deref().unwrap_or("none");

    let mut msg = format!(
        "[XGAUNTLET COCKPIT TELEMETRY]\n\
         Task: {task_id_str} | Phase: {phase} | Gauntlet: {verdict_detail}\n\
         Invariants: {invariants} | Mutation: {mutation} | Scope: {scope}{oos_suffix}\n\
         Git: {}@{}{ahead_str} ({git_state}) | Checkout: {checkout} | Drift: {drift}\n\
         Evidence: {evidence}",
        telemetry.git.branch, telemetry.git.head_oid
    );

    if let Some(ref sh) = telemetry.session_health {
        msg.push_str(&format!("\nSession: {}", sh.format_hud_summary()));
    }

    if let Some(pipe_phase) = PipelinePhase::parse_phase(phase) {
        let jit_rendered = render_jit_directive(pipe_phase, Some(task_id_str));
        if !jit_rendered.is_empty() {
            msg.push_str("\n\n");
            msg.push_str(&jit_rendered);
        }
    }

    if let Some(ref sh) = telemetry.session_health {
        if sh.status == SessionHealthStatus::Saturated {
            msg.push_str("\n\n💡 Anbefaling: Sessionen nærmer sig mætning. Forsegl checkpoints og forbered frisk session via 'xgauntlet handoff'.");
        }
    }

    msg
}

/// Formats model-facing telemetry when no active task exists (`Task: NONE`).
pub fn format_idle_telemetry(idle: &IdleWorkspaceTelemetry) -> String {
    let git_state = format_git_state(&idle.git);
    let ahead_str = format_ahead_suffix(idle.commits_ahead);
    let draft_hint = idle
        .latest_draft_id
        .as_deref()
        .map(|id| format!(" (seneste draft: {id})"))
        .unwrap_or_default();

    format!(
        "[XGAUNTLET COCKPIT TELEMETRY]\n\
         Task: NONE | Policy: {} | Gauntlet: {}\n\
         Git: {}@{}{ahead_str} ({git_state}) | Checkout: {}\n\
         Backlog: {} draft{draft_hint}, {} done tasks\n\
         Pointer: Read-Only active. Use discover/spec to create or activate a task in tasks/.",
        idle.policy_mode,
        idle.verification.freshness.as_str(),
        idle.git.branch,
        idle.git.head_oid,
        idle.worktree.checkout_label,
        idle.draft_tasks,
        idle.done_tasks
    )
}

/// Renders the 5-line human-facing blockquote HUD card with clickable Markdown links.
pub fn render_blockquote_hud(telemetry: &TaskTelemetry, next_action: Option<&str>) -> String {
    let short_id = crate::features::checkpoint::extract_short_task_id(&telemetry.task_id);
    let task_label = if !short_id.is_empty() {
        if telemetry.title.starts_with("Task ") {
            telemetry.title.clone()
        } else {
            format!("{short_id} - {}", telemetry.title)
        }
    } else {
        telemetry.title.clone()
    };
    let intent = telemetry.intent.as_deref().unwrap_or("NEW FEATURE");
    let phase = telemetry.phase.as_deref().unwrap_or("SPEC");
    let (verdict, stale_reason) = resolve_gauntlet_verdict(telemetry);
    let gauntlet_segment = match stale_reason {
        Some(r) if verdict == "STALE" || (verdict == "PASS" && r == "partial") => {
            format!("Gauntlet: {verdict} ({r})")
        }
        _ => format!("Gauntlet: {verdict}"),
    };

    let git_state = format_git_state(&telemetry.git);
    let ahead_str = format_ahead_suffix(telemetry.cockpit.as_ref().and_then(|c| c.commits_ahead));
    let wt_suffix = telemetry
        .cockpit
        .as_ref()
        .filter(|c| c.worktree.is_worktree)
        .map(|c| format!(" [{}]", c.worktree.checkout_label))
        .unwrap_or_default();

    let scope = telemetry.scope.as_deref().unwrap_or("unscoped");
    let oos_suffix = telemetry
        .cockpit
        .as_ref()
        .filter(|c| c.scope_hygiene.dirty_out_of_scope > 0)
        .map(|c| format!(" • ⚠️ {} OOS", c.scope_hygiene.dirty_out_of_scope))
        .unwrap_or_default();

    let action = next_action.unwrap_or("Fortsæt med næste handling jf. TDD-fasen.");

    let session_segment = if let Some(ref sh) = telemetry.session_health {
        format!(" | `Session: {}`", sh.format_hud_summary())
    } else {
        String::new()
    };
    let task_link = {
        let norm = telemetry.file_path.trim().replace('\\', "/");
        if norm.is_empty() {
            "tasks/".to_string()
        } else {
            norm
        }
    };
    let arch_link = telemetry
        .cockpit
        .as_ref()
        .and_then(|c| c.diagram_link.as_deref())
        .unwrap_or("architecture.md");

    format!(
        "> ### 🛡️ [Task: {task_label}] `[{intent}: {phase}]`\n\
         > **Status**: `Phase: {phase}` | `{gauntlet_segment}`{session_segment} | `Git: {}@{}{ahead_str}{wt_suffix} • {git_state}`\n\
         > **Progress**: `Criteria: {}/{} {}` | `Scope: {scope}{oos_suffix}`\n\
         > **Links**: 📋 [Task]({task_link}) • 📄 [Spec](spec.md) • 🗺️ [Arch]({arch_link}) • 📖 [Glossary](CONTEXT.md) • 🏛️ [ADR](docs/adr/README.md) • 🧪 [Evidence](evidence.md)\n\
         > 💡 **Next Action:** {action}",
        telemetry.git.branch,
        telemetry.git.head_oid,
        telemetry.criteria.completed,
        telemetry.criteria.total,
        telemetry.criteria.bar
    )
}

/// Renders a compact Read-Only banner for `Task: NONE` workspaces.
pub fn render_idle_banner(idle: &IdleWorkspaceTelemetry) -> String {
    let git_state = format_git_state(&idle.git);
    let ahead_str = format_ahead_suffix(idle.commits_ahead);
    let wt_suffix = if idle.worktree.is_worktree {
        format!(" [{}]", idle.worktree.checkout_label)
    } else {
        String::new()
    };

    format!(
        "> ### 🛡️ [Task: NONE] `[{}]`\n\
         > **Status**: `Policy: {}` | `Gauntlet: {}` | `Backlog: {} draft, {} done` | `Git: {}@{}{ahead_str}{wt_suffix} • {git_state}`\n\
         > 💡 **Next Action:** Brug `discover` / `spec` til at oprette eller aktivere en opgave i `tasks/`.",
        idle.policy_mode,
        idle.policy_mode,
        idle.verification.freshness.as_str(),
        idle.draft_tasks,
        idle.done_tasks,
        idle.git.branch,
        idle.git.head_oid,
    )
}

/// Renders the full user-facing Start-HUD for a `CockpitSnapshot`.
pub fn render_start_hud(snapshot: &CockpitSnapshot) -> String {
    match snapshot {
        CockpitSnapshot::Active(t) => render_blockquote_hud(t, None),
        CockpitSnapshot::Idle(i) => render_idle_banner(i),
    }
}

/// Renders a 1-line compact status chip for repeated unchanged user prompts.
pub fn render_compact_chip(snapshot: &CockpitSnapshot) -> String {
    match snapshot {
        CockpitSnapshot::Active(t) => {
            let short_id = crate::features::checkpoint::extract_short_task_id(&t.task_id);
            let id_label = if short_id.is_empty() {
                t.task_id.as_str()
            } else {
                short_id
            };
            let phase = t.phase.as_deref().unwrap_or("SPEC");
            let (verdict, _) = resolve_gauntlet_verdict(t);
            let session_part = t
                .session_health
                .as_ref()
                .map(|sh| format!(" • Session: `{}`", sh.format_hud_summary()))
                .unwrap_or_default();
            let oos_part = t
                .cockpit
                .as_ref()
                .filter(|c| c.scope_hygiene.dirty_out_of_scope > 0)
                .map(|c| format!(" • ⚠️ `{} OOS`", c.scope_hygiene.dirty_out_of_scope))
                .unwrap_or_default();
            let ahead_str = format_ahead_suffix(t.cockpit.as_ref().and_then(|c| c.commits_ahead));
            let checkout = t
                .cockpit
                .as_ref()
                .filter(|c| c.worktree.is_worktree)
                .map(|c| c.worktree.checkout_label.as_str())
                .unwrap_or(t.git.branch.as_str());
            let git_state = format_git_state(&t.git);
            format!(
                "> 🛡️ `{id_label}` • `{phase}` • Gauntlet: `{verdict}` • Criteria: `{}/{}`{session_part}{oos_part} • Git: `{checkout}@{}{ahead_str} {git_state}`",
                t.criteria.completed, t.criteria.total, t.git.head_oid
            )
        }
        CockpitSnapshot::Idle(i) => {
            let ahead_str = format_ahead_suffix(i.commits_ahead);
            let git_state = format_git_state(&i.git);
            format!(
                "> 🛡️ `Task: NONE` • `Read-Only` • Gauntlet: `{}` • Backlog: `{} draft` • Git: `{}@{}{ahead_str} {git_state}`",
                i.verification.freshness.as_str(),
                i.draft_tasks,
                i.git.branch,
                i.git.head_oid
            )
        }
    }
}

/// Renders the model-facing telemetry text for either active or idle workspaces.
pub fn format_snapshot_system_message(snapshot: &CockpitSnapshot) -> String {
    match snapshot {
        CockpitSnapshot::Active(t) => format_ephemeral_telemetry(t),
        CockpitSnapshot::Idle(i) => format_idle_telemetry(i),
    }
}

/// Computes the structural state fingerprint used by Smart Kadence to detect meaningful state transitions.
pub fn structural_hud_fingerprint(snapshot: &CockpitSnapshot) -> String {
    match snapshot {
        CockpitSnapshot::Active(t) => {
            let phase = t.phase.as_deref().unwrap_or("SPEC");
            let (verdict, stale_reason) = resolve_gauntlet_verdict(t);
            let checkout = t
                .cockpit
                .as_ref()
                .map(|c| c.worktree.checkout_label.as_str())
                .unwrap_or("primary");
            let health = t
                .session_health
                .as_ref()
                .map(|sh| sh.status.as_str())
                .unwrap_or("None");
            let oos = t
                .cockpit
                .as_ref()
                .map(|c| c.scope_hygiene.dirty_out_of_scope > 0)
                .unwrap_or(false);
            format!(
                "active|{}|{}|{}|{}|{}/{}|{}|{}|{}|oos:{}",
                t.task_id,
                phase,
                verdict,
                stale_reason.unwrap_or(""),
                t.criteria.completed,
                t.criteria.total,
                checkout,
                t.git.branch,
                health,
                oos
            )
        }
        CockpitSnapshot::Idle(i) => {
            format!(
                "idle|{}|{}|drafts:{}|done:{}|{}|clean:{}",
                i.verification.freshness.as_str(),
                i.worktree.checkout_label,
                i.draft_tasks,
                i.done_tasks,
                i.git.branch,
                i.git.is_clean
            )
        }
    }
}
