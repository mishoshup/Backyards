use std::collections::BTreeSet;

use crate::report::GateError;
use crate::review::diff::{AddedLines, parse_added_lines};
use crate::review::model::{ChatMessage, ChatRequest, ModelClient, ReviewAnswer, parse_answer};
use crate::review::pipeline::endpoint::endpoint_for;
use crate::review::post::{ValidationOutcome, validate_violations};
use crate::review::prompt::{
    RuleFile, allowed_rule_identifiers, build_system_prompt, build_user_prompt,
    load_reviewer_instructions, load_rule_files,
};
use crate::review::tools::ToolRunner;
use crate::review::{ReviewError, ReviewerMode, run_holistic_review};
use crate::workspace::Repository;

const REVIEWER_MODES: [ReviewerMode; 2] = [ReviewerMode::Inline, ReviewerMode::Holistic];

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

pub fn run_both_reviewers(
    context: &ReviewContext<'_>,
) -> Result<Vec<(ReviewerMode, ValidationOutcome)>, GateError> {
    std::thread::scope(|scope| {
        let workers: Vec<_> = REVIEWER_MODES
            .iter()
            .map(|mode| scope.spawn(|| run_one_reviewer(context, *mode)))
            .collect();
        REVIEWER_MODES
            .iter()
            .zip(workers)
            .map(|(mode, worker)| join_worker(worker).map(|outcome| (*mode, outcome)))
            .collect()
    })
}

fn join_worker(
    worker: std::thread::ScopedJoinHandle<'_, Result<ValidationOutcome, GateError>>,
) -> Result<ValidationOutcome, GateError> {
    worker.join().unwrap_or_else(|_| {
        Err(ReviewError::InvalidAnswer {
            reason: "reviewer thread panicked".to_owned(),
        }
        .into())
    })
}

fn run_one_reviewer(
    context: &ReviewContext<'_>,
    mode: ReviewerMode,
) -> Result<ValidationOutcome, GateError> {
    let answer = request_answer(context, mode)?;
    let allowed: BTreeSet<&str> = allowed_rule_identifiers(&context.rules);
    Ok(validate_violations(
        answer.violations,
        mode,
        &context.added,
        &allowed,
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
    let instructions = load_reviewer_instructions(context.repository, mode)?;
    let system = build_system_prompt(&instructions, mode, &context.rules);
    let user = build_user_prompt(mode, &context.diff);
    Ok(vec![
        ChatMessage::new("system", system),
        ChatMessage::new("user", user),
    ])
}
