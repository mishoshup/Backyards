use crate::support::{Fixture, any_message_contains, error_messages};
use xtask::checks::check_workflow_policy;

const GOOD_WORKFLOW: &str = "permissions: {}\njobs:\n  review:\n    if: github.event.pull_request.head.repo.full_name == github.repository\n    timeout-minutes: 45\n    steps:\n      - with:\n          ref: refs/pull/1/head\n";

fn workflow_fixture(workflow: &str) -> Fixture {
    let fixture = Fixture::create();
    fixture.write(".github/workflows/agent-code-review.yml", workflow);
    fixture.write(
        ".github/scripts/review/should-skip-review.sh",
        "git rev-parse HEAD\n",
    );
    fixture.write("scripts/review/allowed-fetch-hosts.txt", "example.test\n");
    fixture.stage("scripts/review/allowed-fetch-hosts.txt");
    fixture.write("scripts/review/run-local-review.sh", "gh -f \"$prompt\"\n");
    fixture
}

#[test]
fn compliant_workflow_has_no_violations() {
    let findings = check_workflow_policy(&workflow_fixture(GOOD_WORKFLOW).repository()).unwrap();
    assert_eq!(error_messages(&findings), Vec::<String>::new());
}

#[test]
fn cancelling_review_runs_is_reported() {
    let workflow = format!("{GOOD_WORKFLOW}concurrency:\n  cancel-in-progress: true\n");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "must not cancel in progress"
    ));
}

#[test]
fn short_timeout_is_reported() {
    let workflow = GOOD_WORKFLOW.replace("45", "30");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "timeout must be at least 45"
    ));
}

#[test]
fn missing_empty_permissions_is_reported() {
    let workflow = GOOD_WORKFLOW.replace("permissions: {}", "permissions: write-all");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "default permissions must be empty"
    ));
}

#[test]
fn missing_fork_refusal_is_reported() {
    let workflow = GOOD_WORKFLOW.replace("head.repo.full_name == github.repository", "true");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "fork pull requests must be refused"
    ));
}

#[test]
fn unpinned_checkout_is_reported() {
    let workflow = GOOD_WORKFLOW.replace("refs/pull/", "refs/heads/");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "checkout must name the pull request ref"
    ));
}

#[test]
fn untracked_allowlist_is_reported() {
    let fixture = Fixture::create();
    fixture.write(".github/workflows/agent-code-review.yml", GOOD_WORKFLOW);
    fixture.write(
        ".github/scripts/review/should-skip-review.sh",
        "git rev-parse HEAD\n",
    );
    fixture.write("scripts/review/allowed-fetch-hosts.txt", "example.test\n");
    fixture.write("scripts/review/run-local-review.sh", "gh -f \"$prompt\"\n");
    let findings = check_workflow_policy(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "allowed-fetch-hosts.txt must be listed by git"
    ));
}
