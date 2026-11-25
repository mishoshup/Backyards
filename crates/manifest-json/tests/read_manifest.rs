use manifest::ManifestReader;
use manifest_json::JsonManifestReader;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ExampleSection {
    value: u32,
}

#[test]
fn reads_a_declared_section() {
    let text = r#"{"schema_version":1,"example":{"value":7}}"#;
    let manifest = JsonManifestReader.read(text).expect("valid");
    let section: Option<ExampleSection> = manifest.section("example").expect("section is valid");
    assert_eq!(section, Some(ExampleSection { value: 7 }));
}

#[test]
fn absent_section_is_none() {
    let manifest = JsonManifestReader
        .read(r#"{"schema_version":1}"#)
        .expect("valid");
    let section: Option<ExampleSection> = manifest.section("example").expect("no error");
    assert_eq!(section, None);
}

#[test]
fn reports_an_invalid_section_with_its_name() {
    let manifest = JsonManifestReader
        .read(r#"{"schema_version":1,"example":{"value":"text"}}"#)
        .expect("valid envelope");
    let error = manifest
        .section::<ExampleSection>("example")
        .expect_err("must fail");
    assert!(error.to_string().contains("section example is invalid"));
}

#[test]
fn lists_sections_no_feature_claimed() {
    let manifest = JsonManifestReader
        .read(r#"{"schema_version":1,"gardens":[],"typo":{}}"#)
        .expect("valid");
    assert_eq!(manifest.unclaimed_sections(&["gardens"]), ["typo"]);
}

#[test]
fn rejects_unsupported_schema_version() {
    let error = JsonManifestReader
        .read(r#"{"schema_version":2}"#)
        .expect_err("must fail");
    assert!(error.to_string().contains("schema_version 2"));
}

#[test]
fn rejects_missing_schema_version() {
    assert!(JsonManifestReader.read(r#"{"gardens":[]}"#).is_err());
}
