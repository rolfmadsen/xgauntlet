use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub verdict: DecisionVerdict,
    pub reason: String,
    pub reason_code: u32,
}

impl PolicyDecision {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"verdict":"{}","reason":"{}","reason_code":{}}}"#,
                self.verdict.as_str(),
                self.reason,
                self.reason_code
            )
        })
    }
}

pub fn parse_capability_request(json: &str) -> CapabilityRequest {
    serde_json::from_str(json).unwrap_or(CapabilityRequest {
        action_type: ToolActionType::Other,
        raw_tool_name: String::new(),
        target_resource: String::new(),
        payload_json: String::new(),
    })
}

pub fn parse_enforcement_context(json: &str) -> EnforcementContext {
    serde_json::from_str(json).unwrap_or_default()
}
