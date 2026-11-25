use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::review::github::ProgressComment;
use crate::review::pipeline::finish::finish_failure;
use crate::review::pipeline::options::ReviewRun;

const REVIEW_DEADLINE: Duration = Duration::from_mins(22);
const DEADLINE_REASON: &str = "The review exceeded its 22 minute budget before the job time limit.";

pub fn with_deadline<Outcome>(
    run: ReviewRun<'_>,
    comment: &ProgressComment,
    work: impl FnOnce() -> Outcome,
) -> Outcome {
    let is_settled = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let watchdog = scope.spawn(|| fail_when_overdue(run, comment, &is_settled));
        let outcome = work();
        wait_if_watchdog_won(&is_settled);
        watchdog.thread().unpark();
        outcome
    })
}

fn wait_if_watchdog_won(is_settled: &AtomicBool) {
    if is_settled.swap(true, Ordering::SeqCst) {
        loop {
            std::thread::park();
        }
    }
}

fn fail_when_overdue(run: ReviewRun<'_>, comment: &ProgressComment, is_settled: &AtomicBool) {
    let deadline = Instant::now() + REVIEW_DEADLINE;
    while !is_settled.load(Ordering::SeqCst) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() && !is_settled.swap(true, Ordering::SeqCst) {
            abandon_review(run, comment);
        }
        std::thread::park_timeout(remaining);
    }
}

fn abandon_review(run: ReviewRun<'_>, comment: &ProgressComment) -> ! {
    let exit_code = finish_failure(run, comment, DEADLINE_REASON).map_or(2, |()| 1);
    std::process::exit(exit_code)
}
