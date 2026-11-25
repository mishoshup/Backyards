use std::path::Path;

use regex::Regex;
use serde::Deserialize;

use crate::review::tools::error::ToolError;
use crate::review::tools::limits::MAXIMUM_LISTED_FILES;
use crate::workspace::capture_command;

const DOUBLE_STAR_MARKER: &str = "\u{0}";

#[derive(Debug, Deserialize)]
pub struct ListFilesArguments {
    pub glob: String,
}

pub fn list_files(root: &Path, arguments: &ListFilesArguments) -> Result<String, ToolError> {
    let pattern = compile_glob(&arguments.glob)?;
    let matching: Vec<String> = list_repository_paths(root)?
        .into_iter()
        .filter(|path| pattern.is_match(path))
        .collect();
    Ok(render_listing(&matching))
}

pub fn list_repository_paths(root: &Path) -> Result<Vec<String>, ToolError> {
    let listing = capture_command(
        root,
        "git",
        &["ls-files", "--cached", "--others", "--exclude-standard"],
        &[0],
    )
    .map_err(|error| ToolError::ListingFailed {
        reason: error.to_string(),
    })?;
    Ok(listing.lines().map(str::to_owned).collect())
}

fn compile_glob(glob: &str) -> Result<Regex, ToolError> {
    let escaped = regex::escape(glob);
    let translated = escaped
        .replace(r"\*\*", DOUBLE_STAR_MARKER)
        .replace(r"\*", "[^/]*")
        .replace(r"\?", "[^/]")
        .replace(DOUBLE_STAR_MARKER, ".*");
    Regex::new(&format!("^{translated}$")).map_err(|source| ToolError::InvalidPattern {
        reason: source.to_string(),
    })
}

fn render_listing(paths: &[String]) -> String {
    let shown = paths
        .iter()
        .take(MAXIMUM_LISTED_FILES)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    if paths.len() > MAXIMUM_LISTED_FILES {
        format!("{shown}\n[output truncated after {MAXIMUM_LISTED_FILES} lines]")
    } else {
        shown
    }
}
