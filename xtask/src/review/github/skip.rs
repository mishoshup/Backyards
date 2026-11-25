const COMMENT_EVENT: &str = "issue_comment";
const SUCCESS_STATE: &str = "success";

#[must_use]
pub fn should_skip_review(event_name: &str, existing_state: Option<&str>) -> bool {
    event_name != COMMENT_EVENT && existing_state == Some(SUCCESS_STATE)
}
