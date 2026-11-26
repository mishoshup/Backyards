use std::path::Path;

use regex::Regex;
use serde::Deserialize;

use crate::review::tools::error::ToolError;
use crate::review::tools::limits::{MAXIMUM_LISTED_FILES, render_capped_lines};
use crate::workspace::capture_command_limited;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListFilesArguments {
    pub glob: String,
}

pub fn list_files(root: &Path, arguments: &ListFilesArguments) -> Result<String, ToolError> {
    let pattern = compile_glob(&arguments.glob)?;
    let matching: Vec<String> = list_repository_paths(root)?
        .into_iter()
        .filter(|path| pattern.is_match(path))
        .collect();
    Ok(render_capped_lines(&matching, MAXIMUM_LISTED_FILES))
}

const MAXIMUM_LISTING_BYTES: usize = 16_000_000;

pub fn list_repository_paths(root: &Path) -> Result<Vec<String>, ToolError> {
    let listing = capture_command_limited(
        root,
        "git",
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        MAXIMUM_LISTING_BYTES,
    )
    .map_err(|error| ToolError::ListingFailed {
        reason: error.to_string(),
    })?;
    reject_oversized_listing(&listing)?;
    Ok(listing
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect())
}

fn reject_oversized_listing(listing: &str) -> Result<(), ToolError> {
    if listing.len() <= MAXIMUM_LISTING_BYTES {
        return Ok(());
    }
    Err(ToolError::ListingFailed {
        reason: format!("file listing is over the {MAXIMUM_LISTING_BYTES} byte limit"),
    })
}

fn compile_glob(glob: &str) -> Result<Regex, ToolError> {
    let translated = translate_glob(glob);
    Regex::new(&format!("^{translated}$")).map_err(|source| ToolError::InvalidPattern {
        reason: source.to_string(),
    })
}

fn translate_double_star(
    characters: &mut std::iter::Peekable<std::str::Chars<'_>>,
    translated: &mut String,
) {
    if characters.next_if_eq(&'/').is_some() {
        translated.push_str("(?:.*/)?");
    } else {
        translated.push_str(".*");
    }
}

fn translate_glob(glob: &str) -> String {
    let mut translated = String::new();
    let mut characters = glob.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '*' if characters.next_if_eq(&'*').is_some() => {
                translate_double_star(&mut characters, &mut translated);
            }
            '*' => translated.push_str("[^/]*"),
            '?' => translated.push_str("[^/]"),
            other => translated.push_str(&regex::escape(&other.to_string())),
        }
    }
    translated
}
