use clap::{Args, Subcommand};

use crate::report::GateError;
use crate::review::github::EventName;
use crate::review::local::{LocalOptions, run_local_review};
use crate::review::pipeline::{CiOptions, StatusPrefix, run_ci_review};
use crate::workspace::Repository;

#[derive(Debug, Subcommand)]
pub enum ReviewTask {
    /// Review the working tree against a base reference with a local model
    Local(LocalArguments),
    /// Run both reviewers against a pull request and post the results
    PullRequest(PullRequestArguments),
}

#[derive(Debug, Args)]
pub struct LocalArguments {
    #[arg(long, env = "BASE_REFERENCE", default_value = "origin/main")]
    base: String,
    #[arg(
        long,
        env = "LOCAL_REVIEW_MODEL",
        default_value = "opencode/big-pickle"
    )]
    model: String,
}

#[derive(Debug, Args)]
pub struct PullRequestArguments {
    #[arg(long)]
    pull_request: u64,
    #[arg(long, env = "GITHUB_REPOSITORY", value_parser = parse_repository)]
    repository: String,
    #[arg(long, env = "GITHUB_EVENT_NAME", value_enum)]
    event_name: EventName,
    #[arg(long, env = "REVIEW_COMMENT_BODY", default_value = "")]
    comment_body: String,
    #[arg(long, env = "REVIEW_COMMENT_AUTHOR_ASSOCIATION", default_value = "")]
    comment_author_association: String,
    #[arg(long, default_value = "ai-review-completed/deepseek")]
    status_prefix: StatusPrefix,
}

impl From<&LocalArguments> for LocalOptions {
    fn from(arguments: &LocalArguments) -> Self {
        Self {
            base_reference: arguments.base.clone(),
            model: arguments.model.clone(),
        }
    }
}

impl From<&PullRequestArguments> for CiOptions {
    fn from(arguments: &PullRequestArguments) -> Self {
        Self {
            pull_request: arguments.pull_request,
            repository: arguments.repository.clone(),
            event_name: arguments.event_name,
            comment_body: arguments.comment_body.clone(),
            comment_author_association: arguments.comment_author_association.clone(),
            status_prefix: arguments.status_prefix.clone(),
        }
    }
}

pub fn run_review_task(task: &ReviewTask, repository: &Repository) -> Result<(), GateError> {
    match task {
        ReviewTask::Local(arguments) => run_local_review(repository, &arguments.into()),
        ReviewTask::PullRequest(arguments) => run_ci_review(repository, &arguments.into()),
    }
}

fn parse_repository(value: &str) -> Result<String, String> {
    match value.split_once('/') {
        Some((owner, name)) if is_repository_part(owner) && is_repository_part(name) => {
            Ok(value.to_owned())
        }
        _ => Err("repository must be written as owner/name".to_owned()),
    }
}

fn is_repository_part(part: &str) -> bool {
    !part.is_empty()
        && part.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}
