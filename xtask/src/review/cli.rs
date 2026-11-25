use clap::Subcommand;

use crate::report::GateError;
use crate::review::local::{LocalOptions, run_local_review};
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
    }
}
