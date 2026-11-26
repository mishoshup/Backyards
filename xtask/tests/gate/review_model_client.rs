use std::time::Duration;

use crate::fake_model_server::{
    FakeModelServer, Script, client_for, content_event, finish_event, reasoning_event,
    tool_call_event,
};
use xtask::review::{ChatMessage, ChatRequest, ModelClient, ReviewError, Role, parse_answer};

fn ask(client: &ModelClient) -> Result<xtask::review::ModelReply, ReviewError> {
    let messages = [ChatMessage::new(Role::User, "review".to_owned())];
    let request = ChatRequest {
        messages: &messages,
        tools: None,
    };
    client.complete(request, &mut Vec::new())
}

fn answer_stream(answer: &str) -> String {
    format!(
        "{}{}{}{}",
        reasoning_event(),
        reasoning_event(),
        content_event(answer),
        finish_event("stop")
    )
}

#[test]
fn reasoning_chunks_then_answer_is_collected() {
    let server =
        FakeModelServer::start(vec![Script::Respond(answer_stream("{\"violations\":[]}"))]);
    let reply = ask(&client_for(&server, Some(Duration::from_millis(400)))).unwrap();
    assert_eq!(reply.content, "{\"violations\":[]}");
    assert_eq!(reply.reasoning_chunks, 2);
    assert_eq!(parse_answer(&reply.content).unwrap().violations, []);
    server.finish();
}

#[test]
fn server_error_is_retried_once_then_succeeds() {
    let server = FakeModelServer::start(vec![
        Script::Status(500),
        Script::Respond(answer_stream("{\"violations\":[]}")),
    ]);
    assert!(ask(&client_for(&server, Some(Duration::from_millis(400)))).is_ok());
    assert_eq!(server.finish(), 2);
}

#[test]
fn client_error_is_not_retried() {
    let server = FakeModelServer::start(vec![Script::Status(400)]);
    let error = ask(&client_for(&server, Some(Duration::from_millis(400)))).unwrap_err();
    assert!(matches!(
        error,
        ReviewError::ModelStatus { status: 400, .. }
    ));
    assert_eq!(server.finish(), 1);
}

#[test]
fn second_server_error_is_reported() {
    let server = FakeModelServer::start(vec![Script::Status(503), Script::Status(503)]);
    let error = ask(&client_for(&server, Some(Duration::from_millis(400)))).unwrap_err();
    assert!(matches!(
        error,
        ReviewError::ModelStatus { status: 503, .. }
    ));
    server.finish();
}

#[test]
fn idle_stall_times_out_and_retries() {
    let server = FakeModelServer::start(vec![
        Script::StallAfterHeaders,
        Script::Respond(answer_stream("{\"violations\":[]}")),
    ]);
    let result = ask(&client_for(&server, Some(Duration::from_millis(400))));
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(server.finish(), 2);
}

#[test]
fn truncated_answer_is_a_failure() {
    let server = FakeModelServer::start(vec![Script::Respond(format!(
        "{}{}",
        content_event("{\"viol"),
        finish_event("length")
    ))]);
    let error = ask(&client_for(&server, Some(Duration::from_millis(400)))).unwrap_err();
    assert!(matches!(error, ReviewError::ModelTruncated { .. }));
    server.finish();
}

#[test]
fn stream_closed_without_finish_reason_is_a_failure() {
    let server =
        FakeModelServer::start(vec![Script::Respond(content_event("{\"violations\":[]}"))]);
    let error = ask(&client_for(&server, Some(Duration::from_millis(400)))).unwrap_err();
    assert!(matches!(error, ReviewError::ModelTruncated { .. }));
    server.finish();
}

#[test]
fn empty_and_invalid_answers_are_failures() {
    assert!(matches!(parse_answer("  "), Err(ReviewError::EmptyAnswer)));
    assert!(matches!(
        parse_answer("not json"),
        Err(ReviewError::InvalidAnswer { .. })
    ));
    assert!(matches!(
        parse_answer("{\"violations\":[{\"ruleId\":\"A-1\"}]}"),
        Err(ReviewError::InvalidAnswer { .. })
    ));
}

#[test]
fn parse_answer_ignores_prose_after_the_json_object() {
    let answer = parse_answer(r#"{"violations":[]} Note: I considered {path:line} but declined"#);
    assert_eq!(answer.unwrap().violations, []);
}

#[test]
fn low_confidence_finding_parses() {
    let answer = parse_answer("{\"violations\":[{\"ruleId\":\"A-1\",\"path\":\"a.rs\",\"line\":3,\"body\":\"A-1 x\",\"confidence\":\"low\"}]}").unwrap();
    assert_eq!(answer.violations[0].line, 3);
}

#[test]
fn oversized_tool_call_index_is_rejected_without_allocation() {
    let server = FakeModelServer::start(vec![Script::Respond(tool_call_event(
        1_000_000_000,
        "call",
        "grep",
        "{}",
    ))]);
    let error = ask(&client_for(&server, Some(Duration::from_millis(400)))).unwrap_err();
    assert!(matches!(error, ReviewError::MalformedStreamEvent { .. }));
    server.finish();
}

#[test]
fn answer_wrapped_in_prose_is_extracted() {
    let answer = parse_answer("Here is the review: {\"violations\":[]}").unwrap();
    assert_eq!(answer.violations, vec![]);
}

#[test]
fn answer_after_brace_in_prose_is_parsed() {
    let reply = "Review {draft}: {\"violations\":[]}";
    assert_eq!(parse_answer(reply).unwrap().violations, []);
}
