use crate::report::GateError;
use crate::review::ReviewerMode;
use crate::review::github::GitHub;
use crate::review::model::Violation;
use crate::review::post::inline_comment::{PostTarget, post_inline_comment};
use crate::review::post::summary_comment::{InlinePlacement, post_summary_comment};

pub const MAXIMUM_POSTED_FINDINGS: usize = 30;

pub fn publish_findings(
    github: &GitHub,
    target: PostTarget<'_>,
    mode: ReviewerMode,
    accepted: &[Violation],
) -> Result<(), GateError> {
    let (posted_candidates, overflow) = split_at_cap(accepted);
    let mut placement = post_each_inline(github, target, posted_candidates);
    placement.unplaced.extend_from_slice(overflow);
    if placement.unplaced.is_empty() {
        return Ok(());
    }
    post_summary_comment(github, target, mode, &placement)
}

fn split_at_cap(accepted: &[Violation]) -> (&[Violation], &[Violation]) {
    accepted.split_at(accepted.len().min(MAXIMUM_POSTED_FINDINGS))
}

fn post_each_inline(
    github: &GitHub,
    target: PostTarget<'_>,
    violations: &[Violation],
) -> InlinePlacement {
    let mut placement = InlinePlacement::default();
    for violation in violations {
        if let Err(error) = post_inline_comment(github, target, violation) {
            placement.unplaced.push(violation.clone());
            placement.failures.push(describe_failure(&error));
        }
    }
    placement
}

fn describe_failure(error: &GateError) -> String {
    if let GateError::CommandFailed { status, stderr, .. } = error {
        return format!("{status}: {}", stderr.lines().next().unwrap_or_default());
    }
    error.to_string()
}
