use std::path::{Path, PathBuf};

use super::manifest_layers::check_manifests;
use crate::report::Finding;
use crate::report::GateError;
use crate::report::{ScanRule, report_matching_lines};
use crate::workspace::{Repository, compile_pattern};

const AREA: &str = "architecture";
const CATCH_ALL_STEMS: [&str; 4] = ["utils", "helpers", "misc", "common"];
const HARD_LINE_LIMIT: usize = 400;
const SOFT_LINE_LIMIT: usize = 300;
// Split so the architecture scan does not match this source file.
const RUNTIME_MACRO_PATTERN: &str = concat!(r"#\[tokio", r"::main\]");
const PLATFORM_CHECK_PATTERN: &str = concat!(r"cfg\(.*target", "_os");

pub fn check_architecture(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let sources = repository.list_workspace_sources()?;
    let crate_names = repository.list_crate_names()?;
    let mut findings = Vec::new();
    findings.extend(find_catch_all_file_names(&sources));
    findings.extend(find_runtime_outside_binaries(repository, &sources)?);
    findings.extend(find_platform_checks_outside_platform(repository, &sources)?);
    findings.extend(find_files_over_length_limits(repository, &sources)?);
    findings.extend(check_manifests(repository, &crate_names)?);
    Ok(findings)
}

fn find_catch_all_file_names(sources: &[PathBuf]) -> Vec<Finding> {
    sources
        .iter()
        .filter(|path| has_catch_all_stem(path))
        .map(|path| Finding::error(AREA, format!("catch-all file name: {}", path.display())))
        .collect()
}

fn has_catch_all_stem(path: &Path) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| CATCH_ALL_STEMS.contains(&stem))
}

fn find_runtime_outside_binaries(
    repository: &Repository,
    sources: &[PathBuf],
) -> Result<Vec<Finding>, GateError> {
    let library_files: Vec<PathBuf> = sources
        .iter()
        .filter(|path| path.file_name().is_none_or(|name| name != "main.rs"))
        .cloned()
        .collect();
    let pattern = compile_pattern(RUNTIME_MACRO_PATTERN)?;
    report_matching_lines(
        repository,
        &ScanRule::new(AREA, "runtime macro outside a binary entry point", &pattern),
        &library_files,
    )
}

fn find_platform_checks_outside_platform(
    repository: &Repository,
    sources: &[PathBuf],
) -> Result<Vec<Finding>, GateError> {
    let outside_platform: Vec<PathBuf> = sources
        .iter()
        .filter(|path| !path.components().any(|part| part.as_os_str() == "platform"))
        .cloned()
        .collect();
    let pattern = compile_pattern(PLATFORM_CHECK_PATTERN)?;
    report_matching_lines(
        repository,
        &ScanRule::new(
            AREA,
            "operating system check outside a platform module",
            &pattern,
        ),
        &outside_platform,
    )
}

fn find_files_over_length_limits(
    repository: &Repository,
    sources: &[PathBuf],
) -> Result<Vec<Finding>, GateError> {
    let mut findings = Vec::new();
    for source in sources {
        let line_count = repository.read_text(source)?.lines().count();
        findings.extend(describe_length(source, line_count));
    }
    Ok(findings)
}

fn describe_length(source: &Path, line_count: usize) -> Option<Finding> {
    if line_count > HARD_LINE_LIMIT {
        return Some(Finding::error(
            AREA,
            format!(
                "{} has {line_count} lines, the hard limit is {HARD_LINE_LIMIT}",
                source.display()
            ),
        ));
    }
    (line_count > SOFT_LINE_LIMIT).then(|| {
        Finding::warning(
            AREA,
            format!(
                "{} has {line_count} lines, the soft limit is {SOFT_LINE_LIMIT}",
                source.display()
            ),
        )
    })
}
