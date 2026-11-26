use std::time::Duration;

use crate::review::error::ReviewError;
use crate::workspace::pause_for;

const MAXIMUM_ATTEMPTS: u32 = 2;

pub fn run_with_retry<T>(
    delay: Duration,
    mut attempt: impl FnMut() -> Result<T, ReviewError>,
) -> Result<T, ReviewError> {
    let mut attempts_made = 1;
    loop {
        match attempt() {
            Err(error) if error.is_retryable() && attempts_made < MAXIMUM_ATTEMPTS => {
                attempts_made += 1;
                pause_for(delay);
            }
            outcome => return outcome,
        }
    }
}
