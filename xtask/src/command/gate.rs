use crate::checks::check_architecture;
use crate::checks::check_policy;
use crate::checks::check_workflow_policy;
use crate::report::GateError;
use crate::report::{Finding, report_findings, write_report_line};
use crate::workspace::Repository;
use crate::workspace::run_command;

struct Stage {
    name: &'static str,
    remedy: &'static str,
    run: fn(&Repository) -> Result<(), GateError>,
}

const STAGES: [Stage; 8] = [
    Stage {
        name: "workflow lint",
        remedy: "edit the workflow lines reported by actionlint",
        run: run_workflow_lint,
    },
    Stage {
        name: "workflow policy",
        remedy: "bring the review workflow back to the listed invariants",
        run: run_workflow_policy,
    },
    Stage {
        name: "policy",
        remedy: "remove the listed constructs",
        run: run_policy,
    },
    Stage {
        name: "architecture",
        remedy: "move or split the listed files and dependencies",
        run: run_architecture,
    },
    Stage {
        name: "formatting",
        remedy: "run `cargo fmt`",
        run: run_formatting_check,
    },
    Stage {
        name: "lint",
        remedy: "run `cargo clippy --workspace --all-targets` and fix the warnings",
        run: run_lint_check,
    },
    Stage {
        name: "tests",
        remedy: "run `cargo test --workspace` and fix the failures",
        run: run_tests,
    },
    Stage {
        name: "dependencies",
        remedy: "run `cargo deny check` and resolve the reported advisories or licenses",
        run: run_dependency_check,
    },
];

pub fn run_gate(repository: &Repository, should_keep_going: bool) -> Result<(), GateError> {
    let failed = run_stages(repository, should_keep_going)?;
    if failed.is_empty() {
        Ok(())
    } else {
        Err(GateError::StagesFailed { names: failed })
    }
}

fn run_stages(
    repository: &Repository,
    should_keep_going: bool,
) -> Result<Vec<&'static str>, GateError> {
    let mut failed = Vec::new();
    for (index, stage) in STAGES.iter().enumerate() {
        if !run_stage(stage, repository)? {
            failed.push(stage.name);
            if !should_keep_going {
                report_skipped_stages(index + 1)?;
                break;
            }
        }
    }
    Ok(failed)
}

fn report_skipped_stages(first_skipped_index: usize) -> Result<(), GateError> {
    let skipped: Vec<&str> = STAGES[first_skipped_index..]
        .iter()
        .map(|stage| stage.name)
        .collect();
    if skipped.is_empty() {
        return Ok(());
    }
    write_report_line(&format!(
        "skipped stages (rerun with --keep-going to run them): {}",
        skipped.join(", ")
    ))
}

fn run_stage(stage: &Stage, repository: &Repository) -> Result<bool, GateError> {
    match (stage.run)(repository) {
        Ok(()) => Ok(true),
        Err(error) => {
            write_report_line(&error.to_string())?;
            write_report_line(&format!("{}: to fix, {}", stage.name, stage.remedy))?;
            Ok(false)
        }
    }
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
