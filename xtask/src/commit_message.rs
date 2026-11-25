use std::path::Path;

use crate::error::GateError;
use crate::finding::Finding;
use crate::repository::{Repository, compile_pattern};

const AREA: &str = "commit message";
const MAXIMUM_SUBJECT_LENGTH: usize = 72;
const SUBJECT_PATTERN: &str =
    r"^(feat|fix|docs|refactor|test|chore|build|ci|perf)\([a-z0-9-]+\): [a-z].*[^.]$";
const TRAILER_PATTERN: &str = r"(?mi)^(Co-Authored-By|Claude-Session):";

pub fn check_commit_message_file(
    repository: &Repository,
    message_file: &Path,
) -> Result<Vec<Finding>, GateError> {
    let message = repository.read_text(message_file)?;
    check_commit_message(&message)
}

pub fn check_commit_message(message: &str) -> Result<Vec<Finding>, GateError> {
    let subject = message.lines().next().unwrap_or_default();
    let mut findings = Vec::new();
    findings.extend(check_no_forbidden_trailers(message)?);
    findings.extend(check_subject_length(subject));
    findings.extend(check_subject_format(subject)?);
    Ok(findings)
}

fn check_no_forbidden_trailers(message: &str) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(TRAILER_PATTERN)?;
    if pattern.is_match(message) {
        return Ok(vec![Finding::error(
            AREA,
            "Co-Authored-By and Claude-Session trailers are not allowed".to_owned(),
        )]);
    }
    Ok(Vec::new())
}

fn check_subject_length(subject: &str) -> Vec<Finding> {
    let length = subject.chars().count();
    if length <= MAXIMUM_SUBJECT_LENGTH {
        return Vec::new();
    }
    vec![Finding::error(
        AREA,
        format!("subject has {length} characters, the limit is {MAXIMUM_SUBJECT_LENGTH}"),
    )]
}

fn check_subject_format(subject: &str) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(SUBJECT_PATTERN)?;
    if pattern.is_match(subject) {
        return Ok(Vec::new());
    }
    Ok(vec![Finding::error(
        AREA,
        "subject must match <type>(<scope>): <lowercase imperative subject> with no trailing period"
            .to_owned(),
    )])
}
