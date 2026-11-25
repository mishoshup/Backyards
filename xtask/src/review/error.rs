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
    #[error("model request timed out: {reason}")]
    ModelTimeout { reason: String },
    #[error("model connection failed: {reason}")]
    ModelConnection { reason: String },
    #[error("model returned HTTP {status}")]
    ModelStatus { status: u16 },
    #[error("model stream ended with finish reason {finish_reason}")]
    ModelTruncated { finish_reason: String },
    #[error("model returned an empty answer")]
    EmptyAnswer,
    #[error("model answer is not valid review JSON: {reason}")]
    InvalidAnswer { reason: String },
    #[error("malformed model stream event: {reason}")]
    MalformedStreamEvent { reason: String },
}

impl ReviewError {
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::ModelTimeout { .. } | Self::ModelConnection { .. } => true,
            Self::ModelStatus { status } => *status == 429 || *status >= 500,
            Self::DiffTooLarge { .. }
            | Self::MalformedHunkHeader { .. }
            | Self::MalformedDiff { .. }
            | Self::NoRuleFiles { .. }
            | Self::MissingRuleIdentifier { .. }
            | Self::MissingRuleHeading { .. }
            | Self::ModelTruncated { .. }
            | Self::EmptyAnswer
            | Self::InvalidAnswer { .. }
            | Self::MalformedStreamEvent { .. } => false,
        }
    }
}
