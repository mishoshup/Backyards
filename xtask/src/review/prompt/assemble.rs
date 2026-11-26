use crate::review::ReviewerMode;
use crate::review::prompt::contract::OUTPUT_CONTRACT;
use crate::review::prompt::rules::{RuleFile, build_rule_checklist};

#[must_use]
pub fn build_system_prompt(instructions: &str, mode: ReviewerMode, rules: &[RuleFile]) -> String {
    let mut sections = vec![instructions.trim_end().to_owned()];
    if mode.should_include_rule_files() {
        sections.push(render_rule_section(rules));
    }
    sections.push(OUTPUT_CONTRACT.to_owned());
    sections.join("\n\n")
}

#[must_use]
pub fn build_user_prompt(mode: ReviewerMode, diff: &str) -> String {
    format!("{}\n\n{diff}", mode.user_request())
}

fn render_rule_section(rules: &[RuleFile]) -> String {
    let bodies = rules
        .iter()
        .map(render_rule_file)
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "Rule checklist:\n{}\n\nRule files:\n\n{bodies}",
        build_rule_checklist(rules)
    )
}

fn render_rule_file(rule: &RuleFile) -> String {
    format!("### {}\n{}", rule.file_name, rule.content.trim_end())
}
