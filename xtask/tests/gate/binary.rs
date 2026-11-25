use std::process::{Command, Output};

use crate::fixture::{CLEAN_MANIFEST, Fixture};

fn run_xtask(fixture: &Fixture, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(arguments)
        .arg("--repository-root")
        .arg(fixture.path())
        .output()
        .unwrap()
}

#[test]
fn policy_exits_zero_on_a_clean_repository() {
    let fixture = Fixture::create();
    fixture.write("crates/demo/Cargo.toml", CLEAN_MANIFEST);
    fixture.write("crates/demo/src/lib.rs", "pub fn demo() {}\n");
    let output = run_xtask(&fixture, &["policy"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(output.stderr, Vec::<u8>::new());
}

#[test]
fn policy_exits_nonzero_and_reports_the_violation_on_stderr() {
    let fixture = Fixture::create();
    fixture.write("crates/demo/Cargo.toml", CLEAN_MANIFEST);
    fixture.write(
        "crates/demo/src/lib.rs",
        &format!("// {}: later\npub fn demo() {{}}\n", concat!("TO", "DO")),
    );
    let output = run_xtask(&fixture, &["policy"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, Vec::<u8>::new());
    let report = String::from_utf8(output.stderr).unwrap();
    assert!(report.contains("marker comment"));
    assert!(report.contains("1 violation"));
}

#[test]
fn keep_going_names_every_failed_stage() {
    let fixture = Fixture::create();
    let output = run_xtask(&fixture, &["check", "--keep-going"]);
    assert_eq!(output.status.code(), Some(1));
    let report = String::from_utf8(output.stderr).unwrap();
    let summary = report
        .lines()
        .find(|line| line.starts_with("failed stages:"))
        .unwrap();
    assert!(report.contains("formatting: to fix, run `cargo fmt`"));
    assert!(summary.contains("formatting"));
    assert!(summary.contains("lint"));
    assert!(summary.contains("tests"));
}

#[test]
fn fail_fast_names_the_stages_it_skipped() {
    let fixture = Fixture::create();
    let output = run_xtask(&fixture, &["check"]);
    assert_eq!(output.status.code(), Some(1));
    let report = String::from_utf8(output.stderr).unwrap();
    let skipped = report
        .lines()
        .find(|line| line.starts_with("skipped stages"))
        .unwrap();
    assert!(skipped.contains("dependencies"));
}
