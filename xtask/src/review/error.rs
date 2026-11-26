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
    #[error("model returned HTTP {status}: {detail}")]
    ModelStatus { status: u16, detail: String },
    #[error("commit status has an unknown state {value}")]
    UnknownStatusState { value: String },
    #[error("progress report could not be written: {reason}")]
    ProgressWrite { reason: String },
    #[error("model stream ended with finish reason {finish_reason}")]
    ModelTruncated { finish_reason: String },
    #[error("model output over the {limit} byte limit in {part}")]
    ModelOutputTooLarge { part: &'static str, limit: usize },
    #[error("model returned an empty answer")]
    EmptyAnswer,
    #[error("model answer is not valid review JSON: {reason}")]
    InvalidAnswer { reason: String },
    #[error("malformed model stream event: {reason}")]
    MalformedStreamEvent { reason: String },
    #[error("{details}")]
    ReviewersFailed { details: String },
    #[error("{cause}; recording the failure also failed: {recording}")]
    FailureNotRecorded { cause: String, recording: String },
    #[error("environment variable {name} is {reason}")]
    MissingEnvironment {
        name: &'static str,
        reason: &'static str,
    },
}

impl ReviewError {
    #[must_use]
    pub const fn unusable_environment(name: &'static str, error: &std::env::VarError) -> Self {
        let reason = match error {
            std::env::VarError::NotPresent => "not set",
            std::env::VarError::NotUnicode(_) => "not valid unicode",
        };
        Self::MissingEnvironment { name, reason }
    }

    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::ModelTimeout { .. } | Self::ModelConnection { .. } | Self::EmptyAnswer => true,
            Self::ModelStatus { status, .. } => *status == 429 || *status >= 500,
            Self::DiffTooLarge { .. }
            | Self::MalformedHunkHeader { .. }
            | Self::MalformedDiff { .. }
            | Self::NoRuleFiles { .. }
            | Self::MissingRuleIdentifier { .. }
            | Self::MissingRuleHeading { .. }
            | Self::ModelTruncated { .. }
            | Self::ModelOutputTooLarge { .. }
            | Self::InvalidAnswer { .. }
            | Self::MalformedStreamEvent { .. }
            | Self::ProgressWrite { .. }
            | Self::UnknownStatusState { .. }
            | Self::ReviewersFailed { .. }
            | Self::FailureNotRecorded { .. }
            | Self::MissingEnvironment { .. } => false,
        }
    }
}
