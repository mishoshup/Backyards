use std::time::Duration;

use crate::fake_model_server::{
    FakeModelServer, Script, content_event, finish_event, reasoning_event, tool_call_event,
};
use crate::fixture::Fixture;
use xtask::review::{
    ChatMessage, MAXIMUM_ROUNDS, ModelClient, ModelEndpoint, ReviewError, ToolRunner,
    run_holistic_review,
};

fn client_for(server: &FakeModelServer) -> ModelClient {
    let mut endpoint = ModelEndpoint::new(server.url.clone(), "key".to_owned(), "model".to_owned());
    endpoint.retry_delay = Duration::from_millis(10);
    ModelClient::new(endpoint)
}

fn final_answer(answer: &str) -> Script {
    Script::Respond(format!(
        "{}{}{}",
        reasoning_event(),
        content_event(answer),
        finish_event("stop")
    ))
}

fn tool_round(identifier: &str, name: &str, arguments: &str) -> Script {
    Script::Respond(format!(
        "{}{}",
        reasoning_event(),
        tool_call_event(0, identifier, name, arguments)
    ))
}

fn run(
    server: &FakeModelServer,
    fixture: &Fixture,
) -> Result<xtask::review::ReviewAnswer, ReviewError> {
    let messages = vec![ChatMessage::new("user", "review".to_owned())];
    run_holistic_review(
        &client_for(server),
        ToolRunner::new(fixture.path()),
        messages,
        &mut Vec::new(),
    )
}

#[test]
fn tool_result_and_reasoning_are_sent_back_before_the_final_answer() {
    let fixture = Fixture::create();
    fixture.write("notes.md", "first line\n");
    let server = FakeModelServer::start(vec![
        tool_round("call_1", "read_file", r#"{"path":"notes.md"}"#),
        final_answer(r#"{"violations":[]}"#),
    ]);
    let answer = run(&server, &fixture).unwrap();
    assert_eq!(answer.violations.len(), 0);
    let bodies = server.request_bodies();
    let second: serde_json::Value = serde_json::from_str(&bodies[1]).unwrap();
    let messages = second["messages"].as_array().unwrap();
    let assistant = &messages[1];
    assert_eq!(assistant["reasoning_content"], "thinking");
    assert_eq!(assistant["tool_calls"][0]["id"], "call_1");
    let tool = &messages[2];
    assert_eq!(tool["tool_call_id"], "call_1");
    assert_eq!(tool["content"], "1\tfirst line");
    assert!(bodies[0].contains("\"tools\""));
    server.finish();
}

#[test]
fn final_round_disables_tools_and_announces_the_cap() {
    let fixture = Fixture::create();
    let mut scripts: Vec<Script> = (1..MAXIMUM_ROUNDS)
        .map(|round| tool_round(&format!("call_{round}"), "list_files", r#"{"glob":"*"}"#))
        .collect();
    scripts.push(final_answer(r#"{"violations":[]}"#));
    let server = FakeModelServer::start(scripts);
    run(&server, &fixture).unwrap();
    let bodies = server.request_bodies();
    let last = bodies.last().unwrap();
    assert!(!last.contains("\"tools\""), "{last}");
    assert!(last.contains("Tools are disabled. Answer in JSON now."));
    assert!(bodies[0].contains("\"tools\""));
    server.finish();
}

#[test]
fn tool_request_in_the_final_round_is_a_failure() {
    let fixture = Fixture::create();
    let scripts: Vec<Script> = (1..=MAXIMUM_ROUNDS)
        .map(|round| tool_round(&format!("call_{round}"), "list_files", r#"{"glob":"*"}"#))
        .collect();
    let server = FakeModelServer::start(scripts);
    assert!(matches!(
        run(&server, &fixture),
        Err(ReviewError::InvalidAnswer { .. })
    ));
    server.finish();
}

#[test]
fn unknown_tool_is_reported_to_the_model_not_dropped() {
    let fixture = Fixture::create();
    let server = FakeModelServer::start(vec![
        tool_round("call_1", "delete_everything", "{}"),
        final_answer(r#"{"violations":[]}"#),
    ]);
    run(&server, &fixture).unwrap();
    assert!(server.request_bodies()[1].contains("error: unknown tool delete_everything"));
    server.finish();
}
