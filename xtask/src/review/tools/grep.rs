use std::path::Path;

use regex::Regex;
use serde::Deserialize;

use crate::review::tools::confinement::resolve_inside_repository;
use crate::review::tools::error::ToolError;
use crate::review::tools::limits::{
    MAXIMUM_GREP_MATCHES, MAXIMUM_SEARCHED_FILE_BYTES, truncate_output,
};
use crate::review::tools::list_files::list_repository_paths;

const DEFAULT_SEARCH_PATH: &str = ".";

#[derive(Debug, Deserialize)]
pub struct GrepArguments {
    pub pattern: String,
    pub path: Option<String>,
}

pub fn grep(root: &Path, arguments: &GrepArguments) -> Result<String, ToolError> {
    let pattern = Regex::new(&arguments.pattern).map_err(|source| ToolError::InvalidPattern {
        reason: source.to_string(),
    })?;
    let scope = arguments.path.as_deref().unwrap_or(DEFAULT_SEARCH_PATH);
    let scope_root = resolve_inside_repository(root, scope)?;
    let canonical_root = resolve_inside_repository(root, DEFAULT_SEARCH_PATH)?;
    let matches = search_files(&canonical_root, &scope_root, &pattern)?;
    Ok(truncate_output(&render_matches(&matches)))
}

fn search_files(
    canonical_root: &Path,
    scope_root: &Path,
    pattern: &Regex,
) -> Result<Vec<String>, ToolError> {
    let mut matches = Vec::new();
    for relative in list_repository_paths(canonical_root)? {
        let path = canonical_root.join(&relative);
        if path.starts_with(scope_root) {
            collect_file_matches(&relative, &path, pattern, &mut matches);
        }
        if matches.len() > MAXIMUM_GREP_MATCHES {
            break;
        }
    }
    Ok(matches)
}

fn collect_file_matches(relative: &str, path: &Path, pattern: &Regex, matches: &mut Vec<String>) {
    let Some(text) = read_searchable_text(path) else {
        return;
    };
    matches.extend(
        text.lines()
            .enumerate()
            .filter(|(_, line)| pattern.is_match(line))
            .map(|(index, line)| format!("{relative}:{}:{line}", index + 1)),
    );
}

fn read_searchable_text(path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > MAXIMUM_SEARCHED_FILE_BYTES {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

fn render_matches(matches: &[String]) -> String {
    let shown = matches
        .iter()
        .take(MAXIMUM_GREP_MATCHES)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    if matches.len() > MAXIMUM_GREP_MATCHES {
        format!("{shown}\n[output truncated after {MAXIMUM_GREP_MATCHES} lines]")
    } else {
        shown
    }
}
