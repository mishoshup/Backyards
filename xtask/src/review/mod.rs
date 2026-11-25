mod cli;
mod diff;
mod error;
mod holistic;
mod local;
mod mode;
mod model;
mod post;
mod prompt;
mod tools;

pub use cli::{ReviewTask, run_review_task};
pub use diff::{
    AddedLines, MAXIMUM_DIFF_BYTES, parse_added_lines, read_local_diff, reject_oversized_diff,
};
pub use error::ReviewError;
pub use holistic::{MAXIMUM_ROUNDS, run_holistic_review};
pub use mode::ReviewerMode;
pub use model::{
    ChatMessage, ChatRequest, Confidence, ModelClient, ModelEndpoint, ModelReply, ReviewAnswer,
    ToolCall, Violation, parse_answer,
};
pub use post::{DropReason, DroppedFinding, ValidationOutcome, validate_violations};
pub use prompt::{
    RuleFile, allowed_rule_identifiers, build_rule_checklist, build_system_prompt,
    build_user_prompt, load_reviewer_instructions, load_rule_files,
};
pub use tools::{ToolError, ToolRunner, tool_definitions};
