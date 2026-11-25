use crate::fixture::{
    CLEAN_MANIFEST, Fixture, any_message_contains, error_messages, warning_messages,
};
use xtask::checks::check_architecture;

fn layered_fixture() -> Fixture {
    let fixture = Fixture::create();
    for name in [
        "kernel",
        "manifest",
        "manifest-json",
        "gardens",
        "gardens-boxlite",
        "backyards",
    ] {
        fixture.write(&format!("crates/{name}/Cargo.toml"), CLEAN_MANIFEST);
        fixture.write(&format!("crates/{name}/src/lib.rs"), "pub fn run() {}\n");
    }
    fixture
}

#[test]
fn layered_workspace_has_no_violations() {
    let findings = check_architecture(&layered_fixture().repository()).unwrap();
    assert_eq!(error_messages(&findings), Vec::<String>::new());
}

#[test]
fn catch_all_file_name_is_reported() {
    let fixture = layered_fixture();
    fixture.write("crates/kernel/src/utils.rs", "pub fn run() {}\n");
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert!(any_message_contains(&findings, "catch-all file name"));
}

#[test]
fn runtime_macro_in_a_library_is_reported() {
    let fixture = layered_fixture();
    let macro_text = concat!("#[tokio", "::main]");
    fixture.write(
        "crates/gardens/src/lib.rs",
        &format!("{macro_text}\nasync fn run() {{}}\n"),
    );
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "runtime macro outside a binary entry point"
    ));
}

#[test]
fn runtime_macro_in_a_binary_entry_point_is_allowed() {
    let fixture = layered_fixture();
    let macro_text = concat!("#[tokio", "::main]");
    fixture.write(
        "crates/backyards/src/main.rs",
        &format!("{macro_text}\nasync fn main() {{}}\n"),
    );
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert_eq!(error_messages(&findings), Vec::<String>::new());
}

#[test]
fn operating_system_check_outside_platform_is_reported() {
    let fixture = layered_fixture();
    let attribute = concat!("#[cfg(target", "_os = \"macos\")]");
    fixture.write(
        "crates/kernel/src/lib.rs",
        &format!("{attribute}\nfn run() {{}}\n"),
    );
    fixture.write(
        "crates/kernel/src/platform/mac.rs",
        &format!("{attribute}\nfn run() {{}}\n"),
    );
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert_eq!(error_messages(&findings).len(), 1);
}

#[test]
fn file_over_the_hard_limit_is_an_error_and_over_the_soft_limit_a_warning() {
    let fixture = layered_fixture();
    fixture.write("crates/kernel/src/long.rs", &"fn a() {}\n".repeat(401));
    fixture.write("crates/kernel/src/medium.rs", &"fn a() {}\n".repeat(301));
    let findings = check_architecture(&fixture.repository()).unwrap();
    let errors = error_messages(&findings);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("long.rs"));
    let warnings = warning_messages(&findings);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("medium.rs has 301 lines"));
}

#[test]
fn anyhow_outside_the_binary_crate_is_reported() {
    let fixture = layered_fixture();
    fixture.write(
        "crates/kernel/Cargo.toml",
        &format!("{CLEAN_MANIFEST}\n[dependencies]\nanyhow = \"1\"\n"),
    );
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "uses anyhow outside the binary crate"
    ));
}

#[test]
fn feature_depending_on_another_feature_is_reported() {
    let fixture = layered_fixture();
    fixture.write(
        "crates/gardens/Cargo.toml",
        &format!("{CLEAN_MANIFEST}\n[dependencies]\nmanifest-json.workspace = true\n"),
    );
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "crates/gardens depends on manifest-json"
    ));
}

#[test]
fn adapter_may_depend_on_its_own_feature() {
    let fixture = layered_fixture();
    fixture.write(
        "crates/gardens-boxlite/Cargo.toml",
        &format!(
            "{CLEAN_MANIFEST}\n[dependencies]\ngardens.workspace = true\nkernel.workspace = true\n"
        ),
    );
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert_eq!(error_messages(&findings), Vec::<String>::new());
}

#[test]
fn product_crate_depending_on_tooling_is_reported() {
    let fixture = layered_fixture();
    fixture.write(
        "crates/kernel/Cargo.toml",
        &format!("{CLEAN_MANIFEST}\n[dependencies]\nxtask.workspace = true\n"),
    );
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "depends on the xtask tooling crate"
    ));
}

#[test]
fn front_end_depending_on_an_adapter_is_reported() {
    let fixture = layered_fixture();
    fixture.write(
        "crates/backyards-tui/Cargo.toml",
        &format!("{CLEAN_MANIFEST}\n[dependencies]\ngardens-boxlite.workspace = true\n"),
    );
    let findings = check_architecture(&fixture.repository()).unwrap();
    assert!(any_message_contains(
        &findings,
        "front end depends on an adapter"
    ));
}
