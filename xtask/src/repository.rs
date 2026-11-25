use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

use crate::error::GateError;

const CRATES_DIRECTORY: &str = "crates";

#[derive(Debug, Clone)]
pub struct Repository {
    root: PathBuf,
}

impl Repository {
    #[must_use]
    pub const fn at(root: PathBuf) -> Self {
        Self { root }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn has_file(&self, relative: &Path) -> bool {
        self.root.join(relative).is_file()
    }

    pub fn list_files(&self, extensions: &[&str]) -> Result<Vec<PathBuf>, GateError> {
        let listing = self.run_git_listing()?;
        let files = parse_listing(&listing)?;
        Ok(files
            .into_iter()
            .filter(|path| self.has_file(path))
            .filter(|path| has_extension(path, extensions))
            .collect())
    }

    pub fn read_text(&self, relative: &Path) -> Result<String, GateError> {
        std::fs::read_to_string(self.root.join(relative)).map_err(|source| GateError::ReadFile {
            path: relative.to_path_buf(),
            source,
        })
    }

    pub fn list_crate_names(&self) -> Result<Vec<String>, GateError> {
        let directory = self.root.join(CRATES_DIRECTORY);
        if !directory.is_dir() {
            return Ok(Vec::new());
        }
        let mut names = collect_directory_names(&directory)?;
        names.sort();
        Ok(names)
    }

    #[must_use]
    pub fn crate_manifest_path(crate_name: &str) -> PathBuf {
        Path::new(CRATES_DIRECTORY)
            .join(crate_name)
            .join("Cargo.toml")
    }

    fn run_git_listing(&self) -> Result<Vec<u8>, GateError> {
        let output = Command::new("git")
            .args([
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ])
            .current_dir(&self.root)
            .output()
            .map_err(|source| GateError::RunCommand {
                command: "git ls-files".to_owned(),
                source,
            })?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(GateError::CommandFailed {
                command: "git ls-files".to_owned(),
                status: output.status,
            })
        }
    }
}

pub fn compile_pattern(pattern: &str) -> Result<Regex, GateError> {
    Regex::new(pattern).map_err(|source| GateError::InvalidPattern {
        pattern: pattern.to_owned(),
        source,
    })
}

fn parse_listing(listing: &[u8]) -> Result<Vec<PathBuf>, GateError> {
    listing
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(parse_entry)
        .collect()
}

fn parse_entry(entry: &[u8]) -> Result<PathBuf, GateError> {
    std::str::from_utf8(entry)
        .map(PathBuf::from)
        .map_err(GateError::NonUtf8Path)
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extensions.contains(&extension))
}

fn collect_directory_names(directory: &Path) -> Result<Vec<String>, GateError> {
    let read_error = |source: std::io::Error| GateError::ReadDirectory {
        path: directory.to_path_buf(),
        source,
    };
    let mut names = Vec::new();
    for entry in std::fs::read_dir(directory).map_err(read_error)? {
        let entry = entry.map_err(read_error)?;
        if entry.path().is_dir() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(names)
}
