use crate::architecture::check_architecture;
use crate::error::GateError;
use crate::finding::{Finding, report_findings};
use crate::policy::check_policy;
use crate::process::run_command;
use crate::repository::Repository;
use crate::workflow_policy::check_workflow_policy;

type Stage = fn(&Repository) -> Result<(), GateError>;

const STAGES: [Stage; 10] = [
    run_workflow_lint,
    run_workflow_policy,
    run_shell_check,
    run_review_script_tests,
    run_policy,
    run_architecture,
    run_formatting_check,
    run_lint_check,
    run_tests,
    run_dependency_check,
];

pub fn run_gate(repository: &Repository) -> Result<(), GateError> {
    for stage in STAGES {
        stage(repository)?;
    }
    Ok(())
}

pub fn run_policy(repository: &Repository) -> Result<(), GateError> {
    report_findings("policy", &check_policy(repository)?)
}

pub fn run_architecture(repository: &Repository) -> Result<(), GateError> {
    report_findings("architecture", &check_architecture(repository)?)
}

pub fn run_workflow_policy(repository: &Repository) -> Result<(), GateError> {
    report_findings("workflow", &check_workflow_policy(repository)?)
}

pub fn report_check(area: &'static str, findings: &[Finding]) -> Result<(), GateError> {
    report_findings(area, findings)
}

fn run_workflow_lint(repository: &Repository) -> Result<(), GateError> {
    run_command(
        repository.root(),
        "actionlint",
        &[
            ".github/workflows/agent-code-review.yml",
            ".github/workflows/ci.yml",
            ".github/workflows/pull-request.yml",
        ],
    )
}

fn run_shell_check(repository: &Repository) -> Result<(), GateError> {
    run_command(repository.root(), "scripts/check-shell.sh", &[])
}

fn run_review_script_tests(repository: &Repository) -> Result<(), GateError> {
    run_command(repository.root(), "scripts/test-review-scripts.sh", &[])
}

fn run_formatting_check(repository: &Repository) -> Result<(), GateError> {
    run_command(
        repository.root(),
        "cargo",
        &["fmt", "--all", "--", "--check"],
    )
}

fn run_lint_check(repository: &Repository) -> Result<(), GateError> {
    run_command(
        repository.root(),
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )
}

fn run_tests(repository: &Repository) -> Result<(), GateError> {
    run_command(repository.root(), "cargo", &["test", "--workspace"])
}

fn run_dependency_check(repository: &Repository) -> Result<(), GateError> {
    run_command(repository.root(), "cargo", &["deny", "check"])
}
