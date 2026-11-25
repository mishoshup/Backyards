use crate::fixture::{CLEAN_MANIFEST, Fixture, any_message_contains, error_messages};
use xtask::checks::check_policy;

fn clean_fixture() -> Fixture {
    let fixture = Fixture::create();
    fixture.write("crates/kernel/Cargo.toml", CLEAN_MANIFEST);
    fixture.write("crates/kernel/src/lib.rs", "pub fn run() {}\n");
    fixture
}

#[test]
fn clean_repository_has_no_violations() {
    let fixture = clean_fixture();
    let findings = check_policy(&fixture.repository()).unwrap();
    assert_eq!(error_messages(&findings), Vec::<String>::new());
}

#[test]
fn marker_comment_is_reported_with_file_and_line() {
    let fixture = clean_fixture();
    let marker = concat!("// TO", "DO: later");
    fixture.write(
        "crates/kernel/src/lib.rs",
        &format!("pub fn run() {{}}\n{marker}\n"),
    );
    let findings = check_policy(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "crates/kernel/src/lib.rs:2"
    ));
}

#[test]
fn em_dash_in_documentation_is_reported() {
    let fixture = clean_fixture();
    fixture.write("README.md", "one \u{2014} two\n");
    let findings = check_policy(&fixture.repository()).unwrap();
    assert!(any_message_contains(&findings, "em dash: README.md:1"));
}

#[test]
fn crate_without_workspace_lints_is_reported() {
    let fixture = clean_fixture();
    fixture.write("crates/kernel/Cargo.toml", "[package]\nname = \"kernel\"\n");
    let findings = check_policy(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "must contain [lints] workspace = true"
    ));
}

#[test]
fn lint_opt_out_attribute_is_reported() {
    let fixture = clean_fixture();
    let attribute = concat!("#[al", "low(clippy::unwrap_used)]");
    fixture.write(
        "crates/kernel/src/lib.rs",
        &format!("{attribute}\npub fn run() {{}}\n"),
    );
    let findings = check_policy(&fixture.repository()).unwrap();
    assert!(any_message_contains(&findings, "lint opt-out"));
}

#[test]
fn source_install_command_is_reported() {
    let fixture = clean_fixture();
    let command = concat!("cargo ", "install ripgrep");
    fixture.write("docs.md", &format!("run {command}\n"));
    let findings = check_policy(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "use cargo binstall instead"
    ));
}

#[test]
fn untracked_file_listed_by_git_is_still_scanned() {
    let fixture = clean_fixture();
    fixture.write("notes.md", "bad \u{2014} dash\n");
    let findings = check_policy(&fixture.repository()).unwrap();
    assert_ne!(error_messages(&findings), Vec::<String>::new());
}
