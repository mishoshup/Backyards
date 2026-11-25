use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use xtask::review::{
    Confidence, GitHub, PostTarget, ReviewerMode, RunPermission, StatusState, Violation,
    build_footer, build_summary_body, decide_run_permission, has_reviewable_path, publish_findings,
    should_skip_review,
};

use crate::fixture::Fixture;

const HEAD_SHA: &str = "0123456789abcdef0123456789abcdef01234567";

struct FakeGh {
    program: PathBuf,
    log: PathBuf,
}

impl FakeGh {
    fn install(directory: &Path, failing_marker: &str, output: &str) -> Self {
        let program = directory.join("fake-gh");
        let log = directory.join("gh-calls.log");
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n<<END>>\\n' \"$*\" >> '{}'\ncase \"$*\" in *'{failing_marker}'*) exit 1;; esac\nprintf '%s' '{output}'\n",
            log.display()
        );
        std::fs::write(&program, script).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        Self { program, log }
    }

    fn github(&self, directory: &Path) -> GitHub {
        GitHub::with_program(
            self.program.to_string_lossy().into_owned(),
            "owner/repository".to_owned(),
            directory.to_path_buf(),
        )
    }

    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(&self.log)
            .map(|text| {
                text.split("\n<<END>>\n")
                    .filter(|call| !call.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn violation(path: &str, line: u32, confidence: Confidence) -> Violation {
    Violation {
        rule_identifier: "STYLE-1".to_owned(),
        path: path.to_owned(),
        line,
        body: format!("STYLE-1 problem at {line}"),
        confidence,
    }
}

fn target() -> PostTarget<'static> {
    PostTarget {
        pull_request: 7,
        head_sha: HEAD_SHA,
    }
}

fn setup(failing_marker: &str, output: &str) -> (Fixture, FakeGh) {
    let fixture = Fixture::create();
    let fake = FakeGh::install(fixture.path(), failing_marker, output);
    (fixture, fake)
}

#[test]
fn valid_findings_are_posted_inline_with_footer_and_commit() {
    let (fixture, fake) = setup("never-matches", "");
    let github = fake.github(fixture.path());
    let findings = [violation("src/a.rs", 3, Confidence::Low)];
    publish_findings(&github, target(), ReviewerMode::Inline, &findings).unwrap();
    let calls = fake.calls();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].contains("pulls/7/comments"), "{}", calls[0]);
    assert!(calls[0].contains("path=src/a.rs"));
    assert!(calls[0].contains("line=3"));
    assert!(calls[0].contains(&format!("commit_id={HEAD_SHA}")));
    assert!(calls[0].contains("Low confidence: STYLE-1 problem at 3"));
    assert!(calls[0].contains(&build_footer("owner/repository", HEAD_SHA)));
}

#[test]
fn findings_that_fail_inline_placement_move_to_the_summary_comment() {
    let (fixture, fake) = setup("pulls/7/comments", "");
    let github = fake.github(fixture.path());
    let findings = [violation("src/a.rs", 3, Confidence::High)];
    publish_findings(&github, target(), ReviewerMode::Holistic, &findings).unwrap();
    let calls = fake.calls();
    assert_eq!(calls.len(), 4);
    assert!(
        calls[..3]
            .iter()
            .all(|call| call.contains("pulls/7/comments"))
    );
    assert!(calls[3].contains("issues/7/comments"));
    assert!(calls[3].contains("`src/a.rs:3` STYLE-1 problem at 3"));
    assert!(calls[3].contains("1 inline placement failed"));
}

#[test]
fn findings_beyond_the_posting_cap_are_summarized_not_dropped() {
    let (fixture, fake) = setup("never-matches", "");
    let github = fake.github(fixture.path());
    let findings: Vec<Violation> = (1..=32)
        .map(|line| violation("src/a.rs", line, Confidence::High))
        .collect();
    publish_findings(&github, target(), ReviewerMode::Inline, &findings).unwrap();
    let calls = fake.calls();
    assert_eq!(calls.len(), 31);
    assert!(calls[30].contains("issues/7/comments"));
    assert!(calls[30].contains("`src/a.rs:32`"));
    assert!(!calls[30].contains("`src/a.rs:30`"));
}

#[test]
fn no_findings_post_nothing() {
    let (fixture, fake) = setup("never-matches", "");
    let github = fake.github(fixture.path());
    publish_findings(&github, target(), ReviewerMode::Inline, &[]).unwrap();
    assert_eq!(fake.calls().len(), 0);
}

#[test]
fn summary_body_reports_omitted_findings() {
    let findings: Vec<Violation> = (1..=101)
        .map(|line| violation("src/a.rs", line, Confidence::High))
        .collect();
    let body = build_summary_body(ReviewerMode::Inline, &findings);
    assert!(body.ends_with("1 further finding omitted."), "{body}");
}

#[test]
fn progress_comments_are_counted_from_the_listing() {
    let (fixture, fake) = setup("never-matches", "11\n22\n");
    let github = fake.github(fixture.path());
    assert_eq!(github.count_progress_comments(7).unwrap(), 2);
}

#[test]
fn third_run_is_refused_unless_forced() {
    assert_eq!(
        decide_run_permission(1, "/agent-review"),
        RunPermission::Allowed
    );
    assert_eq!(
        decide_run_permission(2, "/agent-review"),
        RunPermission::Refused { previous_runs: 2 }
    );
    assert_eq!(
        decide_run_permission(5, "please /agent-review force"),
        RunPermission::Allowed
    );
}

#[test]
fn reviewed_commit_is_skipped_except_for_an_explicit_comment() {
    assert!(should_skip_review(
        "pull_request",
        Some(StatusState::Success)
    ));
    assert!(!should_skip_review(
        "issue_comment",
        Some(StatusState::Success)
    ));
    assert!(!should_skip_review(
        "pull_request",
        Some(StatusState::Failure)
    ));
    assert!(!should_skip_review("pull_request", None));
}

#[test]
fn only_rust_and_policy_paths_are_reviewable() {
    let docs = vec!["README.md".to_owned(), "docs/notes.txt".to_owned()];
    assert!(!has_reviewable_path(&docs).unwrap());
    let code = vec!["README.md".to_owned(), "xtask/src/main.rs".to_owned()];
    assert!(has_reviewable_path(&code).unwrap());
}
