use crate::review::github::event_name::EventName;
use crate::review::github::status::StatusState;

#[must_use]
pub fn should_skip_review(event_name: EventName, existing_state: Option<StatusState>) -> bool {
    event_name != EventName::IssueComment && existing_state == Some(StatusState::Success)
}
