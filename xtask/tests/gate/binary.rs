use std::process::{Command, Output};

use crate::support::{CLEAN_MANIFEST, Fixture};

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
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
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
    assert!(output.stdout.is_empty());
    let report = String::from_utf8(output.stderr).unwrap();
    assert!(report.contains("marker comment"));
    assert!(report.contains("1 violation"));
}
