use std::path::{Path, PathBuf};

use regex::Regex;

use crate::report::Finding;
use crate::report::GateError;
use crate::workspace::Repository;

#[derive(Debug)]
pub struct ScanRule<'rule> {
    pub area: &'static str,
    pub heading: &'rule str,
    pub pattern: &'rule Regex,
}

impl<'rule> ScanRule<'rule> {
    #[must_use]
    pub const fn new(area: &'static str, heading: &'rule str, pattern: &'rule Regex) -> Self {
        Self {
            area,
            heading,
            pattern,
        }
    }
}

pub fn report_matching_lines(
    repository: &Repository,
    rule: &ScanRule,
    files: &[PathBuf],
) -> Result<Vec<Finding>, GateError> {
    let mut findings = Vec::new();
    for file in files {
        let text = repository.read_text(file)?;
        findings.extend(describe_matches(rule, file, &text));
    }
    Ok(findings)
}

#[must_use]
pub fn describe_matches(rule: &ScanRule, file: &Path, text: &str) -> Vec<Finding> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| rule.pattern.is_match(line))
        .map(|(index, line)| describe_line(rule, file, index + 1, line))
        .collect()
}

fn describe_line(rule: &ScanRule, file: &Path, number: usize, line: &str) -> Finding {
    Finding::error(
        rule.area,
        format!(
            "{}: {}:{number}: {}",
            rule.heading,
            file.display(),
            line.trim()
        ),
    )
}
