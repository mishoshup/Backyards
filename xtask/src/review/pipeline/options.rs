use crate::review::github::{AuthorAssociation, EventName, GitHub, RepositoryName};
use crate::review::pipeline::status_prefix::StatusPrefix;

#[derive(Debug, Clone)]
pub struct CiOptions {
    pub pull_request: u64,
    pub repository: RepositoryName,
    pub event_name: EventName,
    pub comment_body: String,
    pub comment_author_association: AuthorAssociation,
    pub status_prefix: StatusPrefix,
}

impl CiOptions {
    #[must_use]
    pub fn status_context(&self) -> String {
        format!("{}/pr-{}", self.status_prefix, self.pull_request)
    }
}

#[derive(Clone, Copy)]
pub struct ReviewRun<'run> {
    pub github: &'run GitHub,
    pub options: &'run CiOptions,
    pub head_sha: &'run str,
}
