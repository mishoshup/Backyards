use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commit_message::check_commit_message_file;
use crate::error::GateError;
use crate::finding::Finding;
use crate::gate::{report_check, run_architecture, run_gate, run_policy, run_workflow_policy};
use crate::pull_request::{check_pull_request_description, check_pull_request_naming};
use crate::repository::Repository;

#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "Repository tooling for Backyards",
    version,
    propagate_version = true
)]
pub struct Cli {
    /// Repository to check, defaults to the repository containing xtask
    #[arg(long, global = true, env = "XTASK_REPOSITORY_ROOT")]
    pub repository_root: Option<PathBuf>,

    #[command(subcommand)]
    pub task: Task,
}

#[derive(Debug, Subcommand)]
pub enum Task {
    /// Run every check the repository requires before a commit
    Check,
    /// Check marker comments, em dashes, lint inheritance and opt-outs
    Policy,
    /// Check crate layering, file names and file lengths
    Architecture,
    /// Check the invariants of the review workflow
    WorkflowPolicy,
    /// Check a commit message file
    CommitMessage { message_file: PathBuf },
    /// Check a pull request title and branch name
    PullRequestNaming {
        #[arg(long, env = "PULL_REQUEST_TITLE")]
        title: String,
        #[arg(long, env = "PULL_REQUEST_BRANCH")]
        branch: String,
    },
    /// Check a pull request description
    PullRequestDescription {
        #[arg(long, env = "PULL_REQUEST_BODY", default_value = "")]
        body: String,
    },
}

pub fn run_task(task: &Task, repository: &Repository) -> Result<(), GateError> {
    match task {
        Task::Check => run_gate(repository),
        Task::Policy => run_policy(repository),
        Task::Architecture => run_architecture(repository),
        Task::WorkflowPolicy => run_workflow_policy(repository),
        Task::CommitMessage { message_file } => run_commit_message(repository, message_file),
        Task::PullRequestNaming { title, branch } => report_pull_request(
            "pull request naming",
            check_pull_request_naming(title, branch),
        ),
        Task::PullRequestDescription { body } => report_pull_request(
            "pull request description",
            check_pull_request_description(body),
        ),
    }
}

fn run_commit_message(
    repository: &Repository,
    message_file: &std::path::Path,
) -> Result<(), GateError> {
    report_check(
        "commit message",
        &check_commit_message_file(repository, message_file)?,
    )
}

fn report_pull_request(
    area: &'static str,
    findings: Result<Vec<Finding>, GateError>,
) -> Result<(), GateError> {
    report_check(area, &findings?)
}
