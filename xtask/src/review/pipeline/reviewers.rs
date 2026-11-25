use crate::report::GateError;
use crate::review::diff::{AddedLines, parse_added_lines};
use crate::review::model::{
    ChatMessage, ChatRequest, ModelClient, ReviewAnswer, Role, parse_answer,
};
use crate::review::pipeline::endpoint::endpoint_for;
use crate::review::post::{ValidationOutcome, validate_answer};
use crate::review::prompt::{RuleFile, build_review_prompts, load_rule_files};
use crate::review::tools::ToolRunner;
use crate::review::worker::run_each_reviewer_mode;
use crate::review::{ReviewerMode, run_holistic_review};
use crate::workspace::Repository;

pub struct ReviewContext<'context> {
    pub repository: &'context Repository,
    pub diff: String,
    pub added: AddedLines,
    pub rules: Vec<RuleFile>,
}

pub fn load_review_context(
    repository: &Repository,
    diff: String,
) -> Result<ReviewContext<'_>, GateError> {
    let added = parse_added_lines(&diff)?;
    let rules = load_rule_files(repository)?;
    Ok(ReviewContext {
        repository,
        diff,
        added,
        rules,
    })
}

pub use crate::review::worker::ReviewerResult;

pub fn run_both_reviewers(context: &ReviewContext<'_>) -> Vec<ReviewerResult> {
    run_each_reviewer_mode(|mode| run_one_reviewer(context, mode))
}

fn run_one_reviewer(
    context: &ReviewContext<'_>,
    mode: ReviewerMode,
) -> Result<ValidationOutcome, GateError> {
    let answer = request_answer(context, mode)?;
    Ok(validate_answer(
        answer,
        mode,
        &context.added,
        &context.rules,
    ))
}

fn request_answer(
    context: &ReviewContext<'_>,
    mode: ReviewerMode,
) -> Result<ReviewAnswer, GateError> {
    let client = ModelClient::new(endpoint_for(mode)?);
    let messages = build_messages(context, mode)?;
    match mode {
        ReviewerMode::Inline => ask_once(&client, &messages),
        ReviewerMode::Holistic => Ok(run_holistic_review(
            &client,
            ToolRunner::new(context.repository.root()),
            messages,
            &mut std::io::stderr(),
        )?),
    }
}

fn ask_once(client: &ModelClient, messages: &[ChatMessage]) -> Result<ReviewAnswer, GateError> {
    let request = ChatRequest {
        messages,
        tools: None,
    };
    let reply = client.complete(request, &mut std::io::stderr())?;
    Ok(parse_answer(&reply.content)?)
}

fn build_messages(
    context: &ReviewContext<'_>,
    mode: ReviewerMode,
) -> Result<Vec<ChatMessage>, GateError> {
    let prompts = build_review_prompts(context.repository, mode, &context.rules, &context.diff)?;
    Ok(vec![
        ChatMessage::new(Role::System, prompts.system),
        ChatMessage::new(Role::User, prompts.user),
    ])
}
