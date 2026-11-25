use std::path::Path;

use serde::Deserialize;

use crate::review::tools::confinement::resolve_inside_repository;
use crate::review::tools::error::ToolError;
use crate::review::tools::limits::truncate_output;

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
    let text = std::fs::read_to_string(&resolved).map_err(|source| ToolError::ReadFailed {
        reason: source.to_string(),
    })?;
    Ok(truncate_output(&number_lines(&text, arguments)))
}

fn number_lines(text: &str, arguments: &ReadFileArguments) -> String {
    let first = arguments.start_line.unwrap_or(FIRST_LINE);
    let last = arguments.end_line.unwrap_or(usize::MAX);
    let mut numbered = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if (first..=last).contains(&number) {
            numbered.push(format!("{number}\t{line}"));
        }
    }
    numbered.join("\n")
}
