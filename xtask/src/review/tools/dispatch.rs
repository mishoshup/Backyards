use std::path::Path;

use serde::de::DeserializeOwned;

use crate::review::tools::error::ToolError;
use crate::review::tools::fetch_url::fetch_url;
use crate::review::tools::grep::grep;
use crate::review::tools::list_files::list_files;
use crate::review::tools::read_file::read_file;

#[derive(Debug, Clone, Copy)]
pub struct ToolRunner<'root> {
    root: &'root Path,
}

impl<'root> ToolRunner<'root> {
    #[must_use]
    pub const fn new(root: &'root Path) -> Self {
        Self { root }
    }

    #[must_use]
    pub fn run(&self, name: &str, arguments: &str) -> String {
        match self.dispatch(name, arguments) {
            Ok(output) => output,
            Err(error) => format!("error: {error}"),
        }
    }

    fn dispatch(&self, name: &str, arguments: &str) -> Result<String, ToolError> {
        match name {
            "read_file" => read_file(self.root, &parse_arguments(arguments)?),
            "list_files" => list_files(self.root, &parse_arguments(arguments)?),
            "grep" => grep(self.root, &parse_arguments(arguments)?),
            "fetch_url" => fetch_url(&parse_arguments(arguments)?),
            other => Err(ToolError::UnknownTool {
                name: other.to_owned(),
            }),
        }
    }
}

fn parse_arguments<T: DeserializeOwned>(arguments: &str) -> Result<T, ToolError> {
    serde_json::from_str(arguments).map_err(|source| ToolError::InvalidArguments {
        reason: source.to_string(),
    })
}
