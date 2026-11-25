use std::path::Path;

use regex::Regex;
use serde::Deserialize;

use crate::review::tools::confinement::{resolve_inside_canonical_root, resolve_inside_repository};
use crate::review::tools::error::ToolError;
use crate::review::tools::limits::{
    MAXIMUM_GREP_MATCHES, MAXIMUM_SEARCHED_FILE_BYTES, render_capped_lines, truncate_output,
};
use crate::review::tools::list_files::list_repository_paths;

const DEFAULT_SEARCH_PATH: &str = ".";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrepArguments {
    pub pattern: String,
    pub path: Option<String>,
}

#[derive(Default)]
struct SearchResult {
    matches: Vec<String>,
    skipped: Vec<String>,
}

pub fn grep(root: &Path, arguments: &GrepArguments) -> Result<String, ToolError> {
    let pattern = Regex::new(&arguments.pattern).map_err(|source| ToolError::InvalidPattern {
        reason: source.to_string(),
    })?;
    let scope = arguments.path.as_deref().unwrap_or(DEFAULT_SEARCH_PATH);
    let scope_root = resolve_inside_repository(root, scope)?;
    let canonical_root = resolve_inside_repository(root, DEFAULT_SEARCH_PATH)?;
    let result = search_files(&canonical_root, &scope_root, &pattern)?;
    Ok(truncate_output(&render_result(&result)))
}

fn search_files(
    canonical_root: &Path,
    scope_root: &Path,
    pattern: &Regex,
) -> Result<SearchResult, ToolError> {
    let mut result = SearchResult::default();
    for relative in list_repository_paths(canonical_root)? {
        let path = canonical_root.join(&relative);
        if path.starts_with(scope_root) {
            collect_confined_matches(canonical_root, &relative, pattern, &mut result);
        }
        if result.matches.len() > MAXIMUM_GREP_MATCHES {
            break;
        }
    }
    Ok(result)
}

fn collect_confined_matches(
    canonical_root: &Path,
    relative: &str,
    pattern: &Regex,
    result: &mut SearchResult,
) {
    match resolve_inside_canonical_root(canonical_root, relative) {
        Ok(resolved) => collect_file_matches(relative, &resolved, pattern, result),
        Err(error) => result.skipped.push(format!("{relative}: {error}")),
    }
}

fn collect_file_matches(relative: &str, path: &Path, pattern: &Regex, result: &mut SearchResult) {
    match read_searchable_text(path) {
        Ok(text) => result
            .matches
            .extend(find_matching_lines(relative, &text, pattern)),
        Err(reason) => result.skipped.push(format!("{relative}: {reason}")),
    }
}

fn find_matching_lines(relative: &str, text: &str, pattern: &Regex) -> Vec<String> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if pattern.is_match(line) {
            found.push(format!("{relative}:{}:{line}", index + 1));
        }
        if found.len() > MAXIMUM_GREP_MATCHES {
            break;
        }
    }
    found
}

fn read_searchable_text(path: &Path) -> Result<String, ToolError> {
    let metadata = std::fs::metadata(path).map_err(|source| read_failure(&source))?;
    if !metadata.is_file() || metadata.len() > MAXIMUM_SEARCHED_FILE_BYTES {
        return Err(ToolError::NotSearchable {
            limit: MAXIMUM_SEARCHED_FILE_BYTES,
        });
    }
    std::fs::read_to_string(path).map_err(|source| read_failure(&source))
}

fn read_failure(source: &std::io::Error) -> ToolError {
    ToolError::ReadFailed {
        reason: source.to_string(),
    }
}

fn render_result(result: &SearchResult) -> String {
    let matches = render_capped_lines(&result.matches, MAXIMUM_GREP_MATCHES);
    if result.skipped.is_empty() {
        return matches;
    }
    format!("{matches}\n[skipped files]\n{}", result.skipped.join("\n"))
}
