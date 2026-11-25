use crate::review::github::status::StatusState;

const COMMENT_EVENT: &str = "issue_comment";

#[must_use]
pub fn should_skip_review(event_name: &str, existing_state: Option<StatusState>) -> bool {
    event_name != COMMENT_EVENT && existing_state == Some(StatusState::Success)
}
