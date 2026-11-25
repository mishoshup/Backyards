use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error("diff is {size} bytes, over the {limit} byte limit; split the pull request")]
    DiffTooLarge { size: usize, limit: usize },
    #[error("malformed hunk header in diff: {header}")]
    MalformedHunkHeader { header: String },
    #[error("malformed line inside a diff hunk: {line}")]
    MalformedDiff { line: String },
    #[error("rules directory {directory} contains no rule files")]
    NoRuleFiles { directory: PathBuf },
    #[error("rule file {path} has no valid ruleId in its front matter")]
    MissingRuleIdentifier { path: PathBuf },
    #[error("rule file {path} has no heading")]
    MissingRuleHeading { path: PathBuf },
}
