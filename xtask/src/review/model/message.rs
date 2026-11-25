use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCallKind {
    Function,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: ToolCallKind,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    #[must_use]
    pub const fn new(role: Role, content: String) -> Self {
        Self {
            role,
            content,
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[must_use]
    pub fn assistant_with_calls(content: String, reasoning: String, calls: Vec<ToolCall>) -> Self {
        Self {
            reasoning_content: Some(reasoning),
            tool_calls: Some(calls),
            ..Self::new(Role::Assistant, content)
        }
    }

    #[must_use]
    pub fn tool_result(call_identifier: String, content: String) -> Self {
        Self {
            tool_call_id: Some(call_identifier),
            ..Self::new(Role::Tool, content)
        }
    }
}
