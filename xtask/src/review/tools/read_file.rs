use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;

use crate::review::tools::confinement::resolve_inside_repository;
use crate::review::tools::error::ToolError;
use crate::review::tools::limits::{MAXIMUM_TOOL_OUTPUT_BYTES, truncate_output};

const FIRST_LINE: usize = 1;

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
    let file = File::open(&resolved).map_err(|source| read_failure(&source))?;
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
    for (index, line) in reader.lines().enumerate() {
        let number = index + 1;
        if number > last || numbered_size(&numbered) > MAXIMUM_TOOL_OUTPUT_BYTES {
            break;
        }
        if number >= first {
            numbered.push(format!(
                "{number}\t{}",
                line.map_err(|source| read_failure(&source))?
            ));
        }
    }
    Ok(numbered.join("\n"))
}

fn numbered_size(numbered: &[String]) -> usize {
    numbered.iter().map(String::len).sum()
}

fn read_failure(source: &std::io::Error) -> ToolError {
    ToolError::ReadFailed {
        reason: source.to_string(),
    }
}
