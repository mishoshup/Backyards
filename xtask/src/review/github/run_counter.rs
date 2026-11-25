pub const MAXIMUM_RUNS: usize = 2;
pub const FORCE_PHRASE: &str = "/agent-review force";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPermission {
    Allowed,
    Refused { previous_runs: usize },
}

#[must_use]
pub fn decide_run_permission(previous_runs: usize, comment_body: &str) -> RunPermission {
    if previous_runs < MAXIMUM_RUNS || comment_body.contains(FORCE_PHRASE) {
        RunPermission::Allowed
    } else {
        RunPermission::Refused { previous_runs }
    }
}
