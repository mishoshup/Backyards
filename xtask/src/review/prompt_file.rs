use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::report::GateError;
use crate::review::ReviewerMode;

const PROMPT_FILE_MODE: u32 = 0o600;

pub struct PromptFile {
    path: PathBuf,
    is_removed: bool,
}

impl PromptFile {
    pub fn create(mode: ReviewerMode, content: &str) -> Result<Self, GateError> {
        let path = unique_path(mode);
        write_private_file(&path, content).map_err(|source| GateError::WriteFile {
            path: path.clone(),
            source,
        })?;
        Ok(Self {
            path,
            is_removed: false,
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn remove(mut self) -> Result<(), GateError> {
        self.is_removed = true;
        std::fs::remove_file(&self.path).map_err(|source| GateError::RemoveFile {
            path: self.path.clone(),
            source,
        })
    }
}

impl Drop for PromptFile {
    fn drop(&mut self) {
        if !self.is_removed {
            drop(std::fs::remove_file(&self.path));
        }
    }
}

fn unique_path(mode: ReviewerMode) -> PathBuf {
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    std::env::temp_dir().join(format!(
        "backyards-review-{}-{}-{nanoseconds}.prompt",
        mode.label(),
        std::process::id()
    ))
}

fn write_private_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PROMPT_FILE_MODE)
        .open(path)?;
    file.write_all(content.as_bytes())
}
