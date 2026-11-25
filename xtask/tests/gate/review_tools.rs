use crate::fixture::Fixture;
use xtask::review::ToolRunner;

fn fixture_with_sources() -> Fixture {
    let fixture = Fixture::create();
    fixture.write("src/lib.rs", "fn one() {}\nfn two() {}\nfn three() {}\n");
    fixture.write("src/nested/deep.rs", "fn deep() {}\n");
    fixture.write("notes.md", "documentation\n");
    fixture
}

fn run(fixture: &Fixture, name: &str, arguments: &str) -> String {
    ToolRunner::new(fixture.path()).run(name, arguments)
}

#[test]
fn read_file_numbers_the_requested_range() {
    let fixture = fixture_with_sources();
    let output = run(
        &fixture,
        "read_file",
        r#"{"path":"src/lib.rs","start_line":2,"end_line":3}"#,
    );
    assert_eq!(output, "2\tfn two() {}\n3\tfn three() {}");
}

#[test]
fn read_file_marks_output_cut_at_the_size_cap() {
    let fixture = Fixture::create();
    let long_file = "0123456789\n".repeat(5_000);
    fixture.write("big.txt", &long_file);
    let output = run(&fixture, "read_file", r#"{"path":"big.txt"}"#);
    assert!(output.ends_with("[output truncated after 30000 bytes]"));
}

#[test]
fn read_file_rejects_parent_traversal() {
    let fixture = fixture_with_sources();
    let outside_name = format!("backyards-outside-{}.txt", std::process::id());
    let outside = fixture.path().with_file_name(&outside_name);
    std::fs::write(&outside, "secret").unwrap();
    let output = run(
        &fixture,
        "read_file",
        &format!(r#"{{"path":"../{outside_name}"}}"#),
    );
    std::fs::remove_file(&outside).unwrap();
    assert!(output.contains("outside the repository"), "{output}");
}

#[test]
fn read_file_rejects_an_absolute_path_outside_the_repository() {
    let fixture = fixture_with_sources();
    let output = run(&fixture, "read_file", r#"{"path":"/etc/hosts"}"#);
    assert!(output.contains("outside the repository"), "{output}");
}

#[cfg(unix)]
#[test]
fn read_file_rejects_a_symlink_leaving_the_repository() {
    let fixture = fixture_with_sources();
    std::os::unix::fs::symlink("/etc/hosts", fixture.path().join("link.txt")).unwrap();
    let output = run(&fixture, "read_file", r#"{"path":"link.txt"}"#);
    assert!(output.contains("outside the repository"), "{output}");
}

#[cfg(unix)]
#[test]
fn grep_does_not_follow_a_symlink_leaving_the_repository() {
    let fixture = fixture_with_sources();
    std::os::unix::fs::symlink("/etc/hosts", fixture.path().join("link.txt")).unwrap();
    let output = run(&fixture, "grep", r#"{"pattern":"localhost"}"#);
    assert!(!output.contains("link.txt:1:"), "{output}");
    assert!(output.contains("link.txt: "), "{output}");
}

#[test]
fn list_files_matches_single_and_double_star_globs() {
    let fixture = fixture_with_sources();
    let shallow = run(&fixture, "list_files", r#"{"glob":"src/*.rs"}"#);
    assert_eq!(shallow, "src/lib.rs");
    let deep = run(&fixture, "list_files", r#"{"glob":"**/*.rs"}"#);
    assert!(deep.contains("src/nested/deep.rs") && deep.contains("src/lib.rs"));
}

#[test]
fn grep_reports_path_line_and_text() {
    let fixture = fixture_with_sources();
    let output = run(&fixture, "grep", r#"{"pattern":"fn t(wo|hree)"}"#);
    assert_eq!(
        output,
        "src/lib.rs:2:fn two() {}\nsrc/lib.rs:3:fn three() {}"
    );
}

#[test]
fn grep_scope_limits_the_search() {
    let fixture = fixture_with_sources();
    let output = run(&fixture, "grep", r#"{"pattern":"fn","path":"src/nested"}"#);
    assert_eq!(output, "src/nested/deep.rs:1:fn deep() {}");
}

#[test]
fn grep_reports_an_invalid_pattern() {
    let fixture = fixture_with_sources();
    let output = run(&fixture, "grep", r#"{"pattern":"("}"#);
    assert!(output.starts_with("error: invalid pattern"), "{output}");
}

#[test]
fn fetch_url_refuses_plain_http_and_unlisted_hosts() {
    let fixture = fixture_with_sources();
    let plain = run(&fixture, "fetch_url", r#"{"url":"http://docs.rs/x"}"#);
    assert!(plain.contains("only https"), "{plain}");
    let unlisted = run(&fixture, "fetch_url", r#"{"url":"https://example.com/x"}"#);
    assert!(
        unlisted.contains("not an allowed documentation host"),
        "{unlisted}"
    );
    let lookalike = run(
        &fixture,
        "fetch_url",
        r#"{"url":"https://docs.rs.evil.test/x"}"#,
    );
    assert!(lookalike.contains("not an allowed"), "{lookalike}");
    let userinfo = run(
        &fixture,
        "fetch_url",
        r#"{"url":"https://docs.rs:443@attacker.example/x"}"#,
    );
    assert!(userinfo.contains("not an allowed"), "{userinfo}");
}

#[test]
fn grep_names_files_it_could_not_search() {
    let fixture = fixture_with_sources();
    fixture.write("data/binary.dat", "\u{0}");
    std::fs::write(fixture.path().join("data/invalid.txt"), [0xff_u8, 0xfe]).unwrap();
    let output = run(&fixture, "grep", r#"{"pattern":"fn two"}"#);
    assert!(output.contains("[skipped files]"), "{output}");
    assert!(
        output.contains("data/invalid.txt: cannot read file"),
        "{output}"
    );
}

#[test]
fn tool_arguments_reject_unknown_keys() {
    let fixture = fixture_with_sources();
    let output = run(&fixture, "read_file", r#"{"path":"src/lib.rs","lines":3}"#);
    assert!(output.contains("invalid arguments"), "{output}");
}

#[test]
fn unknown_tool_and_bad_arguments_are_reported() {
    let fixture = fixture_with_sources();
    assert!(run(&fixture, "delete_file", "{}").contains("unknown tool"));
    assert!(run(&fixture, "read_file", "not json").contains("invalid arguments"));
}
