use crate::checks::check_architecture;
use crate::checks::check_policy;
use crate::checks::check_workflow_policy;
use crate::report::GateError;
use crate::report::{Finding, report_findings};
use crate::workspace::Repository;
use crate::workspace::run_command;

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
    let workflows = list_workflow_paths(repository)?;
    let arguments: Vec<&str> = workflows.iter().map(String::as_str).collect();
    run_command(repository.root(), "actionlint", &arguments)
}

fn list_workflow_paths(repository: &Repository) -> Result<Vec<String>, GateError> {
    Ok(repository
        .list_files(&["yml"])?
        .iter()
        .filter(|path| path.starts_with(".github/workflows"))
        .map(|path| path.display().to_string())
        .collect())
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
