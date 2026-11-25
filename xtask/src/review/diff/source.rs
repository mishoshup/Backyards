use crate::report::GateError;
use crate::review::diff::size_limit::reject_oversized_diff;
use crate::workspace::{Repository, capture_command};

const SUCCESS_EXIT: i32 = 0;
const DIFFERENCES_FOUND_EXIT: i32 = 1;

pub fn read_local_diff(repository: &Repository, base_reference: &str) -> Result<String, GateError> {
    let merge_base = find_merge_base(repository, base_reference)?;
    let mut diff = capture_command(
        repository.root(),
        "git",
        &["diff", merge_base.trim()],
        &[SUCCESS_EXIT],
    )?;
    diff.push_str(&diff_untracked_files(repository)?);
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

fn diff_untracked_files(repository: &Repository) -> Result<String, GateError> {
    let listing = capture_command(
        repository.root(),
        "git",
        &["ls-files", "--others", "--exclude-standard"],
        &[SUCCESS_EXIT],
    )?;
    listing
        .lines()
        .map(|path| diff_new_file(repository, path))
        .collect()
}

fn diff_new_file(repository: &Repository, path: &str) -> Result<String, GateError> {
    capture_command(
        repository.root(),
        "git",
        &["diff", "--no-index", "--", "/dev/null", path],
        &[SUCCESS_EXIT, DIFFERENCES_FOUND_EXIT],
    )
}
