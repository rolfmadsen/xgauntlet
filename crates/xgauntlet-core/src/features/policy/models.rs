//! Domain models for the xGauntlet Security & Zero Ambient Authority Policy Engine.

use serde::{Deserialize, Serialize};

/// Tool action classification for fine-grained capability evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolActionType {
    ReadFile,
    WriteFile,
    ExecuteCommand,
    Other,
}

impl ToolActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::ExecuteCommand => "execute_command",
            Self::Other => "other",
        }
    }
}

/// Strongly-typed capability request from an agent or harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub action_type: ToolActionType,
    #[serde(default)]
    pub raw_tool_name: String,
    #[serde(default)]
    pub target_resource: String,
    #[serde(default)]
    pub payload_json: String,
}

/// Immutable trusted context provided by the supervisor runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EnforcementContext {
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub has_active_task: bool,
    #[serde(default)]
    pub active_task_id: String,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub allowed_scopes: Vec<String>,
    #[serde(default)]
    pub allowed_test_targets: Vec<String>,
}

impl EnforcementContext {
    /// Creates an enforcement context directly resolved from a workspace directory.
    pub fn from_workspace(workspace: &std::path::Path, read_only: bool) -> Self {
        let active_task_id = crate::features::tasks::resolve_active_task_id(workspace);
        let mut has_active_task = false;
        let mut active_task_id_str = String::new();
        let mut allowed_scopes = Vec::new();
        let mut allowed_test_targets = Vec::new();

        if let Some(ref tid) = active_task_id {
            let task_file = workspace.join("tasks").join(format!("{tid}.md"));
            if task_file.is_file() {
                match crate::features::tasks::parse_task_file(&task_file) {
                    Ok(info) => {
                        has_active_task = true;
                        active_task_id_str = tid.clone();
                        allowed_scopes = info.scope;
                        allowed_test_targets = info.test_refs;
                    }
                    Err(_) => {
                        has_active_task = false;
                        active_task_id_str.clear();
                        allowed_scopes.clear();
                        allowed_test_targets.clear();
                    }
                }
            }
        }

        let workspace_id = workspace
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("workspace")
            .to_string();

        Self {
            workspace_id,
            has_active_task,
            active_task_id: active_task_id_str,
            read_only,
            allowed_scopes,
            allowed_test_targets,
        }
    }
}

/// Policy evaluation decision verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionVerdict {
    Allow,
    Deny,
    Ask,
    ForceAsk,
}

impl DecisionVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Ask => "ask",
            Self::ForceAsk => "force_ask",
        }
    }
}

/// Strongly-typed policy decision returned by the policy engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub verdict: DecisionVerdict,
    pub reason: String,
    pub reason_code: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<EnforcementContext>,
}
