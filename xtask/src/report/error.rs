use std::path::PathBuf;
use std::process::ExitStatus;

#[derive(Debug, thiserror::Error)]
pub enum GateError {
    #[error("cannot read {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write {path}: {source}")]
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot remove {path}: {source}")]
    RemoveFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot list {path}: {source}")]
    ReadDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot run {command}: {source}")]
    RunCommand {
        command: String,
        source: std::io::Error,
    },
    #[error("{command} exited with {status}: {stderr}")]
    CommandFailed {
        command: String,
        status: ExitStatus,
        stderr: String,
    },
    #[error("git listed a path that is not valid UTF-8: {0}")]
    NonUtf8Path(#[source] std::str::Utf8Error),
    #[error("{directory} has an entry whose name is not valid UTF-8: {name:?}")]
    NonUtf8DirectoryName {
        directory: PathBuf,
        name: std::ffi::OsString,
    },
    #[error("cannot write the report: {0}")]
    WriteReport(#[source] std::io::Error),
    #[error("invalid pattern {pattern}: {source}")]
    InvalidPattern {
        pattern: String,
        source: regex::Error,
    },
    #[error("failed stages: {}", names.join(", "))]
    StagesFailed { names: Vec<&'static str> },
    #[error(transparent)]
    Review(#[from] crate::review::ReviewError),
    #[error("{area}: {}", crate::report::pluralize(*count, "violation", "violations"))]
    Violations { area: &'static str, count: usize },
}
