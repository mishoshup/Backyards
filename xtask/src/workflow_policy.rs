use std::path::Path;

use crate::error::GateError;
use crate::finding::Finding;
use crate::repository::{Repository, compile_pattern};

const AREA: &str = "workflow";
const REVIEW_WORKFLOW: &str = ".github/workflows/agent-code-review.yml";
const SKIP_SCRIPT: &str = ".github/scripts/review/should-skip-review.sh";
const ALLOWLIST: &str = "scripts/review/allowed-fetch-hosts.txt";
const LOCAL_REVIEW_SCRIPT: &str = "scripts/review/run-local-review.sh";
const MINIMUM_TIMEOUT_MINUTES: u32 = 45;
const TIMEOUT_PATTERN: &str = r"timeout-minutes:\s*(\d+)";

pub fn check_workflow_policy(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let workflow = repository.read_text(Path::new(REVIEW_WORKFLOW))?;
    let mut findings = Vec::new();
    findings.extend(check_runs_are_not_cancelled(&workflow));
    findings.extend(check_timeout_covers_worst_case(&workflow)?);
    findings.extend(check_default_permissions_are_empty(&workflow));
    findings.extend(check_reviewed_commit_is_pinned(repository, &workflow)?);
    findings.extend(check_fork_pull_requests_are_refused(&workflow));
    findings.extend(check_allowlist_is_present(repository)?);
    findings.extend(check_local_review_attaches_prompt_file(repository)?);
    Ok(findings)
}

fn create_violation(message: &str) -> Vec<Finding> {
    vec![Finding::error(AREA, message.to_owned())]
}

fn check_runs_are_not_cancelled(workflow: &str) -> Vec<Finding> {
    if workflow.contains("cancel-in-progress: true") {
        return create_violation("review runs must not cancel in progress");
    }
    Vec::new()
}

fn check_timeout_covers_worst_case(workflow: &str) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(TIMEOUT_PATTERN)?;
    let Ok(timeouts) = pattern
        .captures_iter(workflow)
        .map(|captures| captures[1].parse::<u32>())
        .collect::<Result<Vec<u32>, _>>()
    else {
        return Ok(create_violation("review timeout is not a valid number"));
    };
    let longest = timeouts.into_iter().max().unwrap_or(0);
    if longest >= MINIMUM_TIMEOUT_MINUTES {
        return Ok(Vec::new());
    }
    Ok(create_violation(&format!(
        "review timeout must be at least {MINIMUM_TIMEOUT_MINUTES} minutes"
    )))
}

fn check_default_permissions_are_empty(workflow: &str) -> Vec<Finding> {
    if workflow
        .lines()
        .any(|line| line.starts_with("permissions: {}"))
    {
        return Vec::new();
    }
    create_violation("workflow default permissions must be empty")
}

fn check_reviewed_commit_is_pinned(
    repository: &Repository,
    workflow: &str,
) -> Result<Vec<Finding>, GateError> {
    let mut findings = Vec::new();
    if !workflow.contains("ref: refs/pull/") {
        findings.extend(create_violation("checkout must name the pull request ref"));
    }
    let skip_script = repository.read_text(Path::new(SKIP_SCRIPT))?;
    if !skip_script.contains("rev-parse HEAD") {
        findings.extend(create_violation(
            "head SHA must come from the checked-out commit",
        ));
    }
    Ok(findings)
}

fn check_fork_pull_requests_are_refused(workflow: &str) -> Vec<Finding> {
    if workflow.contains("head.repo.full_name == github.repository") {
        return Vec::new();
    }
    create_violation("fork pull requests must be refused")
}

fn check_allowlist_is_present(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let listed = repository.list_files(&["txt"])?;
    if listed.iter().any(|path| path == Path::new(ALLOWLIST)) {
        return Ok(Vec::new());
    }
    Ok(create_violation(
        "allowed-fetch-hosts.txt must be listed by git",
    ))
}

fn check_local_review_attaches_prompt_file(
    repository: &Repository,
) -> Result<Vec<Finding>, GateError> {
    let script = repository.read_text(Path::new(LOCAL_REVIEW_SCRIPT))?;
    if script.contains("-f \"$") {
        return Ok(Vec::new());
    }
    Ok(create_violation(
        "local review must attach the prompt as a file",
    ))
}
