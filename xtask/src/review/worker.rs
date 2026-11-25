use std::thread::ScopedJoinHandle;

use crate::report::GateError;
use crate::review::post::ValidationOutcome;
use crate::review::{ALL_REVIEWER_MODES, ReviewError, ReviewerMode};

pub type ReviewerResult = (ReviewerMode, Result<ValidationOutcome, GateError>);

pub fn run_each_reviewer_mode<Work>(work: Work) -> Vec<ReviewerResult>
where
    Work: Fn(ReviewerMode) -> Result<ValidationOutcome, GateError> + Sync,
{
    std::thread::scope(|scope| {
        let workers: Vec<_> = ALL_REVIEWER_MODES
            .iter()
            .map(|mode| scope.spawn(|| work(*mode)))
            .collect();
        ALL_REVIEWER_MODES
            .iter()
            .zip(workers)
            .map(|(mode, worker)| (*mode, join_worker(worker)))
            .collect()
    })
}

fn join_worker(
    worker: ScopedJoinHandle<'_, Result<ValidationOutcome, GateError>>,
) -> Result<ValidationOutcome, GateError> {
    worker.join().unwrap_or_else(|payload| {
        Err(ReviewError::InvalidAnswer {
            reason: format!(
                "reviewer thread panicked: {}",
                describe_panic(payload.as_ref())
            ),
        }
        .into())
    })
}

fn describe_panic(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "no message".to_owned())
}
