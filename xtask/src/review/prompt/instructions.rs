use std::path::Path;

use crate::report::GateError;
use crate::review::ReviewerMode;
use crate::workspace::Repository;

const FRONT_MATTER_OPENING: &str = "---\n";
const FRONT_MATTER_CLOSING: &str = "\n---\n";

pub fn load_reviewer_instructions(
    repository: &Repository,
    mode: ReviewerMode,
) -> Result<String, GateError> {
    let text = repository.read_text(Path::new(mode.agent_file()))?;
    Ok(strip_front_matter(&text).to_owned())
}

fn strip_front_matter(text: &str) -> &str {
    let Some(without_opening) = text.strip_prefix(FRONT_MATTER_OPENING) else {
        return text;
    };
    without_opening
        .split_once(FRONT_MATTER_CLOSING)
        .map_or(text, |(_, body)| body.trim_start())
}
