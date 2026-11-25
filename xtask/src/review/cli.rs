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
        #[arg(long, default_value = "ai-review-completed/deepseek")]
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
            status_prefix,
        } => run_ci_review(
            repository,
            &CiOptions {
                pull_request: *pull_request,
                repository: github_repository.clone(),
                event_name: event_name.clone(),
                comment_body: comment_body.clone(),
                status_prefix: status_prefix.clone(),
            },
        ),
    }
}
