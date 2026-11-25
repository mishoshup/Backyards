use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EventName {
    #[value(name = "pull_request")]
    PullRequest,
    #[value(name = "issue_comment")]
    IssueComment,
}
