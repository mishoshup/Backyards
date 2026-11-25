mod added_lines;
mod size_limit;
mod source;

pub use added_lines::{AddedLines, parse_added_lines};
pub use size_limit::{MAXIMUM_DIFF_BYTES, reject_oversized_diff};
pub use source::read_local_diff;
