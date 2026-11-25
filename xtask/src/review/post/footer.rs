use crate::review::github::ProgressComment;

#[must_use]
pub fn build_footer(repository: &str, head_sha: &str) -> String {
    format!(
        "\n\n---\n\nReviewed at: [{}](https://github.com/{repository}/commit/{head_sha}) | Rate this suggestion with a thumbs up or down; reactions measure reviewer quality.",
        ProgressComment::short_sha(head_sha)
    )
}
