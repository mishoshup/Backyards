use std::path::{Path, PathBuf};

use crate::review::tools::error::ToolError;

pub fn resolve_inside_repository(root: &Path, requested: &str) -> Result<PathBuf, ToolError> {
    let canonical_root = canonicalize(root)?;
    let resolved = canonicalize(&canonical_root.join(requested))?;
    if resolved.starts_with(&canonical_root) {
        Ok(resolved)
    } else {
        Err(ToolError::PathOutsideRepository)
    }
}

fn canonicalize(path: &Path) -> Result<PathBuf, ToolError> {
    path.canonicalize()
        .map_err(|source| ToolError::UnresolvablePath {
            reason: source.to_string(),
        })
}
