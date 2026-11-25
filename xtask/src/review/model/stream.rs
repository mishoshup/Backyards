use serde::Deserialize;

use crate::review::error::ReviewError;

const DATA_PREFIX: &str = "data:";
const DONE_MARKER: &str = "[DONE]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    ToolCalls,
    Other(String),
}

impl FinishReason {
    fn from_label(label: String) -> Self {
        match label.as_str() {
            "stop" => Self::Stop,
            "tool_calls" => Self::ToolCalls,
            _ => Self::Other(label),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamEvent {
    Ignored,
    Reasoning(String),
    ToolCalls(Vec<ToolCallFragment>),
    Content(String),
    Finished(FinishReason),
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallFragment {
    pub index: usize,
    pub identifier: Option<String>,
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ToolCallChunk {
    index: usize,
    id: Option<String>,
    function: Option<FunctionChunk>,
}

#[derive(Debug, Deserialize)]
struct FunctionChunk {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Chunk {
    #[serde(default)]
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Delta,
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,
    tool_calls: Option<Vec<ToolCallChunk>>,
}

pub fn parse_stream_line(line: &str) -> Result<Vec<StreamEvent>, ReviewError> {
    let Some(payload) = line.strip_prefix(DATA_PREFIX) else {
        return Ok(vec![StreamEvent::Ignored]);
    };
    let payload = payload.trim();
    if payload == DONE_MARKER {
        return Ok(vec![StreamEvent::Done]);
    }
    let chunk: Chunk =
        serde_json::from_str(payload).map_err(|source| ReviewError::MalformedStreamEvent {
            reason: source.to_string(),
        })?;
    Ok(interpret_chunk(chunk))
}

fn interpret_chunk(chunk: Chunk) -> Vec<StreamEvent> {
    let Some(choice) = chunk.choices.into_iter().next() else {
        return vec![StreamEvent::Ignored];
    };
    let mut events = interpret_delta(choice.delta);
    if let Some(reason) = choice.finish_reason {
        events.push(StreamEvent::Finished(FinishReason::from_label(reason)));
    }
    events
}

fn interpret_delta(delta: Delta) -> Vec<StreamEvent> {
    let mut events = Vec::new();
    push_non_empty(&mut events, delta.reasoning_content, StreamEvent::Reasoning);
    push_non_empty(&mut events, delta.content, StreamEvent::Content);
    if let Some(calls) = delta.tool_calls {
        events.push(StreamEvent::ToolCalls(
            calls.into_iter().map(to_fragment).collect(),
        ));
    }
    if events.is_empty() {
        events.push(StreamEvent::Ignored);
    }
    events
}

fn push_non_empty(
    events: &mut Vec<StreamEvent>,
    text: Option<String>,
    build: fn(String) -> StreamEvent,
) {
    if let Some(text) = text.filter(|candidate| !candidate.is_empty()) {
        events.push(build(text));
    }
}

fn to_fragment(chunk: ToolCallChunk) -> ToolCallFragment {
    let (name, arguments) = chunk
        .function
        .map_or((None, None), |function| (function.name, function.arguments));
    ToolCallFragment {
        index: chunk.index,
        identifier: chunk.id,
        name,
        arguments,
    }
}
