use crate::review::error::ReviewError;

pub const MAXIMUM_DIFF_BYTES: usize = 600_000;

pub const fn reject_oversized_diff(diff: &str) -> Result<(), ReviewError> {
    if diff.len() > MAXIMUM_DIFF_BYTES {
        Err(ReviewError::DiffTooLarge {
            size: diff.len(),
            limit: MAXIMUM_DIFF_BYTES,
        })
    } else {
        Ok(())
    }
}
