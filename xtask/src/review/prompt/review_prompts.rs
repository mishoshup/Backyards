use crate::report::GateError;
use crate::review::ReviewerMode;
use crate::review::prompt::assemble::{build_system_prompt, build_user_prompt};
use crate::review::prompt::instructions::load_reviewer_instructions;
use crate::review::prompt::rules::RuleFile;
use crate::workspace::Repository;

pub struct ReviewPrompts {
    pub system: String,
    pub user: String,
}

pub fn build_review_prompts(
    repository: &Repository,
    mode: ReviewerMode,
    rules: &[RuleFile],
    diff: &str,
) -> Result<ReviewPrompts, GateError> {
    let instructions = load_reviewer_instructions(repository, mode)?;
    Ok(ReviewPrompts {
        system: build_system_prompt(&instructions, mode, rules),
        user: build_user_prompt(mode, diff),
    })
}
