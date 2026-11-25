use crate::review::model::{Confidence, Violation};

#[must_use]
pub fn describe_finding_body(violation: &Violation) -> String {
    match violation.confidence {
        Confidence::High => violation.body.clone(),
        Confidence::Low => format!("Low confidence: {}", violation.body),
    }
}
