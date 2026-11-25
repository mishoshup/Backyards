use crate::report::write_report_line;
use crate::review::ReviewerMode;
use crate::review::github::{GitHub, ProgressComment, StatusState, StatusUpdate};
use crate::review::pipeline::options::ReviewRun;
use crate::review::post::ValidationOutcome;

const BOT_LOGIN: &str = "github-actions[bot]";

pub struct ReviewSummary {
    pub mode: ReviewerMode,
    pub accepted: usize,
    pub dropped: usize,
}

pub fn summarize(outcomes: &[(ReviewerMode, ValidationOutcome)]) -> Vec<ReviewSummary> {
    outcomes
        .iter()
        .map(|(mode, outcome)| ReviewSummary {
            mode: *mode,
            accepted: outcome.accepted.len(),
            dropped: outcome.dropped.len(),
        })
        .collect()
}

pub fn describe_summary(summary: &ReviewSummary) -> String {
    let noun = if summary.accepted == 1 {
        "finding"
    } else {
        "findings"
    };
    let count = if summary.accepted == 0 {
        "no findings, all ok".to_owned()
    } else {
        format!("{} {noun}", summary.accepted)
    };
    format!(
        "{}: {count} ({} dropped as invalid)",
        summary.mode.label(),
        summary.dropped
    )
}

pub fn finish_success(run: ReviewRun<'_>, comment: &ProgressComment, summaries: &[ReviewSummary]) {
    let lines: Vec<String> = summaries.iter().map(describe_summary).collect();
    let body = format!(
        "Agent review of {}: completed\n\n- {}",
        ProgressComment::short_sha(run.head_sha),
        lines.join("\n- ")
    );
    let clean = summaries.iter().all(|summary| summary.accepted == 0);
    update_comment_and_reactions(run.github, comment, &body, clean);
    record_status(
        run,
        StatusState::Success,
        "Reviewed at this commit - comment /agent-review to re-run",
    );
}

pub fn finish_failure(run: ReviewRun<'_>, comment: &ProgressComment, reason: &str) {
    let body = format!(
        "Agent review of {}: did not complete\n\n{reason}\n\nNothing was posted.",
        ProgressComment::short_sha(run.head_sha)
    );
    update_comment_and_reactions(run.github, comment, &body, false);
    record_status(run, StatusState::Failure, "Review did not complete");
}

pub fn record_status(run: ReviewRun<'_>, state: StatusState, description: &str) {
    let context = run.options.status_context();
    let update = StatusUpdate {
        head_sha: run.head_sha,
        context: &context,
        state,
        description,
    };
    report_if_failed("status update", run.github.set_status(&update));
}

fn update_comment_and_reactions(
    github: &GitHub,
    comment: &ProgressComment,
    body: &str,
    approve: bool,
) {
    report_if_failed(
        "progress comment update",
        github.update_progress_comment(comment, body),
    );
    report_if_failed(
        "in-progress reaction removal",
        github.remove_comment_reaction(&comment.identifier, "eyes", BOT_LOGIN),
    );
    if approve {
        report_if_failed(
            "approval reaction",
            github.add_comment_reaction(&comment.identifier, "+1"),
        );
    }
}

fn report_if_failed<Value, Failure: std::fmt::Display>(
    action: &str,
    result: Result<Value, Failure>,
) {
    if let Err(error) = result {
        write_report_line(&format!("warning: {action} failed: {error}")).ok();
    }
}
