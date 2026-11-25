use crate::report::{GateError, pluralize};
use crate::review::ReviewerMode;
use crate::review::github::GitHub;
use crate::review::model::Violation;
use crate::review::post::finding_text::describe_finding_body;
use crate::review::post::inline_comment::PostTarget;

const MAXIMUM_LISTED_PLACEMENT_FAILURES: usize = 5;

pub const MAXIMUM_SUMMARY_FINDINGS: usize = 100;
const MAXIMUM_SUMMARY_BYTES: usize = 60_000;

#[must_use]
pub fn build_summary_body(mode: ReviewerMode, violations: &[Violation]) -> String {
    let candidates = violations.iter().take(MAXIMUM_SUMMARY_FINDINGS);
    let lines = keep_lines_within_budget(candidates.map(describe_summary_line));
    let omitted = violations.len().saturating_sub(lines.len());
    format!(
        "Automated {} review: some findings could not be placed inline.\n\n{}{}",
        mode.label(),
        lines.join("\n"),
        describe_omitted(omitted)
    )
}

fn keep_lines_within_budget(lines: impl Iterator<Item = String>) -> Vec<String> {
    let mut used_bytes = 0;
    lines
        .take_while(|line| {
            used_bytes += line.len() + 1;
            used_bytes <= MAXIMUM_SUMMARY_BYTES
        })
        .collect()
}

fn describe_summary_line(violation: &Violation) -> String {
    format!(
        "- `{}:{}` {}",
        violation.path,
        violation.line,
        describe_finding_body(violation)
    )
}

fn describe_omitted(omitted: usize) -> String {
    if omitted == 0 {
        return String::new();
    }
    format!(
        "\n\n{} omitted.",
        pluralize(omitted, "further finding", "further findings")
    )
}

#[derive(Debug, Default)]
pub struct InlinePlacement {
    pub unplaced: Vec<Violation>,
    pub failures: Vec<String>,
}

fn describe_placement_failures(failures: &[String]) -> String {
    if failures.is_empty() {
        return String::new();
    }
    let count = pluralize(
        failures.len(),
        "inline placement failed",
        "inline placements failed",
    );
    let listed = failures
        .iter()
        .take(MAXIMUM_LISTED_PLACEMENT_FAILURES)
        .map(|failure| format!("- {failure}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("\n\n{count}:\n{listed}")
}

pub fn post_summary_comment(
    github: &GitHub,
    target: PostTarget<'_>,
    mode: ReviewerMode,
    placement: &InlinePlacement,
) -> Result<(), GateError> {
    let body_field = format!(
        "body={}{}",
        build_summary_body(mode, &placement.unplaced),
        describe_placement_failures(&placement.failures)
    );
    github
        .post_content(
            &format!("issues/{}/comments", target.pull_request),
            &["-X", "POST", "-f", &body_field],
        )
        .map(|_| ())
}
