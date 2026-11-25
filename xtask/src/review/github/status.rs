use crate::report::GateError;
use crate::review::ReviewError;
use crate::review::github::client::GitHub;

const MAXIMUM_DESCRIPTION_CHARACTERS: usize = 140;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusState {
    Pending,
    Success,
    Failure,
}

impl StatusState {
    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "pending" => Some(Self::Pending),
            "success" => Some(Self::Success),
            "failure" => Some(Self::Failure),
            _ => None,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatusUpdate<'update> {
    pub head_sha: &'update str,
    pub context: &'update str,
    pub state: StatusState,
    pub description: &'update str,
}

impl GitHub {
    pub fn set_status(&self, update: &StatusUpdate<'_>) -> Result<(), GateError> {
        let description: String = update
            .description
            .chars()
            .take(MAXIMUM_DESCRIPTION_CHARACTERS)
            .collect();
        let state_field = format!("state={}", update.state.label());
        let context_field = format!("context={}", update.context);
        let description_field = format!("description={description}");
        let mut options = vec![
            "-X",
            "POST",
            "-f",
            &state_field,
            "-f",
            &context_field,
            "-f",
            &description_field,
        ];
        let target_field = run_url()?.map(|url| format!("target_url={url}"));
        if let Some(field) = &target_field {
            options.extend(["-f", field]);
        }
        self.api(&format!("statuses/{}", update.head_sha), &options)
            .map(|_| ())
    }

    pub fn read_status_state(
        &self,
        head_sha: &str,
        context: &str,
    ) -> Result<Option<StatusState>, GateError> {
        let filter =
            format!("[.statuses[] | select(.context == \"{context}\") | .state] | first // empty");
        let output = self.api(
            &format!("commits/{head_sha}/status"),
            &["--paginate", "--jq", &filter],
        )?;
        let state = output.trim();
        if state.is_empty() {
            return Ok(None);
        }
        StatusState::from_label(state).map(Some).ok_or_else(|| {
            ReviewError::UnknownStatusState {
                value: state.to_owned(),
            }
            .into()
        })
    }
}

fn run_url() -> Result<Option<String>, ReviewError> {
    let Ok(run) = std::env::var("GITHUB_RUN_ID") else {
        return Ok(None);
    };
    let server = require_environment("GITHUB_SERVER_URL")?;
    let repository = require_environment("GITHUB_REPOSITORY")?;
    Ok(Some(format!("{server}/{repository}/actions/runs/{run}")))
}

fn require_environment(name: &'static str) -> Result<String, ReviewError> {
    std::env::var(name).map_err(|source| ReviewError::MissingEnvironment { name, source })
}
