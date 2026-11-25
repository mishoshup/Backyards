use crate::report::GateError;
use crate::review::github::client::GitHub;

pub const BOT_LOGIN: &str = "github-actions[bot]";
pub const PROGRESS_MARKER: &str = "<!-- backyards-agent-review -->";
const SHORT_SHA_LENGTH: usize = 7;

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
        let identifier = self.api(
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
        self.api(
            &format!("issues/comments/{}", comment.identifier),
            &["-X", "PATCH", "-f", &body_field],
        )
        .map(|_| ())
    }

    pub fn count_progress_comments(&self, pull_request: u64) -> Result<usize, GateError> {
        let filter = format!(
            ".[] | select(.user.login == \"{BOT_LOGIN}\" and (.body | contains(\"{PROGRESS_MARKER}\"))) | .id"
        );
        let output = self.api(
            &format!("issues/{pull_request}/comments"),
            &["--paginate", "--jq", &filter],
        )?;
        Ok(output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count())
    }
}
