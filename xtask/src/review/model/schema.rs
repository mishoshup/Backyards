use serde::Deserialize;

use crate::review::error::ReviewError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Violation {
    #[serde(rename = "ruleId")]
    pub rule_identifier: String,
    pub path: String,
    pub line: u32,
    pub body: String,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewAnswer {
    pub violations: Vec<Violation>,
}

pub fn parse_answer(text: &str) -> Result<ReviewAnswer, ReviewError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ReviewError::EmptyAnswer);
    }
    serde_json::from_str(trimmed).map_err(|source| ReviewError::InvalidAnswer {
        reason: source.to_string(),
    })
}

pub fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    text.get(start..=end)
}
