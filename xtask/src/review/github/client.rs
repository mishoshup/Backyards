use std::path::PathBuf;
use std::time::Duration;

use crate::report::GateError;
use crate::review::github::repository_name::RepositoryName;
use crate::workspace::{capture_command, capture_command_limited, pause_for};

#[derive(Debug, Clone)]
pub struct GitHub {
    program: String,
    repository: RepositoryName,
    directory: PathBuf,
    posting_pause: Duration,
    retry_backoff: Duration,
}

const SUCCESS_EXIT: i32 = 0;
const POSTING_PAUSE: Duration = Duration::from_secs(2);
const RETRY_BACKOFF: Duration = Duration::from_secs(10);
const POST_RETRIES: u32 = 2;

impl GitHub {
    #[must_use]
    pub fn new(repository: RepositoryName, directory: PathBuf) -> Self {
        let mut github = Self::with_program("gh".to_owned(), repository, directory);
        github.posting_pause = POSTING_PAUSE;
        github.retry_backoff = RETRY_BACKOFF;
        github
    }

    #[must_use]
    pub const fn with_program(
        program: String,
        repository: RepositoryName,
        directory: PathBuf,
    ) -> Self {
        Self {
            program,
            repository,
            directory,
            posting_pause: Duration::ZERO,
            retry_backoff: Duration::ZERO,
        }
    }

    #[must_use]
    pub fn repository(&self) -> &str {
        self.repository.as_str()
    }

    pub fn call(&self, arguments: &[&str]) -> Result<String, GateError> {
        capture_command(&self.directory, &self.program, arguments, &[SUCCESS_EXIT])
    }

    pub fn post_content(&self, route_suffix: &str, options: &[&str]) -> Result<String, GateError> {
        pause_for(self.posting_pause);
        let mut result = self.call_api(route_suffix, options);
        for attempt in 1..=POST_RETRIES {
            if !result.as_ref().is_err_and(is_transient_failure) {
                break;
            }
            pause_for(self.retry_backoff * attempt);
            result = self.call_api(route_suffix, options);
        }
        result
    }

    pub fn call_api_limited(
        &self,
        route_suffix: &str,
        options: &[&str],
        limit: usize,
    ) -> Result<String, GateError> {
        let route = self.build_route(route_suffix);
        let arguments = build_api_arguments(&route, options);
        capture_command_limited(&self.directory, &self.program, &arguments, limit)
    }

    pub fn call_api(&self, route_suffix: &str, options: &[&str]) -> Result<String, GateError> {
        let route = self.build_route(route_suffix);
        self.call(&build_api_arguments(&route, options))
    }

    fn build_route(&self, route_suffix: &str) -> String {
        format!("repos/{}/{route_suffix}", self.repository.as_str())
    }
}

fn build_api_arguments<'argument>(
    route: &'argument str,
    options: &[&'argument str],
) -> Vec<&'argument str> {
    let mut arguments = vec!["api", route];
    arguments.extend_from_slice(options);
    arguments
}

fn is_transient_failure(error: &GateError) -> bool {
    let GateError::CommandFailed { stderr, .. } = error else {
        return false;
    };
    stderr.contains("HTTP 429") || stderr.contains("rate limit")
}
