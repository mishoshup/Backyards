use crate::report::{GateError, pluralize, write_report_line};
use crate::review::ReviewerMode;
use crate::review::github::{BOT_LOGIN, GitHub, ProgressComment, StatusState, StatusUpdate};
use crate::review::pipeline::options::ReviewRun;
use crate::review::post::{DroppedFinding, ValidationOutcome};

const MAXIMUM_LISTED_DROPPED: usize = 20;

pub struct ReviewSummary {
    pub mode: ReviewerMode,
    pub accepted: usize,
    pub dropped: usize,
    pub dropped_details: Vec<String>,
}

pub fn summarize(outcomes: &[(ReviewerMode, ValidationOutcome)]) -> Vec<ReviewSummary> {
    outcomes
        .iter()
        .map(|(mode, outcome)| ReviewSummary {
            mode: *mode,
            accepted: outcome.accepted.len(),
            dropped: outcome.dropped.len(),
            dropped_details: describe_dropped_findings(&outcome.dropped),
        })
        .collect()
}

fn describe_dropped_findings(dropped: &[DroppedFinding]) -> Vec<String> {
    let mut details: Vec<String> = dropped
        .iter()
        .take(MAXIMUM_LISTED_DROPPED)
        .map(describe_dropped)
        .collect();
    let omitted = dropped.len().saturating_sub(MAXIMUM_LISTED_DROPPED);
    if omitted > 0 {
        details.push(format!("  - {omitted} further dropped findings not listed"));
    }
    details
}

fn describe_dropped(dropped: &DroppedFinding) -> String {
    format!("  - {}", dropped.describe())
}

pub fn describe_summary(summary: &ReviewSummary) -> String {
    let count = if summary.accepted == 0 {
        "no findings, all ok".to_owned()
    } else {
        pluralize(summary.accepted, "finding", "findings")
    };
    let headline = format!(
        "{}: {count} ({} dropped as invalid)",
        summary.mode.label(),
        summary.dropped
    );
    if summary.dropped_details.is_empty() {
        return headline;
    }
    format!("{headline}\n{}", summary.dropped_details.join("\n"))
}

pub fn finish_success(
    run: ReviewRun<'_>,
    comment: &ProgressComment,
    summaries: &[ReviewSummary],
) -> Result<(), GateError> {
    let lines: Vec<String> = summaries.iter().map(describe_summary).collect();
    let body = format!(
        "Agent review of {}: completed\n\n- {}",
        ProgressComment::short_sha(run.head_sha),
        lines.join("\n- ")
    );
    let is_clean = summaries.iter().all(|summary| summary.accepted == 0);
    update_comment_and_reactions(run.github, comment, &body, is_clean)?;
    record_status(
        run,
        StatusState::Success,
        "Reviewed at this commit - comment /agent-review to re-run",
    )
}

pub fn finish_failure(
    run: ReviewRun<'_>,
    comment: &ProgressComment,
    reason: &str,
) -> Result<(), GateError> {
    let body = format!(
        "Agent review of {}: did not complete\n\n{reason}\n\nComments posted before the failure remain on the pull request.",
        ProgressComment::short_sha(run.head_sha)
    );
    update_comment_and_reactions(run.github, comment, &body, false)?;
    record_status(run, StatusState::Failure, "Review did not complete")
}

pub fn record_status(
    run: ReviewRun<'_>,
    state: StatusState,
    description: &str,
) -> Result<(), GateError> {
    let context = run.options.status_context();
    let update = StatusUpdate {
        head_sha: run.head_sha,
        context: &context,
        state,
        description,
    };
    run.github.set_status(&update)
}

fn update_comment_and_reactions(
    github: &GitHub,
    comment: &ProgressComment,
    body: &str,
    should_approve: bool,
) -> Result<(), GateError> {
    report_if_failed(
        "progress comment update",
        github.update_progress_comment(comment, body),
    )?;
    report_if_failed(
        "in-progress reaction removal",
        github.remove_comment_reaction(&comment.identifier, "eyes", BOT_LOGIN),
    )?;
    if should_approve {
        report_if_failed(
            "approval reaction",
            github.add_comment_reaction(&comment.identifier, "+1"),
        )?;
    }
    Ok(())
}

fn report_if_failed<Value, Failure: std::fmt::Display>(
    action: &str,
    result: Result<Value, Failure>,
) -> Result<(), GateError> {
    match result {
        Ok(_) => Ok(()),
        Err(error) => write_report_line(&format!("warning: {action} failed: {error}")),
    }
}
