#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("unknown tool {name}")]
    UnknownTool { name: String },
    #[error("invalid arguments: {reason}")]
    InvalidArguments { reason: String },
    #[error("cannot resolve path: {reason}")]
    UnresolvablePath { reason: String },
    #[error("path is outside the repository")]
    PathOutsideRepository,
    #[error("not a file")]
    NotAFile,
    #[error("invalid pattern: {reason}")]
    InvalidPattern { reason: String },
    #[error("cannot read file: {reason}")]
    ReadFailed { reason: String },
    #[error("not searched, not a regular file or larger than {limit} bytes")]
    NotSearchable { limit: u64 },
    #[error("only https urls are allowed")]
    OnlyHttps,
    #[error("host {host} is not an allowed documentation host")]
    HostNotAllowed { host: String },
    #[error("fetch failed: {reason}")]
    FetchFailed { reason: String },
    #[error("git listing failed: {reason}")]
    ListingFailed { reason: String },
}
