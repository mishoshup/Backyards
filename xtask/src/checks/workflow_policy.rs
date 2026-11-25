use std::path::Path;

use crate::report::Finding;
use crate::report::GateError;
use crate::workspace::{Repository, compile_pattern};

const AREA: &str = "workflow";
const REVIEW_WORKFLOW: &str = ".github/workflows/agent-code-review.yml";
const REQUIRED_TIMEOUT_MINUTES: u32 = 25;
const PIPELINE_COMMAND: &str = "cargo xtask review pull-request";
const FORBIDDEN_TRIGGER: &str = "synchronize";
const REQUIRED_TRIGGER_TYPES: &str = "types: [opened, ready_for_review]";
const TIMEOUT_PATTERN: &str = r"timeout-minutes:\s*(\d+)";

pub fn check_workflow_policy(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let workflow = repository.read_text(Path::new(REVIEW_WORKFLOW))?;
    let mut findings = Vec::new();
    findings.extend(check_runs_are_not_cancelled(&workflow));
    findings.extend(check_timeout_covers_worst_case(&workflow)?);
    findings.extend(check_default_permissions_are_empty(&workflow));
    findings.extend(check_reviewed_commit_is_pinned(&workflow));
    findings.extend(check_fork_pull_requests_are_refused(&workflow));
    findings.extend(check_review_does_not_run_on_every_push(&workflow));
    findings.extend(check_review_uses_the_rust_pipeline(&workflow));
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
    if longest == REQUIRED_TIMEOUT_MINUTES {
        return Ok(Vec::new());
    }
    Ok(create_violation(&format!(
        "review timeout must be {REQUIRED_TIMEOUT_MINUTES} minutes"
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

fn check_reviewed_commit_is_pinned(workflow: &str) -> Vec<Finding> {
    if workflow.contains("ref: refs/pull/") {
        return Vec::new();
    }
    create_violation("checkout must name the pull request ref")
}

fn check_fork_pull_requests_are_refused(workflow: &str) -> Vec<Finding> {
    if workflow.contains("head.repo.full_name == github.repository") {
        return Vec::new();
    }
    create_violation("fork pull requests must be refused")
}

fn check_review_does_not_run_on_every_push(workflow: &str) -> Vec<Finding> {
    if workflow.contains(FORBIDDEN_TRIGGER) || !workflow.contains(REQUIRED_TRIGGER_TYPES) {
        return create_violation(
            "review must trigger only on opened and ready_for_review pull request events",
        );
    }
    Vec::new()
}

fn check_review_uses_the_rust_pipeline(workflow: &str) -> Vec<Finding> {
    if workflow.contains(PIPELINE_COMMAND) {
        return Vec::new();
    }
    create_violation("review must run through cargo xtask review pull-request")
}
