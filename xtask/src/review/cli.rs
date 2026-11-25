use clap::Subcommand;

use crate::report::GateError;
use crate::review::local::{LocalOptions, run_local_review};
use crate::review::pipeline::{CiOptions, run_ci_review};
use crate::workspace::Repository;

#[derive(Debug, Subcommand)]
pub enum ReviewTask {
    /// Review the working tree against a base reference with a local model
    Local {
        #[arg(long, env = "BASE_REFERENCE", default_value = "origin/main")]
        base: String,
        #[arg(
            long,
            env = "LOCAL_REVIEW_MODEL",
            default_value = "opencode/big-pickle"
        )]
        model: String,
    },
    /// Run both reviewers against a pull request and post the results
    PullRequest {
        #[arg(long)]
        pull_request: u64,
        #[arg(long, env = "GITHUB_REPOSITORY")]
        repository: String,
        #[arg(long, env = "GITHUB_EVENT_NAME")]
        event_name: String,
        #[arg(long, env = "REVIEW_COMMENT_BODY", default_value = "")]
        comment_body: String,
        #[arg(long, env = "REVIEW_COMMENT_AUTHOR_ASSOCIATION", default_value = "")]
        comment_author_association: String,
        #[arg(
            long,
            default_value = "ai-review-completed/deepseek",
            value_parser = parse_status_prefix
        )]
        status_prefix: String,
    },
}

pub fn run_review_task(task: &ReviewTask, repository: &Repository) -> Result<(), GateError> {
    match task {
        ReviewTask::Local { base, model } => run_local_review(
            repository,
            &LocalOptions {
                base_reference: base.clone(),
                model: model.clone(),
            },
        ),
        ReviewTask::PullRequest {
            pull_request,
            repository: github_repository,
            event_name,
            comment_body,
            comment_author_association,
            status_prefix,
        } => run_ci_review(
            repository,
            &CiOptions {
                pull_request: *pull_request,
                repository: github_repository.clone(),
                event_name: event_name.clone(),
                comment_body: comment_body.clone(),
                comment_author_association: comment_author_association.clone(),
                status_prefix: status_prefix.clone(),
            },
        ),
    }
}

fn parse_status_prefix(value: &str) -> Result<String, String> {
    if is_valid_status_prefix(value) {
        Ok(value.to_owned())
    } else {
        Err("status prefix must use lowercase letters, digits, '/', '_' and '-' only".to_owned())
    }
}

fn is_valid_status_prefix(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(is_allowed_status_character)
        && is_alphanumeric_edge(value.chars().next())
        && is_alphanumeric_edge(value.chars().last())
}

const fn is_allowed_status_character(character: char) -> bool {
    character.is_ascii_lowercase()
        || character.is_ascii_digit()
        || matches!(character, '/' | '_' | '-')
}

fn is_alphanumeric_edge(character: Option<char>) -> bool {
    character.is_some_and(|symbol| symbol.is_ascii_alphanumeric())
}
