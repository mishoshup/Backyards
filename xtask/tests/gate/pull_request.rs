use xtask::checks::{check_pull_request_description, check_pull_request_naming};
use xtask::report::count_errors;

fn naming_errors(title: &str, branch: &str) -> usize {
    count_errors(&check_pull_request_naming(title, branch).unwrap())
}

fn description_errors(body: &str) -> usize {
    count_errors(&check_pull_request_description(body).unwrap())
}

#[test]
fn matching_title_and_branch_pass() {
    assert_eq!(
        naming_errors("feat(cli): add validate", "feat/add-validate"),
        0
    );
}

#[test]
fn title_without_scope_is_rejected() {
    assert!(naming_errors("feat: add validate", "feat/add-validate") >= 1);
}

#[test]
fn branch_with_uppercase_is_rejected() {
    assert!(naming_errors("feat(cli): add validate", "feat/Add-Validate") >= 1);
}

#[test]
fn branch_type_must_match_title_type() {
    assert_eq!(
        naming_errors("feat(cli): add validate", "fix/add-validate"),
        1
    );
}

#[test]
fn long_title_is_rejected() {
    let title = format!("feat(cli): {}", "a".repeat(70));
    assert!(naming_errors(&title, "feat/long") >= 1);
}

const COMPLETE_BODY: &str = "### Problem\nx\n### Change\ny\n### Verification\nz\n- [x] done\n";

#[test]
fn complete_description_passes() {
    assert_eq!(description_errors(COMPLETE_BODY), 0);
}

#[test]
fn missing_section_is_reported() {
    assert_eq!(description_errors("### Problem\nx\n### Change\ny\n"), 1);
}

#[test]
fn unchecked_box_is_reported_once_for_all_boxes() {
    let body = format!("{COMPLETE_BODY}- [ ] one\n- [ ] two\n");
    assert_eq!(description_errors(&body), 1);
}

#[test]
fn empty_description_reports_every_section() {
    assert_eq!(description_errors(""), 3);
}
