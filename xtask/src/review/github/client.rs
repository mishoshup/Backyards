use std::path::PathBuf;
use std::time::Duration;

use crate::report::GateError;
use crate::workspace::{capture_command, capture_command_limited};

#[derive(Debug, Clone)]
pub struct GitHub {
    program: String,
    repository: String,
    directory: PathBuf,
    posting_pause: Duration,
    retry_backoff: Duration,
}

const POSTING_PAUSE: Duration = Duration::from_secs(2);
const RETRY_BACKOFF: Duration = Duration::from_secs(10);
const POST_RETRIES: u32 = 2;

impl GitHub {
    #[must_use]
    pub fn new(repository: String, directory: PathBuf) -> Self {
        let mut github = Self::with_program("gh".to_owned(), repository, directory);
        github.posting_pause = POSTING_PAUSE;
        github.retry_backoff = RETRY_BACKOFF;
        github
    }

    #[must_use]
    pub const fn with_program(program: String, repository: String, directory: PathBuf) -> Self {
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
        &self.repository
    }

    pub fn call(&self, arguments: &[&str]) -> Result<String, GateError> {
        capture_command(&self.directory, &self.program, arguments, &[0])
    }

    pub fn post_content(&self, route_suffix: &str, options: &[&str]) -> Result<String, GateError> {
        std::thread::park_timeout(self.posting_pause);
        let mut result = self.api(route_suffix, options);
        for attempt in 1..=POST_RETRIES {
            if result.is_ok() {
                break;
            }
            std::thread::park_timeout(self.retry_backoff * attempt);
            result = self.api(route_suffix, options);
        }
        result
    }

    pub fn api_limited(
        &self,
        route_suffix: &str,
        options: &[&str],
        limit: usize,
    ) -> Result<String, GateError> {
        let route = format!("repos/{}/{route_suffix}", self.repository);
        let mut arguments = vec!["api", route.as_str()];
        arguments.extend_from_slice(options);
        capture_command_limited(&self.directory, &self.program, &arguments, limit)
    }

    pub fn api(&self, route_suffix: &str, options: &[&str]) -> Result<String, GateError> {
        let route = format!("repos/{}/{route_suffix}", self.repository);
        let mut arguments = vec!["api", route.as_str()];
        arguments.extend_from_slice(options);
        self.call(&arguments)
    }
}
