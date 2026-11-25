use crate::error::GateError;
use crate::finding::Finding;
use crate::repository::compile_pattern;

const NAMING_AREA: &str = "pull request naming";
const DESCRIPTION_AREA: &str = "pull request description";
const CHANGE_TYPES: &str = "feat|fix|refactor|perf|test|docs|chore|ci|build";
const MAXIMUM_TITLE_LENGTH: usize = 72;
const REQUIRED_SECTIONS: [&str; 3] = ["### Problem", "### Change", "### Verification"];
const UNCHECKED_PATTERN: &str = r"(?m)^- \[ \]";

pub fn check_pull_request_naming(title: &str, branch: &str) -> Result<Vec<Finding>, GateError> {
    let mut findings = Vec::new();
    findings.extend(check_title_format(title)?);
    findings.extend(check_title_length(title));
    findings.extend(check_branch_format(branch)?);
    findings.extend(check_branch_type_matches_title(title, branch));
    Ok(findings)
}

pub fn check_pull_request_description(body: &str) -> Result<Vec<Finding>, GateError> {
    let mut findings = check_required_sections(body);
    findings.extend(check_checklist_complete(body)?);
    Ok(findings)
}

fn naming_error(message: String) -> Vec<Finding> {
    vec![Finding::error(NAMING_AREA, message)]
}

fn check_title_format(title: &str) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(&format!(
        r"^({CHANGE_TYPES})\([a-z0-9-]+\): [a-z0-9].*[^.]$"
    ))?;
    if pattern.is_match(title) {
        return Ok(Vec::new());
    }
    Ok(naming_error(format!(
        "title \"{title}\" must match <type>(<scope>): <subject> without a trailing period"
    )))
}

fn check_title_length(title: &str) -> Vec<Finding> {
    let length = title.chars().count();
    if length <= MAXIMUM_TITLE_LENGTH {
        return Vec::new();
    }
    naming_error(format!(
        "title has {length} characters, the limit is {MAXIMUM_TITLE_LENGTH}"
    ))
}

fn check_branch_format(branch: &str) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(&format!(r"^({CHANGE_TYPES})/[a-z0-9]+(-[a-z0-9]+)*$"))?;
    if pattern.is_match(branch) {
        return Ok(Vec::new());
    }
    Ok(naming_error(format!(
        "branch \"{branch}\" must match <type>/<short-description> in lowercase hyphens"
    )))
}

fn check_branch_type_matches_title(title: &str, branch: &str) -> Vec<Finding> {
    let title_type = title.split('(').next().unwrap_or_default();
    let branch_type = branch.split('/').next().unwrap_or_default();
    if title_type == branch_type {
        return Vec::new();
    }
    naming_error(format!(
        "title type \"{title_type}\" differs from branch type \"{branch_type}\""
    ))
}

fn check_required_sections(body: &str) -> Vec<Finding> {
    REQUIRED_SECTIONS
        .iter()
        .filter(|section| !body.contains(*section))
        .map(|section| Finding::error(DESCRIPTION_AREA, format!("missing section \"{section}\"")))
        .collect()
}

fn check_checklist_complete(body: &str) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(UNCHECKED_PATTERN)?;
    let unchecked = pattern.find_iter(body).count();
    if unchecked == 0 {
        return Ok(Vec::new());
    }
    Ok(vec![Finding::error(
        DESCRIPTION_AREA,
        format!("{unchecked} checklist box(es) unchecked, check each or write n/a after it"),
    )])
}
