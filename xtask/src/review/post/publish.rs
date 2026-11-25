use crate::report::GateError;
use crate::review::ReviewerMode;
use crate::review::github::GitHub;
use crate::review::model::Violation;
use crate::review::post::inline_comment::{PostTarget, post_inline_comment};
use crate::review::post::summary_comment::post_summary_comment;

pub const MAXIMUM_POSTED_FINDINGS: usize = 30;

pub fn publish_findings(
    github: &GitHub,
    target: PostTarget<'_>,
    mode: ReviewerMode,
    accepted: &[Violation],
) -> Result<(), GateError> {
    let (posted_candidates, overflow) = split_at_cap(accepted);
    let mut unplaced = post_each_inline(github, target, posted_candidates);
    unplaced.extend_from_slice(overflow);
    if unplaced.is_empty() {
        return Ok(());
    }
    post_summary_comment(github, target, mode, &unplaced)
}

fn split_at_cap(accepted: &[Violation]) -> (&[Violation], &[Violation]) {
    accepted.split_at(accepted.len().min(MAXIMUM_POSTED_FINDINGS))
}

fn post_each_inline(
    github: &GitHub,
    target: PostTarget<'_>,
    violations: &[Violation],
) -> Vec<Violation> {
    let mut unplaced = Vec::new();
    for violation in violations {
        if post_inline_comment(github, target, violation).is_err() {
            unplaced.push(violation.clone());
        }
    }
    unplaced
}
