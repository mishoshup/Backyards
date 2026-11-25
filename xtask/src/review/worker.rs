use std::thread::ScopedJoinHandle;

use crate::report::GateError;
use crate::review::ReviewError;
use crate::review::post::ValidationOutcome;

pub fn join_worker(
    worker: ScopedJoinHandle<'_, Result<ValidationOutcome, GateError>>,
) -> Result<ValidationOutcome, GateError> {
    worker.join().unwrap_or_else(|_panic| {
        Err(ReviewError::InvalidAnswer {
            reason: "reviewer thread panicked".to_owned(),
        }
        .into())
    })
}
