use std::collections::BTreeSet;
use std::path::Path;
use std::time::Instant;

use crate::report::{GateError, pluralize, write_report_line};
use crate::review::diff::{read_local_diff, split_diff_into_shards};
use crate::review::model::parse_answer;
use crate::review::pipeline::{ReviewContext, load_review_context};
use crate::review::post::{ValidationOutcome, validate_answer};
use crate::review::prompt::build_review_prompts;
use crate::review::prompt_file::PromptFile;
use crate::review::runner::{ReviewRunner, claude_tools};
use crate::review::worker::{ReviewerResult, run_bounded_shards, run_each_reviewer_mode};
use crate::review::{ReviewError, ReviewerMode};
use crate::workspace::{Repository, capture_command, capture_command_with_input_file};

const MAXIMUM_SHARD_BYTES: usize = 30_000;
const MAXIMUM_PARALLEL_SHARDS: usize = 6;

#[derive(Debug, Clone)]
pub struct LocalOptions {
    pub base_reference: String,
    pub model: String,
    pub runner: ReviewRunner,
}

pub fn run_local_review(repository: &Repository, options: &LocalOptions) -> Result<(), GateError> {
    let diff = read_local_diff(repository, &options.base_reference)?;
    let context = load_review_context(repository, diff)?;
    let outcomes = run_both_reviewers(repository, options, &context);
    report_outcomes(&outcomes)
}

fn run_both_reviewers(
    repository: &Repository,
    options: &LocalOptions,
    context: &ReviewContext<'_>,
) -> Vec<ReviewerResult> {
    run_each_reviewer_mode(|mode| run_one_reviewer(repository, options, context, mode))
}

fn run_one_reviewer(
    repository: &Repository,
    options: &LocalOptions,
    context: &ReviewContext<'_>,
    mode: ReviewerMode,
) -> Result<ValidationOutcome, GateError> {
    let started = Instant::now();
    let outcome = run_reviewer_shards(repository, options, context, mode);
    report_reviewer_duration(mode, started)?;
    outcome
}

fn report_reviewer_duration(mode: ReviewerMode, started: Instant) -> Result<(), GateError> {
    write_report_line(&format!(
        "{} reviewer finished in {}s",
        mode.label(),
        started.elapsed().as_secs()
    ))
}

fn run_reviewer_shards(
    repository: &Repository,
    options: &LocalOptions,
    context: &ReviewContext<'_>,
    mode: ReviewerMode,
) -> Result<ValidationOutcome, GateError> {
    let shards = choose_shards(context, mode);
    let results = run_bounded_shards(&shards, MAXIMUM_PARALLEL_SHARDS, |shard| {
        review_diff_text(repository, options, context, (mode, shard))
    });
    merge_shard_outcomes(results)
}

fn choose_shards(context: &ReviewContext<'_>, mode: ReviewerMode) -> Vec<String> {
    match mode {
        ReviewerMode::Inline => split_diff_into_shards(&context.diff, MAXIMUM_SHARD_BYTES),
        ReviewerMode::Holistic => vec![context.diff.clone()],
    }
}

fn merge_shard_outcomes(
    results: Vec<Result<ValidationOutcome, GateError>>,
) -> Result<ValidationOutcome, GateError> {
    let mut merged = ValidationOutcome::default();
    let mut first_failure = None;
    for result in results {
        match result {
            Ok(outcome) => {
                merged.accepted.extend(outcome.accepted);
                merged.dropped.extend(outcome.dropped);
            }
            Err(error) => first_failure = keep_first_and_report_rest(first_failure, error)?,
        }
    }
    match first_failure {
        Some(error) => Err(report_unreported_findings(&merged, error)?),
        None => Ok(merged),
    }
}

fn keep_first_and_report_rest(
    first_failure: Option<GateError>,
    error: GateError,
) -> Result<Option<GateError>, GateError> {
    if first_failure.is_some() {
        write_report_line(&format!("another shard failed: {error}"))?;
        return Ok(first_failure);
    }
    Ok(Some(error))
}

fn report_unreported_findings(
    merged: &ValidationOutcome,
    failure: GateError,
) -> Result<GateError, GateError> {
    write_report_line(&format!(
        "a shard failed; {} from the other shards follow",
        pluralize(merged.accepted.len(), "finding", "findings")
    ))?;
    for violation in &merged.accepted {
        write_report_line(&violation.describe_for_report())?;
    }
    Ok(failure)
}

fn review_diff_text(
    repository: &Repository,
    options: &LocalOptions,
    context: &ReviewContext<'_>,
    (mode, diff): (ReviewerMode, &str),
) -> Result<ValidationOutcome, GateError> {
    let prompts = build_review_prompts(repository, mode, &context.rules, diff)?;
    let prompt_file = PromptFile::create(mode, &format!("{}\n\n{}", prompts.system, prompts.user))?;
    let output = run_model_process(repository, options, mode, prompt_file.path());
    let removal = prompt_file.remove();
    let output = report_removal_before_failure(output, removal)?;
    let answer = parse_opencode_answer(&output)?;
    Ok(validate_answer(
        answer,
        mode,
        &context.added,
        &context.rules,
    ))
}

fn report_removal_before_failure(
    output: Result<String, GateError>,
    removal: Result<(), GateError>,
) -> Result<String, GateError> {
    match (output, removal) {
        (Ok(text), Ok(())) => Ok(text),
        (Ok(_), Err(removal_failure)) => {
            write_report_line("the model answer was discarded because prompt file removal failed")?;
            Err(removal_failure)
        }
        (Err(run_failure), Ok(())) => Err(run_failure),
        (Err(run_failure), Err(removal_failure)) => {
            write_report_line(&removal_failure.to_string())?;
            Err(run_failure)
        }
    }
}

fn run_model_process(
    repository: &Repository,
    options: &LocalOptions,
    mode: ReviewerMode,
    prompt_path: &Path,
) -> Result<String, GateError> {
    match options.runner {
        ReviewRunner::Opencode => run_opencode(repository, options, mode, prompt_path),
        ReviewRunner::Claude => run_claude(repository, options, mode, prompt_path),
    }
}

fn run_claude(
    repository: &Repository,
    options: &LocalOptions,
    mode: ReviewerMode,
    prompt_path: &Path,
) -> Result<String, GateError> {
    capture_command_with_input_file(
        repository.root(),
        options.runner.program(),
        &[
            "-p",
            mode.attachment_request(),
            "--model",
            &options.model,
            "--tools",
            claude_tools(mode),
            "--no-session-persistence",
        ],
        prompt_path,
    )
}

fn run_opencode(
    repository: &Repository,
    options: &LocalOptions,
    mode: ReviewerMode,
    prompt_path: &Path,
) -> Result<String, GateError> {
    let attachment = prompt_path.to_string_lossy();
    capture_command(
        repository.root(),
        "opencode",
        &[
            "run",
            "-m",
            &options.model,
            "-f",
            &attachment,
            mode.attachment_request(),
        ],
        &[0],
    )
}

fn parse_opencode_answer(output: &str) -> Result<crate::review::ReviewAnswer, ReviewError> {
    parse_answer(output)
}

fn report_outcomes(outcomes: &[ReviewerResult]) -> Result<(), GateError> {
    let mut finding_count = 0;
    let mut failed_modes: BTreeSet<&'static str> = BTreeSet::new();
    for (mode, outcome) in outcomes {
        match outcome {
            Ok(validated) => finding_count += report_validated(*mode, validated)?,
            Err(error) => {
                write_report_line(&format!("{} review failed: {error}", mode.label()))?;
                failed_modes.insert(mode.label());
            }
        }
    }
    if !failed_modes.is_empty() {
        return Err(GateError::StagesFailed {
            names: failed_modes.into_iter().collect(),
        });
    }
    if finding_count == 0 {
        Ok(())
    } else {
        Err(GateError::Violations {
            area: "local review",
            count: finding_count,
        })
    }
}

fn report_validated(mode: ReviewerMode, validated: &ValidationOutcome) -> Result<usize, GateError> {
    write_report_line(&format!(
        "{} review: {}, {} dropped",
        mode.label(),
        pluralize(validated.accepted.len(), "finding", "findings"),
        validated.dropped.len()
    ))?;
    for violation in &validated.accepted {
        write_report_line(&violation.describe_for_report())?;
    }
    for dropped in &validated.dropped {
        write_report_line(&format!("  dropped {}", dropped.describe()))?;
    }
    Ok(validated.accepted.len())
}
