use regex::Regex;

use crate::report::GateError;
use crate::review::github::client::GitHub;

const REVIEWABLE_PATTERN: &str = r"(\.rs$|(^|/)Cargo\.(toml|lock)$|^scripts/|^xtask/|^\.github/|^\.agents/|^\.claude/|^(clippy|deny|rustfmt)\.toml$|^rust-toolchain(\.toml)?$|(^|/)AGENTS\.md$)";

pub fn select_reviewable_pattern() -> Result<Regex, GateError> {
    Regex::new(REVIEWABLE_PATTERN).map_err(|source| GateError::InvalidPattern {
        pattern: REVIEWABLE_PATTERN.to_owned(),
        source,
    })
}

pub fn has_reviewable_path(changed_paths: &[String]) -> Result<bool, GateError> {
    let pattern = select_reviewable_pattern()?;
    Ok(changed_paths.iter().any(|path| pattern.is_match(path)))
}

impl GitHub {
    pub fn list_changed_paths(&self, pull_request: u64) -> Result<Vec<String>, GateError> {
        let output = self.call_api(
            &format!("pulls/{pull_request}/files"),
            &["--paginate", "--jq", ".[].filename"],
        )?;
        Ok(output.lines().map(str::to_owned).collect())
    }
}
