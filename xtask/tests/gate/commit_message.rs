use xtask::commit_message::check_commit_message;
use xtask::finding::count_errors;

fn error_count(message: &str) -> usize {
    count_errors(&check_commit_message(message).unwrap())
}

#[test]
fn conventional_subject_passes() {
    assert_eq!(
        error_count("feat(cli): add validate command\n\nBody text.\n"),
        0
    );
}

#[test]
fn trailing_period_is_rejected() {
    assert_eq!(error_count("feat(cli): add validate command.\n"), 1);
}

#[test]
fn uppercase_subject_is_rejected() {
    assert_eq!(error_count("feat(cli): Add validate command\n"), 1);
}

#[test]
fn subject_longer_than_the_limit_is_rejected() {
    let subject = format!("feat(cli): {}", "a".repeat(70));
    assert_eq!(error_count(&subject), 1);
}

#[test]
fn attribution_trailers_are_rejected_in_any_case() {
    let trailer = concat!("co-authored", "-by: someone");
    assert_eq!(
        error_count(&format!("feat(cli): add a thing\n\n{trailer}\n")),
        1
    );
    let session = concat!("Claude-", "Session: https://example.test");
    assert_eq!(
        error_count(&format!("feat(cli): add a thing\n\n{session}\n")),
        1
    );
}
