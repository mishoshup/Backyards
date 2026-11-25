use crate::fixture::Fixture;
use xtask::review::{
    ReviewerMode, allowed_rule_identifiers, build_rule_checklist, build_system_prompt,
    build_user_prompt, load_reviewer_instructions, load_rule_files,
};

const RULES_DIRECTORY: &str = ".agents/skills/rust-coding-standards/rules";

fn rule_text(identifier: &str, heading: &str) -> String {
    format!(
        "---\nruleId: {identifier}\nenforcedBy: review\n---\n# {heading}\nBody of {identifier}.\n"
    )
}

fn fixture_with_rules() -> Fixture {
    let fixture = Fixture::create();
    fixture.write(
        &format!("{RULES_DIRECTORY}/perf-01-borrow.md"),
        &rule_text("PERF-01", "Borrow, do not clone"),
    );
    fixture.write(
        &format!("{RULES_DIRECTORY}/naming-02-boolean.md"),
        &rule_text("NAMING-02", "Prefix booleans"),
    );
    fixture.write(
        &format!("{RULES_DIRECTORY}/_template.md"),
        "---\nruleId: not a rule\n---\n",
    );
    fixture.write(
        ".claude/agents/code-inline-reviewer.md",
        "---\nname: inline\n---\n\n# Inline reviewer\nCheck every hunk.\n",
    );
    fixture
}

#[test]
fn rules_load_in_file_order_and_skip_internal_files() {
    let fixture = fixture_with_rules();
    let rules = load_rule_files(&fixture.repository()).unwrap();
    let identifiers: Vec<&str> = rules.iter().map(|rule| rule.identifier.as_str()).collect();
    assert_eq!(identifiers, ["NAMING-02", "PERF-01"]);
}

#[test]
fn checklist_lists_identifier_heading_and_file() {
    let fixture = fixture_with_rules();
    let rules = load_rule_files(&fixture.repository()).unwrap();
    assert_eq!(
        build_rule_checklist(&rules),
        "- NAMING-02: Prefix booleans (naming-02-boolean.md)\n- PERF-01: Borrow, do not clone (perf-01-borrow.md)"
    );
}

#[test]
fn allowed_identifiers_match_the_loaded_rules() {
    let fixture = fixture_with_rules();
    let rules = load_rule_files(&fixture.repository()).unwrap();
    let allowed = allowed_rule_identifiers(&rules);
    assert!(allowed.contains("PERF-01"));
    assert!(!allowed.contains("PERF-99"));
}

#[test]
fn rule_without_a_valid_identifier_is_rejected() {
    let fixture = fixture_with_rules();
    fixture.write(
        &format!("{RULES_DIRECTORY}/bad.md"),
        "---\nruleId: lowercase-1\n---\n# Bad\n",
    );
    let error = load_rule_files(&fixture.repository()).unwrap_err();
    assert!(error.to_string().contains("no valid ruleId"));
}

#[test]
fn missing_rules_directory_is_reported() {
    let fixture = Fixture::create();
    let error = load_rule_files(&fixture.repository()).unwrap_err();
    assert!(error.to_string().contains("contains no rule files"));
}

#[test]
fn inline_prompt_carries_instructions_rules_and_contract() {
    let fixture = fixture_with_rules();
    let repository = fixture.repository();
    let instructions = load_reviewer_instructions(&repository, ReviewerMode::Inline).unwrap();
    assert!(!instructions.contains("name: inline"));
    let rules = load_rule_files(&repository).unwrap();
    let prompt = build_system_prompt(&instructions, ReviewerMode::Inline, &rules);
    assert!(prompt.contains("Check every hunk."));
    assert!(prompt.contains("Rule checklist:\n- NAMING-02"));
    assert!(prompt.contains("Body of PERF-01."));
    assert!(prompt.contains("\"confidence\""));
}

#[test]
fn holistic_prompt_omits_rule_files() {
    let fixture = fixture_with_rules();
    let rules = load_rule_files(&fixture.repository()).unwrap();
    let prompt = build_system_prompt("Look at design.", ReviewerMode::Holistic, &rules);
    assert!(!prompt.contains("Rule files:"));
    assert!(prompt.contains("Look at design."));
}

#[test]
fn user_prompt_puts_the_request_before_the_diff() {
    let prompt = build_user_prompt(ReviewerMode::Inline, "diff body");
    assert!(prompt.starts_with("Review this diff against every rule."));
    assert!(prompt.ends_with("diff body"));
}
