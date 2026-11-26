use crate::report::GateError;
use crate::review::error::ReviewError;
use crate::review::github::client::GitHub;
use crate::review::github::run_counter::MAXIMUM_RUNS;

pub const BOT_LOGIN: &str = "github-actions[bot]";
pub const PROGRESS_MARKER: &str = "<!-- backyards-agent-review -->";
const SHORT_SHA_LENGTH: usize = 7;
const COMMENTS_PER_PAGE: usize = 100;
const MAXIMUM_COMMENT_PAGES: usize = 30;
const PROGRESS_LABEL: &str = "progress";
const OTHER_LABEL: &str = "other";

#[derive(Debug, Clone)]
pub struct ProgressComment {
    pub identifier: String,
}

impl ProgressComment {
    #[must_use]
    pub fn short_sha(head_sha: &str) -> &str {
        head_sha.get(..SHORT_SHA_LENGTH).unwrap_or(head_sha)
    }

    #[must_use]
    pub fn started_body(head_sha: &str) -> String {
        format!(
            "{PROGRESS_MARKER}\nAgent review started for `{}`. Results will replace this text.",
            Self::short_sha(head_sha)
        )
    }
}

impl GitHub {
    pub fn create_progress_comment(
        &self,
        pull_request: u64,
        head_sha: &str,
    ) -> Result<ProgressComment, GateError> {
        let body_field = format!("body={}", ProgressComment::started_body(head_sha));
        let identifier = self.call_api(
            &format!("issues/{pull_request}/comments"),
            &["-f", &body_field, "--jq", ".id"],
        )?;
        Ok(ProgressComment {
            identifier: identifier.trim().to_owned(),
        })
    }

    pub fn update_progress_comment(
        &self,
        comment: &ProgressComment,
        body: &str,
    ) -> Result<(), GateError> {
        let body_field = format!("body={PROGRESS_MARKER}\n{body}");
        self.call_api(
            &format!("issues/comments/{}", comment.identifier),
            &["-X", "PATCH", "-f", &body_field],
        )
        .map(|_| ())
    }

    pub fn count_progress_comments(&self, pull_request: u64) -> Result<usize, GateError> {
        let mut counted = 0;
        for page in 1..=MAXIMUM_COMMENT_PAGES {
            let labels = self.label_comment_page(pull_request, page)?;
            counted += labels
                .iter()
                .filter(|label| label.as_str() == PROGRESS_LABEL)
                .count();
            if counted >= MAXIMUM_RUNS || labels.len() < COMMENTS_PER_PAGE {
                return Ok(counted);
            }
        }
        Err(ReviewError::CommentPagesExceeded {
            limit: MAXIMUM_COMMENT_PAGES,
        }
        .into())
    }

    fn label_comment_page(&self, pull_request: u64, page: usize) -> Result<Vec<String>, GateError> {
        let filter = format!(
            ".[] | if (.user.login == \"{BOT_LOGIN}\" and (.body | startswith(\"{PROGRESS_MARKER}\"))) then \"{PROGRESS_LABEL}\" else \"{OTHER_LABEL}\" end"
        );
        let route =
            format!("issues/{pull_request}/comments?per_page={COMMENTS_PER_PAGE}&page={page}");
        let output = self.call_api(&route, &["--jq", &filter])?;
        Ok(output.lines().map(str::to_owned).collect())
    }
}
