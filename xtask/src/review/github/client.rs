use std::path::PathBuf;

use crate::report::GateError;
use crate::workspace::capture_command;

#[derive(Debug, Clone)]
pub struct GitHub {
    program: String,
    repository: String,
    directory: PathBuf,
}

impl GitHub {
    #[must_use]
    pub fn new(repository: String, directory: PathBuf) -> Self {
        Self::with_program("gh".to_owned(), repository, directory)
    }

    #[must_use]
    pub const fn with_program(program: String, repository: String, directory: PathBuf) -> Self {
        Self {
            program,
            repository,
            directory,
        }
    }

    #[must_use]
    pub fn repository(&self) -> &str {
        &self.repository
    }

    pub fn call(&self, arguments: &[&str]) -> Result<String, GateError> {
        capture_command(&self.directory, &self.program, arguments, &[0])
    }

    pub fn api(&self, route_suffix: &str, options: &[&str]) -> Result<String, GateError> {
        let route = format!("repos/{}/{route_suffix}", self.repository);
        let mut arguments = vec!["api", route.as_str()];
        arguments.extend_from_slice(options);
        self.call(&arguments)
    }
}
