use clap::ValueEnum;

use crate::review::ReviewerMode;

const OPENCODE_DEFAULT_MODEL: &str = "opencode/big-pickle";
const CLAUDE_DEFAULT_MODEL: &str = "sonnet";
const NO_TOOLS: &str = "";
const HOLISTIC_TOOLS: &str = "Read,Grep,Glob";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReviewRunner {
    Opencode,
    Claude,
}

impl ReviewRunner {
    #[must_use]
    pub const fn default_model(self) -> &'static str {
        match self {
            Self::Opencode => OPENCODE_DEFAULT_MODEL,
            Self::Claude => CLAUDE_DEFAULT_MODEL,
        }
    }

    #[must_use]
    pub const fn program(self) -> &'static str {
        match self {
            Self::Opencode => "opencode",
            Self::Claude => "claude",
        }
    }
}

#[must_use]
pub const fn claude_tools(mode: ReviewerMode) -> &'static str {
    match mode {
        ReviewerMode::Inline => NO_TOOLS,
        ReviewerMode::Holistic => HOLISTIC_TOOLS,
    }
}
