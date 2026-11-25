use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
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
    pub fn new(role: &str, content: String) -> Self {
        Self {
            role: role.to_owned(),
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
            ..Self::new("assistant", content)
        }
    }

    #[must_use]
    pub fn tool_result(call_identifier: String, content: String) -> Self {
        Self {
            tool_call_id: Some(call_identifier),
            ..Self::new("tool", content)
        }
    }
}
