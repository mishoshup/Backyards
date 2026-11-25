use crate::report::{GateError, pluralize};
use crate::review::ReviewerMode;
use crate::review::github::GitHub;
use crate::review::model::Violation;
use crate::review::post::finding_text::describe_finding_body;
use crate::review::post::inline_comment::PostTarget;

pub const MAXIMUM_SUMMARY_FINDINGS: usize = 100;

#[must_use]
pub fn build_summary_body(mode: ReviewerMode, violations: &[Violation]) -> String {
    let listed = violations.iter().take(MAXIMUM_SUMMARY_FINDINGS);
    let lines: Vec<String> = listed.map(describe_summary_line).collect();
    let omitted = violations.len().saturating_sub(MAXIMUM_SUMMARY_FINDINGS);
    format!(
        "Automated {} review: some findings could not be placed inline.\n\n{}{}",
        mode.label(),
        lines.join("\n"),
        describe_omitted(omitted)
    )
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
    let Some(first) = failures.first() else {
        return String::new();
    };
    let count = pluralize(
        failures.len(),
        "inline placement failed",
        "inline placements failed",
    );
    format!("\n\n{count}. First error: {first}")
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
