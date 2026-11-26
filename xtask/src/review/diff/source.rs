use crate::report::{GateError, write_report_line};
use crate::review::diff::size_limit::{MAXIMUM_DIFF_BYTES, reject_oversized_diff};
use crate::review::error::ReviewError;
use crate::workspace::{Repository, capture_command, capture_command_limited};

const SUCCESS_EXIT: i32 = 0;
const DIFFERENCES_FOUND_EXIT: i32 = 1;
const MAXIMUM_UNTRACKED_LISTING_BYTES: usize = 4_000_000;

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

fn reject_oversized_listing(listing: &str) -> Result<(), GateError> {
    if listing.len() > MAXIMUM_UNTRACKED_LISTING_BYTES {
        return Err(ReviewError::DiffTooLarge {
            size: listing.len(),
            limit: MAXIMUM_UNTRACKED_LISTING_BYTES,
        }
        .into());
    }
    Ok(())
}

fn diff_untracked_files(
    repository: &Repository,
    tracked_bytes: usize,
) -> Result<String, GateError> {
    let listing = capture_command_limited(
        repository.root(),
        "git",
        &["ls-files", "--others", "--exclude-standard", "-z"],
        MAXIMUM_UNTRACKED_LISTING_BYTES,
    )?;
    reject_oversized_listing(&listing)?;
    let mut combined = String::new();
    for path in listing.split('\0').filter(|path| !path.is_empty()) {
        reject_oversized_text_file(repository, path, tracked_bytes + combined.len())?;
        combined.push_str(&diff_new_file(repository, path)?);
    }
    Ok(combined)
}

fn reject_oversized_text_file(
    repository: &Repository,
    path: &str,
    already_captured: usize,
) -> Result<(), GateError> {
    let rejection = reject_file_over_remaining_budget(repository, path, already_captured);
    match rejection {
        Err(GateError::Review(ReviewError::DiffTooLarge { .. }))
            if is_binary_file(repository, path)? =>
        {
            write_report_line(&format!("skipped oversized binary file {path}"))
        }
        other => other,
    }
}

fn is_binary_file(repository: &Repository, path: &str) -> Result<bool, GateError> {
    let statistics = capture_command(
        repository.root(),
        "git",
        &["diff", "--no-index", "--numstat", "--", "/dev/null", path],
        &[SUCCESS_EXIT, DIFFERENCES_FOUND_EXIT],
    )?;
    Ok(statistics.starts_with("-\t-\t"))
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
    let file_bytes = std::fs::symlink_metadata(repository.root().join(path))
        .map_err(|source| GateError::RunCommand {
            command: format!("read metadata of {path}"),
            source,
        })?
        .len();
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
