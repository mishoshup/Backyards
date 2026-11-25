pub const MAXIMUM_RUNS: usize = 2;
pub const FORCE_PHRASE: &str = "/agent-review force";
const FORCE_AUTHOR_ASSOCIATION: &str = "OWNER";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPermission {
    Allowed,
    Refused { previous_runs: usize },
}

#[must_use]
pub fn decide_run_permission(
    previous_runs: usize,
    comment_body: &str,
    author_association: &str,
) -> RunPermission {
    if previous_runs < MAXIMUM_RUNS || is_owner_force(comment_body, author_association) {
        RunPermission::Allowed
    } else {
        RunPermission::Refused { previous_runs }
    }
}

fn is_owner_force(comment_body: &str, author_association: &str) -> bool {
    comment_body.contains(FORCE_PHRASE) && author_association == FORCE_AUTHOR_ASSOCIATION
}
