use crate::report::GateError;
use crate::review::github::{
    GitHub, RunPermission, decide_run_permission, has_reviewable_path, should_skip_review,
};
use crate::review::pipeline::options::CiOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunDecision {
    AlreadyReviewed,
    RunCapReached { previous_runs: usize },
    NoReviewablePaths,
    Proceed,
}

pub fn decide_run(
    github: &GitHub,
    options: &CiOptions,
    head_sha: &str,
) -> Result<RunDecision, GateError> {
    let existing = github.read_status_state(head_sha, &options.build_status_context())?;
    if should_skip_review(options.event_name, existing) {
        return Ok(RunDecision::AlreadyReviewed);
    }
    let previous_runs = github.count_progress_comments(options.pull_request)?;
    if let RunPermission::Refused { previous_runs } = decide_run_permission(
        previous_runs,
        &options.comment_body,
        options.comment_author_association,
    ) {
        return Ok(RunDecision::RunCapReached { previous_runs });
    }
    decide_by_changed_paths(github, options)
}

fn decide_by_changed_paths(github: &GitHub, options: &CiOptions) -> Result<RunDecision, GateError> {
    let changed = github.list_changed_paths(options.pull_request)?;
    Ok(if has_reviewable_path(&changed)? {
        RunDecision::Proceed
    } else {
        RunDecision::NoReviewablePaths
    })
}
