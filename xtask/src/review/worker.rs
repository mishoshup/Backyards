use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
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

pub fn run_bounded_shards<Work>(
    shards: &[String],
    parallelism: usize,
    work: Work,
) -> Vec<Result<ValidationOutcome, GateError>>
where
    Work: Fn(&str) -> Result<ValidationOutcome, GateError> + Sync,
{
    let next_shard = AtomicUsize::new(0);
    let finished: Mutex<Vec<(usize, Result<ValidationOutcome, GateError>)>> =
        Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..parallelism.min(shards.len()) {
            scope.spawn(|| drain_shard_queue(shards, &next_shard, &finished, &work));
        }
    });
    let mut results = finished
        .into_inner()
        .unwrap_or_else(PoisonError::into_inner);
    results.sort_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

fn drain_shard_queue<Work>(
    shards: &[String],
    next_shard: &AtomicUsize,
    finished: &Mutex<Vec<(usize, Result<ValidationOutcome, GateError>)>>,
    work: &Work,
) where
    Work: Fn(&str) -> Result<ValidationOutcome, GateError> + Sync,
{
    loop {
        let index = next_shard.fetch_add(1, Ordering::SeqCst);
        let Some(shard) = shards.get(index) else {
            return;
        };
        let result = work(shard);
        finished
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((index, result));
    }
}
