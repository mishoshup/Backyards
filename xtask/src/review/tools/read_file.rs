use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;

use crate::review::tools::confinement::resolve_inside_repository;
use crate::review::tools::error::ToolError;
use crate::review::tools::limits::{MAXIMUM_TOOL_OUTPUT_BYTES, truncate_output};

const FIRST_LINE: usize = 1;
const MAXIMUM_READ_FILE_BYTES: u64 = 5_000_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadFileArguments {
    pub path: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

pub fn read_file(root: &Path, arguments: &ReadFileArguments) -> Result<String, ToolError> {
    let resolved = resolve_inside_repository(root, &arguments.path)?;
    if !resolved.is_file() {
        return Err(ToolError::NotAFile);
    }
    reject_oversized_file(&resolved)?;
    let file = File::open(&resolved).map_err(|source| ToolError::from_read_failure(&source))?;
    let numbered = number_requested_lines(BufReader::new(file), arguments)?;
    Ok(truncate_output(&numbered))
}

fn number_requested_lines(
    reader: impl BufRead,
    arguments: &ReadFileArguments,
) -> Result<String, ToolError> {
    let first = arguments.start_line.unwrap_or(FIRST_LINE);
    let last = arguments.end_line.unwrap_or(usize::MAX);
    let mut numbered = Vec::new();
    let mut numbered_bytes = 0_usize;
    for (index, line) in reader.lines().enumerate() {
        let number = index + 1;
        if number > last || numbered_bytes > MAXIMUM_TOOL_OUTPUT_BYTES {
            break;
        }
        if number >= first {
            let text = format!(
                "{number}\t{}",
                line.map_err(|source| ToolError::from_read_failure(&source))?
            );
            numbered_bytes += text.len();
            numbered.push(text);
        }
    }
    Ok(numbered.join("\n"))
}

fn reject_oversized_file(path: &Path) -> Result<(), ToolError> {
    let size = std::fs::metadata(path)
        .map_err(|source| ToolError::from_read_failure(&source))?
        .len();
    if size > MAXIMUM_READ_FILE_BYTES {
        return Err(ToolError::FileTooLarge {
            limit: MAXIMUM_READ_FILE_BYTES,
        });
    }
    Ok(())
}
