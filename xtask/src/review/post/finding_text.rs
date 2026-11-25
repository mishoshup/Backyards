use crate::review::model::{Confidence, Violation};

const MAXIMUM_FINDING_BODY_BYTES: usize = 2_000;
const TRUNCATION_NOTICE: &str = " [truncated]";

#[must_use]
pub fn describe_finding_body(violation: &Violation) -> String {
    let body = truncate_at_character_boundary(&violation.body, MAXIMUM_FINDING_BODY_BYTES);
    match violation.confidence {
        Confidence::High => body,
        Confidence::Low => format!("Low confidence: {body}"),
    }
}

fn truncate_at_character_boundary(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{TRUNCATION_NOTICE}", &text[..end])
}
