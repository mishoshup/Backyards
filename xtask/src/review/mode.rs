use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReviewerMode {
    Inline,
    Holistic,
}

pub const ALL_REVIEWER_MODES: [ReviewerMode; 2] = [ReviewerMode::Inline, ReviewerMode::Holistic];

impl ReviewerMode {
    #[must_use]
    pub const fn agent_file(self) -> &'static str {
        match self {
            Self::Inline => ".claude/agents/code-inline-reviewer.md",
            Self::Holistic => ".claude/agents/holistic-reviewer.md",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Holistic => "holistic",
        }
    }

    #[must_use]
    pub const fn should_include_rule_files(self) -> bool {
        match self {
            Self::Inline => true,
            Self::Holistic => false,
        }
    }

    #[must_use]
    pub const fn attachment_request(self) -> &'static str {
        match self {
            Self::Inline => {
                "Follow the instructions in the attached file. The file holds the diff and every rule, so do not call tools or read other files. Answer with the JSON object only."
            }
            Self::Holistic => "Follow the instructions in the attached file.",
        }
    }

    #[must_use]
    pub const fn model_variable(self) -> &'static str {
        match self {
            Self::Inline => "REVIEW_MODEL_INLINE",
            Self::Holistic => "REVIEW_MODEL_HOLISTIC",
        }
    }

    #[must_use]
    pub const fn user_request(self) -> &'static str {
        match self {
            Self::Inline => {
                "Review this diff against every rule. Report a violation only for an added line. Answer in JSON."
            }
            Self::Holistic => {
                "Review this diff. Use the tools to read the repository and documentation before you report a finding. Answer in JSON."
            }
        }
    }
}
