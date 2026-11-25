use std::time::Duration;

pub const MAXIMUM_TOOL_OUTPUT_BYTES: usize = 30_000;
pub const MAXIMUM_GREP_MATCHES: usize = 100;
pub const MAXIMUM_LISTED_FILES: usize = 200;
pub const MAXIMUM_SEARCHED_FILE_BYTES: u64 = 1_000_000;
pub const MAXIMUM_FETCH_BYTES: u64 = 2_000_000;
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(20);

use crate::review::text::prefix_within_bytes;

pub fn truncate_output(text: &str) -> String {
    if text.len() <= MAXIMUM_TOOL_OUTPUT_BYTES {
        return text.to_owned();
    }
    format!(
        "{}\n[output truncated after {MAXIMUM_TOOL_OUTPUT_BYTES} bytes]",
        prefix_within_bytes(text, MAXIMUM_TOOL_OUTPUT_BYTES)
    )
}

pub fn render_capped_lines(lines: &[String], maximum: usize) -> String {
    let shown = lines
        .iter()
        .take(maximum)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    if lines.len() > maximum {
        format!("{shown}\n[output truncated after {maximum} lines]")
    } else {
        shown
    }
}
