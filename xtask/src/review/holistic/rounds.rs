use std::io::Write;

use crate::review::error::ReviewError;
use crate::review::model::{
    ChatMessage, ChatRequest, ModelClient, ModelReply, ReviewAnswer, Role, parse_answer,
};
use crate::review::tools::{ToolRunner, tool_definitions};

pub const MAXIMUM_ROUNDS: usize = 8;
const ANNOUNCEMENT_ROUNDS: usize = 2;

pub fn run_holistic_review(
    client: &ModelClient,
    runner: ToolRunner<'_>,
    mut messages: Vec<ChatMessage>,
    progress: &mut dyn Write,
) -> Result<ReviewAnswer, ReviewError> {
    let tools = tool_definitions();
    for round in 1..=MAXIMUM_ROUNDS {
        announce_cap_if_near(&mut messages, round);
        let request = ChatRequest {
            messages: &messages,
            tools: (round < MAXIMUM_ROUNDS).then_some(&tools),
        };
        let reply = client.complete(request, progress)?;
        if reply.tool_calls.is_empty() {
            match parse_answer(&reply.content) {
                Ok(answer) => return Ok(answer),
                Err(error) if round < MAXIMUM_ROUNDS => {
                    request_json_only(&mut messages, reply, &error);
                }
                Err(error) => return Err(error),
            }
            continue;
        }
        reject_tools_in_final_round(round)?;
        answer_tool_calls(runner, &mut messages, reply);
    }
    Err(round_limit_error())
}

fn request_json_only(messages: &mut Vec<ChatMessage>, reply: ModelReply, error: &ReviewError) {
    messages.push(ChatMessage::assistant_answer(
        reply.content,
        reply.reasoning,
    ));
    messages.push(ChatMessage::new(
        Role::User,
        format!("The answer was rejected: {error}. Respond with the JSON object only."),
    ));
}

fn announce_cap_if_near(messages: &mut Vec<ChatMessage>, round: usize) {
    if round + ANNOUNCEMENT_ROUNDS <= MAXIMUM_ROUNDS {
        return;
    }
    let notice = if round == MAXIMUM_ROUNDS {
        format!(
            "This is round {round} of {MAXIMUM_ROUNDS}. Tools are disabled. Answer in JSON now."
        )
    } else {
        format!(
            "This is round {round} of {MAXIMUM_ROUNDS}. Tools are disabled in the final round, so finish your checks."
        )
    };
    messages.push(ChatMessage::new(Role::User, notice));
}

fn reject_tools_in_final_round(round: usize) -> Result<(), ReviewError> {
    if round == MAXIMUM_ROUNDS {
        Err(round_limit_error())
    } else {
        Ok(())
    }
}

fn round_limit_error() -> ReviewError {
    ReviewError::InvalidAnswer {
        reason: format!("no final answer within {MAXIMUM_ROUNDS} rounds"),
    }
}

fn answer_tool_calls(runner: ToolRunner<'_>, messages: &mut Vec<ChatMessage>, reply: ModelReply) {
    let results: Vec<ChatMessage> = reply
        .tool_calls
        .iter()
        .map(|call| {
            ChatMessage::tool_result(
                call.id.clone(),
                runner.run(&call.function.name, &call.function.arguments),
            )
        })
        .collect();
    messages.push(ChatMessage::assistant_with_calls(
        reply.content,
        reply.reasoning,
        reply.tool_calls,
    ));
    messages.extend(results);
}
