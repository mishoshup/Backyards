use gardens::{GardenName, GardensSection, NetworkPolicy, PlanAction, plan_gardens};
use serde_json::Value;

const EXAMPLE: &str = include_str!("../../../manifests/example.backyards.json");

fn parse(text: &str) -> Result<GardensSection, serde_json::Error> {
    serde_json::from_str(text)
}

#[test]
fn reads_the_example_manifest_section() {
    let document: Value = serde_json::from_str(EXAMPLE).expect("example is JSON");
    let section: GardensSection =
        serde_json::from_value(document[GardensSection::NAME].clone()).expect("valid section");
    let specification = &section.gardens()[0];
    assert_eq!(specification.name.as_str(), "dev");
    assert_eq!(specification.cpu_count.get(), 2);
    assert_eq!(specification.network, NetworkPolicy::Disabled);
}

#[test]
fn network_defaults_to_disabled() {
    let section = parse(r#"[{"name":"a","image":"alpine"}]"#).expect("valid");
    assert_eq!(section.gardens()[0].network, NetworkPolicy::Disabled);
}

#[test]
fn rejects_unknown_fields() {
    assert!(parse(r#"[{"name":"a","image":"x","extra":1}]"#).is_err());
}

#[test]
fn rejects_invalid_and_duplicate_names() {
    assert!(parse(r#"[{"name":"Bad_Name","image":"x"}]"#).is_err());
    let error =
        parse(r#"[{"name":"a","image":"x"},{"name":"a","image":"y"}]"#).expect_err("must fail");
    assert!(error.to_string().contains("more than once"));
}

#[test]
fn plan_creates_keeps_and_leaves_unmanaged_gardens() {
    let section =
        parse(r#"[{"name":"new","image":"x"},{"name":"kept","image":"x"}]"#).expect("valid");
    let existing =
        ["kept", "stray"].map(|candidate| GardenName::new(candidate).expect("valid name"));
    let plan = plan_gardens(&section, &existing);
    let kinds: Vec<&str> = plan
        .actions()
        .iter()
        .map(|action| match action {
            PlanAction::Create(_) => "create",
            PlanAction::Keep(_) => "keep",
            PlanAction::LeaveUnmanaged(_) => "leave",
        })
        .collect();
    assert_eq!(kinds, ["create", "keep", "leave"]);
}
