use std::io::{BufRead, BufReader, Write};
use std::time::Instant;

use serde::Serialize;
use serde_json::json;

use crate::review::error::ReviewError;
use crate::review::model::endpoint::{
    CONNECT_TIMEOUT, ModelEndpoint, PROGRESS_INTERVAL, TOTAL_TIMEOUT,
};
use crate::review::model::retry::run_with_retry;
use crate::review::model::stream::{StreamEvent, parse_stream_line};

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    #[must_use]
    pub fn new(role: &str, content: String) -> Self {
        Self {
            role: role.to_owned(),
            content,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelReply {
    pub content: String,
    pub reasoning_chunks: usize,
}

#[derive(Debug)]
pub struct ModelClient {
    endpoint: ModelEndpoint,
    agent: ureq::Agent,
}

impl ModelClient {
    #[must_use]
    pub fn new(endpoint: ModelEndpoint) -> Self {
        let agent = build_agent(&endpoint);
        Self { endpoint, agent }
    }

    pub fn complete(
        &self,
        messages: &[ChatMessage],
        progress: &mut dyn Write,
    ) -> Result<ModelReply, ReviewError> {
        run_with_retry(self.endpoint.retry_delay, || {
            self.attempt(messages, progress)
        })
    }

    fn attempt(
        &self,
        messages: &[ChatMessage],
        progress: &mut dyn Write,
    ) -> Result<ModelReply, ReviewError> {
        let response = self.send_request(messages)?;
        let reader = BufReader::new(response.into_body().into_reader());
        collect_reply(reader, progress)
    }

    fn send_request(
        &self,
        messages: &[ChatMessage],
    ) -> Result<ureq::http::Response<ureq::Body>, ReviewError> {
        self.agent
            .post(&self.endpoint.url)
            .header(
                "Authorization",
                &format!("Bearer {}", self.endpoint.api_key),
            )
            .send_json(build_request_body(&self.endpoint.model, messages))
            .map_err(|error| classify_transport_error(&error))
    }
}

fn build_agent(endpoint: &ModelEndpoint) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_per_read(Some(endpoint.idle_timeout))
        .timeout_global(Some(TOTAL_TIMEOUT))
        .build()
        .into()
}

fn build_request_body(model: &str, messages: &[ChatMessage]) -> serde_json::Value {
    json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "response_format": { "type": "json_object" },
        "reasoning_effort": "high",
        "thinking": { "type": "enabled" },
    })
}

fn classify_transport_error(error: &ureq::Error) -> ReviewError {
    if let ureq::Error::StatusCode(status) = error {
        return ReviewError::ModelStatus { status: *status };
    }
    if let ureq::Error::Timeout(timeout) = error {
        return ReviewError::ModelTimeout {
            reason: timeout.to_string(),
        };
    }
    ReviewError::ModelConnection {
        reason: error.to_string(),
    }
}

fn classify_read_error(error: &std::io::Error) -> ReviewError {
    if error.kind() == std::io::ErrorKind::TimedOut {
        ReviewError::ModelTimeout {
            reason: error.to_string(),
        }
    } else {
        ReviewError::ModelConnection {
            reason: error.to_string(),
        }
    }
}

struct StreamState {
    content: String,
    reasoning_chunks: usize,
    finish_reason: Option<String>,
    started: Instant,
    last_report: Instant,
}

impl StreamState {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            content: String::new(),
            reasoning_chunks: 0,
            finish_reason: None,
            started: now,
            last_report: now,
        }
    }

    fn apply(&mut self, event: StreamEvent) -> bool {
        match event {
            StreamEvent::Content(text) => self.content.push_str(&text),
            StreamEvent::Reasoning => self.reasoning_chunks += 1,
            StreamEvent::Finished(reason) => self.finish_reason = Some(reason),
            StreamEvent::Done => return true,
            StreamEvent::Ignored => {}
        }
        false
    }

    fn report_if_due(&mut self, progress: &mut dyn Write) {
        if self.last_report.elapsed() < PROGRESS_INTERVAL {
            return;
        }
        self.last_report = Instant::now();
        writeln!(
            progress,
            "model working: {}s elapsed, {} reasoning chunks, {} answer bytes",
            self.started.elapsed().as_secs(),
            self.reasoning_chunks,
            self.content.len()
        )
        .ok();
    }

    fn into_reply(self) -> Result<ModelReply, ReviewError> {
        match self.finish_reason.as_deref() {
            Some("stop") | None => Ok(ModelReply {
                content: self.content,
                reasoning_chunks: self.reasoning_chunks,
            }),
            Some(other) => Err(ReviewError::ModelTruncated {
                finish_reason: other.to_owned(),
            }),
        }
    }
}

fn collect_reply(
    reader: impl BufRead,
    progress: &mut dyn Write,
) -> Result<ModelReply, ReviewError> {
    let mut state = StreamState::new();
    for line in reader.lines() {
        let line = line.map_err(|error| classify_read_error(&error))?;
        if state.apply(parse_stream_line(&line)?) {
            break;
        }
        state.report_if_due(progress);
    }
    state.into_reply()
}
