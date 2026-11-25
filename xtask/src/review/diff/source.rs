use crate::report::GateError;
use crate::review::diff::size_limit::{MAXIMUM_DIFF_BYTES, reject_oversized_diff};
use crate::review::error::ReviewError;
use crate::workspace::{Repository, capture_command, capture_command_limited};

const SUCCESS_EXIT: i32 = 0;
const DIFFERENCES_FOUND_EXIT: i32 = 1;

pub fn read_local_diff(repository: &Repository, base_reference: &str) -> Result<String, GateError> {
    let merge_base = find_merge_base(repository, base_reference)?;
    let mut diff = capture_command_limited(
        repository.root(),
        "git",
        &["diff", merge_base.trim()],
        MAXIMUM_DIFF_BYTES,
    )?;
    reject_oversized_diff(&diff)?;
    diff.push_str(&diff_untracked_files(repository, diff.len())?);
    reject_oversized_diff(&diff)?;
    Ok(diff)
}

fn find_merge_base(repository: &Repository, base_reference: &str) -> Result<String, GateError> {
    capture_command(
        repository.root(),
        "git",
        &["merge-base", base_reference, "HEAD"],
        &[SUCCESS_EXIT],
    )
}

fn diff_untracked_files(
    repository: &Repository,
    tracked_bytes: usize,
) -> Result<String, GateError> {
    let listing = capture_command(
        repository.root(),
        "git",
        &["ls-files", "--others", "--exclude-standard"],
        &[SUCCESS_EXIT],
    )?;
    let mut combined = String::new();
    for path in listing.lines() {
        reject_file_over_remaining_budget(repository, path, tracked_bytes + combined.len())?;
        combined.push_str(&diff_new_file(repository, path)?);
    }
    Ok(combined)
}

fn diff_new_file(repository: &Repository, path: &str) -> Result<String, GateError> {
    capture_command(
        repository.root(),
        "git",
        &["diff", "--no-index", "--", "/dev/null", path],
        &[SUCCESS_EXIT, DIFFERENCES_FOUND_EXIT],
    )
}

fn reject_file_over_remaining_budget(
    repository: &Repository,
    path: &str,
    already_captured: usize,
) -> Result<(), GateError> {
    let file_bytes =
        std::fs::metadata(repository.root().join(path)).map_or(0, |metadata| metadata.len());
    let projected = usize::try_from(file_bytes)
        .map_or(usize::MAX, |bytes| bytes.saturating_add(already_captured));
    if projected > MAXIMUM_DIFF_BYTES {
        return Err(ReviewError::DiffTooLarge {
            size: projected,
            limit: MAXIMUM_DIFF_BYTES,
        }
        .into());
    }
    Ok(())
}
