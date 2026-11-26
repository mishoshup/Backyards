use crate::report::{GateError, pluralize, write_report_line};
use crate::review::ReviewError;
use crate::review::ReviewerMode;
use crate::review::github::{FORCE_PHRASE, GitHub, MAXIMUM_RUNS, ProgressComment, StatusState};
use crate::review::pipeline::deadline::with_deadline;
use crate::review::pipeline::decide::{RunDecision, decide_run};
use crate::review::pipeline::finish::{
    ReviewSummary, finish_failure, finish_success, record_status, summarize,
};
use crate::review::pipeline::options::{CiOptions, ReviewRun};
use crate::review::pipeline::reviewers::{ReviewerResult, load_review_context, run_both_reviewers};
use crate::review::post::{PostTarget, ValidationOutcome, publish_findings};
use crate::workspace::{Repository, capture_command};

pub fn run_ci_review(repository: &Repository, options: &CiOptions) -> Result<(), GateError> {
    let github = GitHub::new(options.repository.clone(), repository.root().to_path_buf());
    let head_sha = read_head_sha(repository)?;
    match decide_run(&github, options, &head_sha)? {
        RunDecision::AlreadyReviewed => report("this commit is already reviewed; skipping"),
        RunDecision::RunCapReached { previous_runs } => refuse_run(&github, options, previous_runs),
        RunDecision::NoReviewablePaths => {
            record_status(
                ReviewRun {
                    github: &github,
                    options,
                    head_sha: &head_sha,
                },
                StatusState::Success,
                "Skipped - no reviewable files changed",
            )?;
            report("no reviewable files changed; skipping")
        }
        RunDecision::Proceed => run_reviewed(repository, &github, options, &head_sha),
    }
}

fn report(line: &str) -> Result<(), GateError> {
    write_report_line(line)
}

fn read_head_sha(repository: &Repository) -> Result<String, GateError> {
    let output = capture_command(repository.root(), "git", &["rev-parse", "HEAD"], &[0])?;
    Ok(output.trim().to_owned())
}

fn refuse_run(github: &GitHub, options: &CiOptions, previous_runs: usize) -> Result<(), GateError> {
    let body = format!(
        "Agent review is limited to {MAXIMUM_RUNS} runs per pull request and {previous_runs} have run. Comment `{FORCE_PHRASE}` to run it again."
    );
    let body_field = format!("body={body}");
    github.call_api(
        &format!("issues/{}/comments", options.pull_request),
        &["-X", "POST", "-f", &body_field],
    )?;
    report("run cap reached; refusal comment posted")
}

fn run_reviewed(
    repository: &Repository,
    github: &GitHub,
    options: &CiOptions,
    head_sha: &str,
) -> Result<(), GateError> {
    let run = ReviewRun {
        github,
        options,
        head_sha,
    };
    let comment = create_progress(run)?;
    match with_deadline(run, &comment, || {
        mark_started_then_review(repository, run, &comment)
    }) {
        Ok(summaries) => finish_success(run, &comment, &summaries)
            .map_err(|error| record_failure_keeping_cause(run, &comment, error)),
        Err(error) => Err(record_failure_keeping_cause(run, &comment, error)),
    }
}

fn record_failure_keeping_cause(
    run: ReviewRun<'_>,
    comment: &ProgressComment,
    cause: GateError,
) -> GateError {
    match finish_failure(run, comment, &cause.to_string()) {
        Ok(()) => cause,
        Err(recording) => ReviewError::FailureNotRecorded {
            cause: cause.to_string(),
            recording: recording.to_string(),
        }
        .into(),
    }
}

fn create_progress(run: ReviewRun<'_>) -> Result<ProgressComment, GateError> {
    run.github
        .create_progress_comment(run.options.pull_request, run.head_sha)
}

fn mark_started_then_review(
    repository: &Repository,
    run: ReviewRun<'_>,
    comment: &ProgressComment,
) -> Result<Vec<ReviewSummary>, GateError> {
    mark_started(run, comment)?;
    review_and_publish(repository, run)
}

fn mark_started(run: ReviewRun<'_>, comment: &ProgressComment) -> Result<(), GateError> {
    run.github
        .add_comment_reaction(&comment.identifier, "eyes")?;
    record_status(run, StatusState::Pending, "Review in progress")
}

fn review_and_publish(
    repository: &Repository,
    run: ReviewRun<'_>,
) -> Result<Vec<ReviewSummary>, GateError> {
    let diff = run
        .github
        .read_pull_request_diff(run.options.pull_request, run.head_sha)?;
    let context = load_review_context(repository, diff)?;
    let split = split_results(run_both_reviewers(&context));
    let publication = publish_all(run, &split.outcomes);
    let summaries = summarize(&split.outcomes);
    fail_if_anything_failed(&split.failures, &summaries, publication)?;
    Ok(summaries)
}

struct SplitResults {
    outcomes: Vec<(ReviewerMode, ValidationOutcome)>,
    failures: Vec<(ReviewerMode, GateError)>,
}

fn split_results(results: Vec<ReviewerResult>) -> SplitResults {
    let mut split = SplitResults {
        outcomes: Vec::new(),
        failures: Vec::new(),
    };
    for (mode, result) in results {
        match result {
            Ok(outcome) => split.outcomes.push((mode, outcome)),
            Err(error) => split.failures.push((mode, error)),
        }
    }
    split
}

fn fail_if_anything_failed(
    failures: &[(ReviewerMode, GateError)],
    summaries: &[ReviewSummary],
    publication: Result<(), GateError>,
) -> Result<(), GateError> {
    let mut reasons: Vec<String> = failures
        .iter()
        .map(|(mode, error)| format!("{} reviewer failed: {error}", mode.label()))
        .collect();
    if let Err(error) = publication {
        reasons.push(format!("publishing findings failed: {error}"));
    }
    if reasons.is_empty() {
        return Ok(());
    }
    let posted_reviewers = summaries.len();
    let completed: Vec<String> = summaries.iter().map(ReviewSummary::describe).collect();
    Err(ReviewError::ReviewersFailed {
        details: format!(
            "{}. Completed {}: {}. Run `/agent-review` to retry; after {MAXIMUM_RUNS} runs the owner must use `/agent-review force`.",
            reasons.join("; "),
            pluralize(posted_reviewers, "reviewer", "reviewers"),
            completed.join(" | ")
        ),
    }
    .into())
}

fn publish_all(
    run: ReviewRun<'_>,
    outcomes: &[(ReviewerMode, ValidationOutcome)],
) -> Result<(), GateError> {
    let target = PostTarget {
        pull_request: run.options.pull_request,
        head_sha: run.head_sha,
    };
    let mut failures = Vec::new();
    for (mode, outcome) in outcomes {
        if let Err(error) = publish_findings(run.github, target, *mode, &outcome.accepted) {
            failures.push(error.to_string());
        }
    }
    fail_if_any_publish_failed(&failures)
}

fn fail_if_any_publish_failed(failures: &[String]) -> Result<(), GateError> {
    if failures.is_empty() {
        return Ok(());
    }
    Err(ReviewError::ReviewersFailed {
        details: failures.join("\n"),
    }
    .into())
}
