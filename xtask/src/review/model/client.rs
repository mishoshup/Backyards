use std::io::{BufRead, BufReader, Write};
use std::time::Instant;

use serde_json::{Value, json};

use crate::report::pluralize;

const MAXIMUM_TOOL_CALLS_PER_REPLY: usize = 32;
const MAXIMUM_ANSWER_BYTES: usize = 4_000_000;
const MAXIMUM_REASONING_BYTES: usize = 4_000_000;
const MAXIMUM_TOOL_ARGUMENT_BYTES: usize = 1_000_000;
const MAXIMUM_TOOL_NAME_BYTES: usize = 256;
const MAXIMUM_ERROR_DETAIL_BYTES: u64 = 2_000;

use crate::review::error::ReviewError;
use crate::review::model::endpoint::{
    CONNECT_TIMEOUT, ModelEndpoint, PROGRESS_INTERVAL, TOTAL_TIMEOUT,
};
use crate::review::model::message::{ChatMessage, FunctionCall, ToolCall, ToolCallKind};
use crate::review::model::retry::run_with_retry;
use crate::review::model::stream::{
    FinishReason, StreamEvent, ToolCallFragment, parse_stream_line,
};

const EMPTY_ARGUMENTS: &str = "{}";

#[derive(Debug, Clone, Copy)]
pub struct ChatRequest<'request> {
    pub messages: &'request [ChatMessage],
    pub tools: Option<&'request Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelReply {
    pub content: String,
    pub reasoning: String,
    pub reasoning_chunks: usize,
    pub tool_calls: Vec<ToolCall>,
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
        request: ChatRequest<'_>,
        progress: &mut dyn Write,
    ) -> Result<ModelReply, ReviewError> {
        run_with_retry(self.endpoint.retry_delay, || {
            self.attempt(request, progress)
        })
    }

    fn attempt(
        &self,
        request: ChatRequest<'_>,
        progress: &mut dyn Write,
    ) -> Result<ModelReply, ReviewError> {
        let response = self.send_request(request)?;
        let reader = BufReader::new(response.into_body().into_reader());
        collect_reply(reader, progress)
    }

    fn send_request(
        &self,
        request: ChatRequest<'_>,
    ) -> Result<ureq::http::Response<ureq::Body>, ReviewError> {
        let response = self
            .agent
            .post(&self.endpoint.url)
            .header(
                "Authorization",
                &format!("Bearer {}", self.endpoint.api_key.expose()),
            )
            .send_json(build_request_body(&self.endpoint.model, request))
            .map_err(|error| classify_transport_error(&error))?;
        reject_failure_status(response)
    }
}

fn reject_failure_status(
    response: ureq::http::Response<ureq::Body>,
) -> Result<ureq::http::Response<ureq::Body>, ReviewError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let detail = response
        .into_body()
        .with_config()
        .limit(MAXIMUM_ERROR_DETAIL_BYTES)
        .read_to_string()
        .map_or_else(
            |failure| format!("response body unreadable: {failure}"),
            |body| body.trim().to_owned(),
        );
    Err(ReviewError::ModelStatus {
        status: status.as_u16(),
        detail,
    })
}

fn build_agent(endpoint: &ModelEndpoint) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_per_read(Some(endpoint.idle_timeout))
        .timeout_global(Some(TOTAL_TIMEOUT))
        .http_status_as_error(false)
        .build()
        .into()
}

fn build_request_body(model: &str, request: ChatRequest<'_>) -> Value {
    let mut body = json!({
        "model": model,
        "messages": request.messages,
        "stream": true,
        "reasoning_effort": "high",
        "thinking": { "type": "enabled" },
    });
    match request.tools {
        Some(tools) => body["tools"] = tools.clone(),
        None => body["response_format"] = json!({ "type": "json_object" }),
    }
    body
}

fn classify_transport_error(error: &ureq::Error) -> ReviewError {
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
    reasoning: String,
    reasoning_chunks: usize,
    tool_calls: Vec<ToolCall>,
    finish_reason: Option<FinishReason>,
    is_complete: bool,
    started: Instant,
    last_report: Instant,
}

impl StreamState {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            content: String::new(),
            reasoning: String::new(),
            reasoning_chunks: 0,
            tool_calls: Vec::new(),
            finish_reason: None,
            is_complete: false,
            started: now,
            last_report: now,
        }
    }

    fn apply_all(&mut self, events: Vec<StreamEvent>) -> Result<bool, ReviewError> {
        for event in events {
            if self.apply(event)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn apply(&mut self, event: StreamEvent) -> Result<bool, ReviewError> {
        match event {
            StreamEvent::Content(text) => self.record_content(&text)?,
            StreamEvent::Reasoning(text) => self.record_reasoning(&text)?,
            StreamEvent::ToolCalls(fragments) => self.merge_tool_calls(fragments)?,
            StreamEvent::Finished(reason) => {
                self.finish_reason = Some(reason);
                self.is_complete = true;
            }
            StreamEvent::Done => {
                self.is_complete = true;
                return Ok(true);
            }
            StreamEvent::Ignored => {}
        }
        Ok(false)
    }

    fn record_content(&mut self, text: &str) -> Result<(), ReviewError> {
        reject_over_limit(
            "answer",
            self.content.len() + text.len(),
            MAXIMUM_ANSWER_BYTES,
        )?;
        self.content.push_str(text);
        Ok(())
    }

    fn record_reasoning(&mut self, text: &str) -> Result<(), ReviewError> {
        reject_over_limit(
            "reasoning",
            self.reasoning.len() + text.len(),
            MAXIMUM_REASONING_BYTES,
        )?;
        self.reasoning.push_str(text);
        self.reasoning_chunks += 1;
        Ok(())
    }

    fn merge_tool_calls(&mut self, fragments: Vec<ToolCallFragment>) -> Result<(), ReviewError> {
        for fragment in fragments {
            reject_unbounded_index(fragment.index)?;
            self.merge_tool_call(fragment)?;
        }
        Ok(())
    }

    fn merge_tool_call(&mut self, fragment: ToolCallFragment) -> Result<(), ReviewError> {
        while self.tool_calls.len() <= fragment.index {
            self.tool_calls.push(empty_tool_call());
        }
        let call = &mut self.tool_calls[fragment.index];
        if let Some(identifier) = fragment.identifier {
            reject_over_limit("tool call id", identifier.len(), MAXIMUM_TOOL_NAME_BYTES)?;
            call.id = identifier;
        }
        if let Some(name) = fragment.name {
            reject_over_limit(
                "tool call name",
                call.function.name.len() + name.len(),
                MAXIMUM_TOOL_NAME_BYTES,
            )?;
            call.function.name.push_str(&name);
        }
        if let Some(arguments) = fragment.arguments {
            reject_over_limit(
                "tool call arguments",
                call.function.arguments.len() + arguments.len(),
                MAXIMUM_TOOL_ARGUMENT_BYTES,
            )?;
            call.function.arguments.push_str(&arguments);
        }
        Ok(())
    }

    fn report_if_due(&mut self, progress: &mut dyn Write) -> Result<(), ReviewError> {
        if self.last_report.elapsed() < PROGRESS_INTERVAL {
            return Ok(());
        }
        self.last_report = Instant::now();
        writeln!(
            progress,
            "model working: {}s elapsed, {}, {}",
            self.started.elapsed().as_secs(),
            pluralize(self.reasoning_chunks, "reasoning chunk", "reasoning chunks"),
            pluralize(self.content.len(), "answer byte", "answer bytes")
        )
        .map_err(|source| ReviewError::ProgressWrite {
            reason: source.to_string(),
        })
    }

    fn into_reply(mut self) -> Result<ModelReply, ReviewError> {
        match self.finish_reason.take() {
            Some(FinishReason::Other(other)) => {
                return Err(ReviewError::ModelTruncated {
                    finish_reason: other,
                });
            }
            None => {
                return Err(ReviewError::ModelTruncated {
                    finish_reason: "stream ended without a finish reason".to_owned(),
                });
            }
            Some(_) => {}
        }
        for call in &mut self.tool_calls {
            if call.function.arguments.trim().is_empty() {
                EMPTY_ARGUMENTS.clone_into(&mut call.function.arguments);
            }
        }
        Ok(ModelReply {
            content: self.content,
            reasoning: self.reasoning,
            reasoning_chunks: self.reasoning_chunks,
            tool_calls: self.tool_calls,
        })
    }
}

const fn reject_over_limit(
    part: &'static str,
    projected_bytes: usize,
    limit: usize,
) -> Result<(), ReviewError> {
    if projected_bytes > limit {
        return Err(ReviewError::ModelOutputTooLarge { part, limit });
    }
    Ok(())
}

fn reject_unbounded_index(index: usize) -> Result<(), ReviewError> {
    if index < MAXIMUM_TOOL_CALLS_PER_REPLY {
        return Ok(());
    }
    Err(ReviewError::MalformedStreamEvent {
        reason: format!("tool call index {index} exceeds {MAXIMUM_TOOL_CALLS_PER_REPLY}"),
    })
}

const fn empty_tool_call() -> ToolCall {
    ToolCall {
        id: String::new(),
        kind: ToolCallKind::Function,
        function: FunctionCall {
            name: String::new(),
            arguments: String::new(),
        },
    }
}

fn collect_reply(
    reader: impl BufRead,
    progress: &mut dyn Write,
) -> Result<ModelReply, ReviewError> {
    let mut state = StreamState::new();
    for line in reader.lines() {
        let line = line.map_err(|error| classify_read_error(&error))?;
        if state.apply_all(parse_stream_line(&line)?)? {
            break;
        }
        state.report_if_due(progress)?;
    }
    state.into_reply()
}
