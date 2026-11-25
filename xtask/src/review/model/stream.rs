use serde::Deserialize;

use crate::review::error::ReviewError;

const DATA_PREFIX: &str = "data:";
const DONE_MARKER: &str = "[DONE]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamEvent {
    Ignored,
    Reasoning(String),
    ToolCall(ToolCallFragment),
    Content(String),
    Finished(String),
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

pub fn parse_stream_line(line: &str) -> Result<StreamEvent, ReviewError> {
    let Some(payload) = line.strip_prefix(DATA_PREFIX) else {
        return Ok(StreamEvent::Ignored);
    };
    let payload = payload.trim();
    if payload == DONE_MARKER {
        return Ok(StreamEvent::Done);
    }
    let chunk: Chunk =
        serde_json::from_str(payload).map_err(|source| ReviewError::MalformedStreamEvent {
            reason: source.to_string(),
        })?;
    Ok(interpret_chunk(chunk))
}

fn interpret_chunk(chunk: Chunk) -> StreamEvent {
    let Some(choice) = chunk.choices.into_iter().next() else {
        return StreamEvent::Ignored;
    };
    if let Some(reason) = choice.finish_reason {
        return StreamEvent::Finished(reason);
    }
    interpret_delta(choice.delta)
}

fn interpret_delta(delta: Delta) -> StreamEvent {
    if let Some(chunk) = delta.tool_calls.and_then(|calls| calls.into_iter().next()) {
        return StreamEvent::ToolCall(to_fragment(chunk));
    }
    match (delta.content, delta.reasoning_content) {
        (Some(content), _) if !content.is_empty() => StreamEvent::Content(content),
        (_, Some(reasoning)) if !reasoning.is_empty() => StreamEvent::Reasoning(reasoning),
        _ => StreamEvent::Ignored,
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
