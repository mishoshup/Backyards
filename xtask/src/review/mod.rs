mod diff;
mod error;
mod mode;
mod model;
mod prompt;

pub use diff::{AddedLines, MAXIMUM_DIFF_BYTES, parse_added_lines, reject_oversized_diff};
pub use error::ReviewError;
pub use mode::ReviewerMode;
pub use model::{
    ChatMessage, Confidence, ModelClient, ModelEndpoint, ModelReply, ReviewAnswer, Violation,
    parse_answer,
};
pub use prompt::{
    RuleFile, allowed_rule_identifiers, build_rule_checklist, build_system_prompt,
    build_user_prompt, load_reviewer_instructions, load_rule_files,
};
