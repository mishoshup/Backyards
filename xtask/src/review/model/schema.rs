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

impl Violation {
    #[must_use]
    pub fn describe_for_report(&self) -> String {
        let marker = match self.confidence {
            Confidence::High => "",
            Confidence::Low => " (low confidence)",
        };
        format!(
            "  {}:{} {}{marker}: {}",
            self.path, self.line, self.rule_identifier, self.body
        )
    }
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
    let json = extract_json_object(trimmed).unwrap_or(trimmed);
    serde_json::from_str(json).map_err(|source| ReviewError::InvalidAnswer {
        reason: source.to_string(),
    })
}

pub fn extract_json_object(text: &str) -> Option<&str> {
    let candidate = text.get(text.find('{')?..)?;
    let mut values =
        serde_json::Deserializer::from_str(candidate).into_iter::<serde::de::IgnoredAny>();
    values.next()?.ok()?;
    candidate.get(..values.byte_offset())
}
