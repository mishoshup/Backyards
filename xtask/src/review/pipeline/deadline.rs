use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::review::github::ProgressComment;
use crate::review::pipeline::finish::finish_failure;
use crate::review::pipeline::options::ReviewRun;
use crate::workspace::terminate_active_children;

const REVIEW_DEADLINE: Duration = Duration::from_mins(18);
const EXIT_CODE_FAILURE_RECORDED: i32 = 1;
const EXIT_CODE_FAILURE_NOT_RECORDED: i32 = 2;

fn describe_deadline_reason() -> String {
    format!(
        "The review exceeded its {}-minute budget before the job time limit.",
        REVIEW_DEADLINE.as_secs() / 60
    )
}

pub fn with_deadline<Outcome>(
    run: ReviewRun<'_>,
    comment: &ProgressComment,
    work: impl FnOnce() -> Outcome,
) -> Outcome {
    let is_settled = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let watchdog = scope.spawn(|| fail_when_overdue(run, comment, &is_settled));
        let _settle_on_exit = SettleOnExit {
            is_settled: &is_settled,
            watchdog: watchdog.thread().clone(),
        };
        let outcome = work();
        wait_if_watchdog_won(&is_settled);
        outcome
    })
}

struct SettleOnExit<'flag> {
    is_settled: &'flag AtomicBool,
    watchdog: std::thread::Thread,
}

impl Drop for SettleOnExit<'_> {
    fn drop(&mut self) {
        self.is_settled.store(true, Ordering::SeqCst);
        self.watchdog.unpark();
    }
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
    terminate_active_children();
    let exit_code = finish_failure(run, comment, &describe_deadline_reason())
        .map_or(EXIT_CODE_FAILURE_NOT_RECORDED, |()| {
            EXIT_CODE_FAILURE_RECORDED
        });
    std::process::exit(exit_code)
}
