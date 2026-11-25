use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::report::GateError;
use crate::review::error::ReviewError;
use crate::workspace::Repository;

const RULES_DIRECTORY: &str = ".agents/skills/rust-coding-standards/rules";
const RULE_IDENTIFIER_KEY: &str = "ruleId:";
const HEADING_PREFIX: &str = "# ";
const IDENTIFIER_SEPARATOR: char = '-';
const INTERNAL_FILE_PREFIX: char = '_';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleFile {
    pub identifier: String,
    pub heading: String,
    pub file_name: String,
    pub content: String,
}

pub fn load_rule_files(repository: &Repository) -> Result<Vec<RuleFile>, GateError> {
    let mut paths = list_rule_paths(repository)?;
    paths.sort();
    if paths.is_empty() {
        return Err(ReviewError::NoRuleFiles {
            directory: PathBuf::from(RULES_DIRECTORY),
        }
        .into());
    }
    paths
        .iter()
        .map(|path| load_rule_file(repository, path))
        .collect()
}

#[must_use]
pub fn build_rule_checklist(rules: &[RuleFile]) -> String {
    rules
        .iter()
        .map(|rule| {
            format!(
                "- {}: {} ({})",
                rule.identifier, rule.heading, rule.file_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[must_use]
pub fn allowed_rule_identifiers(rules: &[RuleFile]) -> BTreeSet<&str> {
    rules.iter().map(|rule| rule.identifier.as_str()).collect()
}

fn list_rule_paths(repository: &Repository) -> Result<Vec<PathBuf>, GateError> {
    Ok(repository
        .list_files(&["md"])?
        .into_iter()
        .filter(|path| is_rule_file_path(path))
        .collect())
}

fn is_rule_file_path(path: &Path) -> bool {
    path.parent() == Some(Path::new(RULES_DIRECTORY))
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| !name.starts_with(INTERNAL_FILE_PREFIX))
}

fn load_rule_file(repository: &Repository, path: &Path) -> Result<RuleFile, GateError> {
    let content = repository.read_text(path)?;
    Ok(parse_rule_file(path, content)?)
}

fn parse_rule_file(path: &Path, content: String) -> Result<RuleFile, ReviewError> {
    let identifier = read_rule_identifier(&content)
        .ok_or_else(|| ReviewError::MissingRuleIdentifier {
            path: path.to_path_buf(),
        })?
        .to_owned();
    let heading = read_heading(&content)
        .ok_or_else(|| ReviewError::MissingRuleHeading {
            path: path.to_path_buf(),
        })?
        .to_owned();
    let file_name = path
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    Ok(RuleFile {
        identifier,
        heading,
        file_name,
        content,
    })
}

fn read_rule_identifier(content: &str) -> Option<&str> {
    let identifier = find_identifier_line(content)?;
    is_valid_rule_identifier(identifier).then_some(identifier)
}

fn find_identifier_line(content: &str) -> Option<&str> {
    for line in content.lines() {
        if let Some(value) = line.strip_prefix(RULE_IDENTIFIER_KEY) {
            return Some(value.trim());
        }
    }
    None
}

fn read_heading(content: &str) -> Option<&str> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(HEADING_PREFIX))
        .map(str::trim)
}

fn is_valid_rule_identifier(identifier: &str) -> bool {
    let parts: Vec<&str> = identifier.split(IDENTIFIER_SEPARATOR).collect();
    let Some((number, words)) = parts.split_last() else {
        return false;
    };
    !words.is_empty()
        && is_made_of(number, |character| character.is_ascii_digit())
        && words
            .iter()
            .all(|word| is_made_of(word, |character| character.is_ascii_uppercase()))
}

fn is_made_of(text: &str, is_allowed: impl Fn(char) -> bool) -> bool {
    !text.is_empty() && text.chars().all(is_allowed)
}
