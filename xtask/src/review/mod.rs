mod cli;
mod diff;
mod error;
mod github;
mod holistic;
mod local;
mod mode;
mod model;
mod pipeline;
mod post;
mod prompt;
mod prompt_file;
mod text;
mod tools;
mod worker;

pub use cli::{ReviewTask, run_review_task};
pub use diff::{
    AddedLines, MAXIMUM_DIFF_BYTES, parse_added_lines, read_local_diff, reject_oversized_diff,
    split_diff_into_shards,
};
pub use error::ReviewError;
pub use github::{
    AuthorAssociation, EventName, GitHub, ProgressComment, RepositoryName, RunPermission,
    StatusState, decide_run_permission, has_reviewable_path, should_skip_review,
};
pub use holistic::{MAXIMUM_ROUNDS, run_holistic_review};
pub use mode::{ALL_REVIEWER_MODES, ReviewerMode};
pub use model::{
    ChatMessage, ChatRequest, Confidence, ModelClient, ModelEndpoint, ModelReply, ReviewAnswer,
    Role, ToolCall, Violation, parse_answer,
};
pub use pipeline::{CiOptions, StatusPrefix, endpoint_for, run_ci_review};
pub use post::{
    DropReason, DroppedFinding, PostTarget, ValidationOutcome, build_footer, build_summary_body,
    publish_findings, validate_violations,
};
pub use prompt::{
    RuleFile, build_rule_checklist, build_system_prompt, build_user_prompt,
    collect_allowed_rule_identifiers, load_reviewer_instructions, load_rule_files,
};
pub use tools::{ToolError, ToolRunner, tool_definitions};
