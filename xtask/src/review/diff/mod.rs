mod added_lines;
mod size_limit;

pub use added_lines::{AddedLines, parse_added_lines};
pub use size_limit::{MAXIMUM_DIFF_BYTES, reject_oversized_diff};
