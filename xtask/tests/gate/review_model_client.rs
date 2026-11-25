use std::time::Duration;

use crate::fake_model_server::{
    FakeModelServer, Script, content_event, finish_event, reasoning_event,
};
use xtask::review::{ChatMessage, ModelClient, ModelEndpoint, ReviewError, parse_answer};

fn client_for(server: &FakeModelServer) -> ModelClient {
    let mut endpoint = ModelEndpoint::new(server.url.clone(), "key".to_owned(), "model".to_owned());
    endpoint.retry_delay = Duration::from_millis(10);
    endpoint.idle_timeout = Duration::from_millis(400);
    ModelClient::new(endpoint)
}

fn ask(client: &ModelClient) -> Result<xtask::review::ModelReply, ReviewError> {
    let messages = [ChatMessage::new("user", "review".to_owned())];
    client.complete(&messages, &mut Vec::new())
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
    let reply = ask(&client_for(&server)).unwrap();
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
    assert!(ask(&client_for(&server)).is_ok());
    assert_eq!(server.finish(), 2);
}

#[test]
fn client_error_is_not_retried() {
    let server = FakeModelServer::start(vec![Script::Status(400)]);
    let error = ask(&client_for(&server)).unwrap_err();
    assert!(matches!(error, ReviewError::ModelStatus { status: 400 }));
    assert_eq!(server.finish(), 1);
}

#[test]
fn second_server_error_is_reported() {
    let server = FakeModelServer::start(vec![Script::Status(503), Script::Status(503)]);
    let error = ask(&client_for(&server)).unwrap_err();
    assert!(matches!(error, ReviewError::ModelStatus { status: 503 }));
    server.finish();
}

#[test]
fn idle_stall_times_out_and_retries() {
    let server = FakeModelServer::start(vec![
        Script::StallAfterHeaders,
        Script::Respond(answer_stream("{\"violations\":[]}")),
    ]);
    let result = ask(&client_for(&server));
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
    let error = ask(&client_for(&server)).unwrap_err();
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
fn low_confidence_finding_parses() {
    let answer = parse_answer("{\"violations\":[{\"ruleId\":\"A-1\",\"path\":\"a.rs\",\"line\":3,\"body\":\"A-1 x\",\"confidence\":\"low\"}]}").unwrap();
    assert_eq!(answer.violations[0].line, 3);
}
