use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::report::{GateError, pluralize, write_report_line};
use crate::review::diff::read_local_diff;
use crate::review::model::{extract_json_object, parse_answer};
use crate::review::pipeline::{ReviewContext, load_review_context};
use crate::review::post::{ValidationOutcome, validate_answer};
use crate::review::prompt::build_review_prompts;
use crate::review::worker::{ReviewerResult, run_each_reviewer_mode};
use crate::review::{ReviewError, ReviewerMode};
use crate::workspace::{Repository, capture_command};

#[derive(Debug, Clone)]
pub struct LocalOptions {
    pub base_reference: String,
    pub model: String,
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
    let prompts = build_review_prompts(repository, mode, &context.rules, &context.diff)?;
    let prompt_path = write_prompt_file(mode, &format!("{}\n\n{}", prompts.system, prompts.user))?;
    let output = run_opencode(repository, options, mode, &prompt_path);
    remove_prompt_file(&prompt_path)?;
    let answer = parse_opencode_answer(&output?)?;
    Ok(validate_answer(
        answer,
        mode,
        &context.added,
        &context.rules,
    ))
}

fn write_prompt_file(mode: ReviewerMode, prompt: &str) -> Result<PathBuf, GateError> {
    let path = std::env::temp_dir().join(format!(
        "backyards-review-{}-{}.prompt",
        mode.label(),
        std::process::id()
    ));
    write_private_file(&path, prompt).map_err(|source| GateError::ReadFile {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

fn write_private_file(path: &Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(content.as_bytes())
}

fn remove_prompt_file(path: &Path) -> Result<(), GateError> {
    std::fs::remove_file(path).map_err(|source| GateError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
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
    let json = extract_json_object(output).ok_or(ReviewError::EmptyAnswer)?;
    parse_answer(json)
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
