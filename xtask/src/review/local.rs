use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::report::{GateError, pluralize, write_report_line};
use crate::review::diff::{AddedLines, parse_added_lines, read_local_diff};
use crate::review::model::{Confidence, Violation, extract_json_object, parse_answer};
use crate::review::post::{ValidationOutcome, validate_violations};
use crate::review::prompt::{
    build_system_prompt, build_user_prompt, collect_allowed_rule_identifiers,
    load_reviewer_instructions, load_rule_files,
};
use crate::review::worker::join_worker;
use crate::review::{ALL_REVIEWER_MODES, ReviewError, ReviewerMode};
use crate::workspace::{Repository, capture_command};

const HOLISTIC_REQUEST: &str = "Follow the instructions in the attached file.";
const INLINE_REQUEST: &str = "Follow the instructions in the attached file. The file holds the diff and every rule, so do not call tools or read other files. Answer with the JSON object only.";

#[derive(Debug, Clone)]
pub struct LocalOptions {
    pub base_reference: String,
    pub model: String,
}

struct LocalContext {
    diff: String,
    added: AddedLines,
    rules: Vec<crate::review::RuleFile>,
}

pub fn run_local_review(repository: &Repository, options: &LocalOptions) -> Result<(), GateError> {
    let context = load_context(repository, options)?;
    let outcomes = run_both_reviewers(repository, options, &context);
    report_outcomes(&outcomes)
}

fn load_context(
    repository: &Repository,
    options: &LocalOptions,
) -> Result<LocalContext, GateError> {
    let diff = read_local_diff(repository, &options.base_reference)?;
    let added = parse_added_lines(&diff)?;
    let rules = load_rule_files(repository)?;
    Ok(LocalContext { diff, added, rules })
}

fn run_both_reviewers(
    repository: &Repository,
    options: &LocalOptions,
    context: &LocalContext,
) -> Vec<(ReviewerMode, Result<ValidationOutcome, GateError>)> {
    std::thread::scope(|scope| {
        let workers: Vec<_> = ALL_REVIEWER_MODES
            .iter()
            .map(|mode| scope.spawn(|| run_one_reviewer(repository, options, context, *mode)))
            .collect();
        ALL_REVIEWER_MODES
            .iter()
            .zip(workers)
            .map(|(mode, worker)| (*mode, join_worker(worker)))
            .collect()
    })
}

fn run_one_reviewer(
    repository: &Repository,
    options: &LocalOptions,
    context: &LocalContext,
    mode: ReviewerMode,
) -> Result<ValidationOutcome, GateError> {
    let instructions = load_reviewer_instructions(repository, mode)?;
    let system = build_system_prompt(&instructions, mode, &context.rules);
    let user = build_user_prompt(mode, &context.diff);
    let prompt_path = write_prompt_file(mode, &format!("{system}\n\n{user}"))?;
    let output = run_opencode(repository, options, mode, &prompt_path);
    remove_prompt_file(&prompt_path)?;
    let answer = parse_opencode_answer(&output?)?;
    let allowed = collect_allowed_rule_identifiers(&context.rules);
    Ok(validate_violations(
        answer.violations,
        mode,
        &context.added,
        &allowed,
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
            select_attachment_request(mode),
        ],
        &[0],
    )
}

fn parse_opencode_answer(output: &str) -> Result<crate::review::ReviewAnswer, ReviewError> {
    let json = extract_json_object(output).ok_or(ReviewError::EmptyAnswer)?;
    parse_answer(json)
}

fn report_outcomes(
    outcomes: &[(ReviewerMode, Result<ValidationOutcome, GateError>)],
) -> Result<(), GateError> {
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
        write_report_line(&describe_violation(violation))?;
    }
    for dropped in &validated.dropped {
        write_report_line(&format!(
            "  dropped {}:{} ({})",
            dropped.violation.path, dropped.violation.line, dropped.reason
        ))?;
    }
    Ok(validated.accepted.len())
}

fn describe_violation(violation: &Violation) -> String {
    let marker = match violation.confidence {
        Confidence::High => "",
        Confidence::Low => " (low confidence)",
    };
    format!(
        "  {}:{} {}{marker}: {}",
        violation.path, violation.line, violation.rule_identifier, violation.body
    )
}

const fn select_attachment_request(mode: ReviewerMode) -> &'static str {
    match mode {
        ReviewerMode::Inline => INLINE_REQUEST,
        ReviewerMode::Holistic => HOLISTIC_REQUEST,
    }
}
