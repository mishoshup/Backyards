use crate::fixture::{Fixture, any_message_contains, error_messages};
use xtask::checks::check_workflow_policy;

const GOOD_WORKFLOW: &str = "permissions: {}\non:\n  pull_request:\n    types: [opened, ready_for_review]\njobs:\n  review:\n    if: github.event.pull_request.head.repo.full_name == github.repository\n    timeout-minutes: 25\n    steps:\n      - run: cargo xtask review pull-request\n        with:\n          ref: refs/pull/1/head\n";

fn workflow_fixture(workflow: &str) -> Fixture {
    let fixture = Fixture::create();
    fixture.write(".github/workflows/agent-code-review.yml", workflow);
    fixture.write(
        "xtask/src/review/pipeline/run.rs",
        "capture_command(root, \"git\", &[\"rev-parse\", \"HEAD\"], &[0])",
    );
    fixture
}

#[test]
fn pipeline_without_local_head_read_is_reported() {
    let fixture = workflow_fixture(GOOD_WORKFLOW);
    fixture.write("xtask/src/review/pipeline/run.rs", "fn run() {}");
    let findings = check_workflow_policy(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "commit under review from the checked-out HEAD"
    ));
}

#[test]
fn compliant_workflow_has_no_violations() {
    let findings = check_workflow_policy(&workflow_fixture(GOOD_WORKFLOW).repository()).unwrap();
    assert_eq!(error_messages(&findings), Vec::<String>::new());
}

#[test]
fn pull_request_trigger_without_explicit_types_is_reported() {
    let workflow = GOOD_WORKFLOW.replace("    types: [opened, ready_for_review]\n", "");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "only on opened and ready_for_review"
    ));
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
    let workflow = GOOD_WORKFLOW.replace("25", "20");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "timeout must be 25 minutes"
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
fn synchronize_trigger_is_reported() {
    let workflow = GOOD_WORKFLOW.replace("[opened, ready_for_review]", "[synchronize]");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "only on opened and ready_for_review"
    ));
}

#[test]
fn missing_rust_pipeline_command_is_reported() {
    let workflow = GOOD_WORKFLOW.replace("cargo xtask review pull-request", "echo skipped");
    let findings = check_workflow_policy(&workflow_fixture(&workflow).repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "must run through cargo xtask review pull-request"
    ));
}
