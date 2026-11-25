use std::path::PathBuf;

use crate::error::GateError;
use crate::finding::Finding;
use crate::line_scan::{ScanRule, report_matching_lines};
use crate::repository::{Repository, compile_pattern};

const AREA: &str = "policy";
const MARKER_EXTENSIONS: [&str; 4] = ["rs", "sh", "toml", "yml"];
const TEXT_EXTENSIONS: [&str; 6] = ["rs", "md", "toml", "sh", "json", "yml"];
const MARKER_PATTERN: &str = r"(//|#)\s*(TODO|FIXME|HACK|XXX)\b";
const EM_DASH_PATTERN: &str = "\u{2014}";
// Split so the policy scan does not match this source file.
const CARGO_INSTALL_PATTERN: &str = concat!("cargo ", "install");
const LINT_OPT_OUT_PATTERN: &str = r"#!?\[(allow|expect)\((clippy::(unwrap_used|expect_used|panic|todo|min_ident_chars|too_many_lines|cognitive_complexity)|unsafe_code)";
const WORKSPACE_LINTS_PATTERN: &str = r"(?m)^\[lints\]\s*\nworkspace\s*=\s*true";

pub fn check_policy(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let mut findings = Vec::new();
    findings.extend(check_no_marker_comments(repository)?);
    findings.extend(check_no_em_dashes(repository)?);
    findings.extend(check_crates_inherit_workspace_lints(repository)?);
    findings.extend(check_no_lint_opt_outs(repository)?);
    findings.extend(check_no_cargo_install(repository)?);
    Ok(findings)
}

fn check_no_marker_comments(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let files = repository.list_files(&MARKER_EXTENSIONS)?;
    let pattern = compile_pattern(MARKER_PATTERN)?;
    report_matching_lines(
        repository,
        &ScanRule::new(AREA, "marker comment", &pattern),
        &files,
    )
}

fn check_no_em_dashes(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let files = repository.list_files(&TEXT_EXTENSIONS)?;
    let pattern = compile_pattern(EM_DASH_PATTERN)?;
    report_matching_lines(
        repository,
        &ScanRule::new(AREA, "em dash", &pattern),
        &files,
    )
}

fn check_no_cargo_install(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let files = repository.list_files(&TEXT_EXTENSIONS)?;
    let pattern = compile_pattern(CARGO_INSTALL_PATTERN)?;
    report_matching_lines(
        repository,
        &ScanRule::new(AREA, "use cargo binstall instead", &pattern),
        &files,
    )
}

fn check_no_lint_opt_outs(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let files = list_workspace_sources(repository)?;
    let pattern = compile_pattern(LINT_OPT_OUT_PATTERN)?;
    report_matching_lines(
        repository,
        &ScanRule::new(AREA, "lint opt-out", &pattern),
        &files,
    )
}

fn check_crates_inherit_workspace_lints(
    repository: &Repository,
) -> Result<Vec<Finding>, GateError> {
    let manifests = list_member_manifests(repository)?;
    let pattern = compile_pattern(WORKSPACE_LINTS_PATTERN)?;
    let mut findings = Vec::new();
    for manifest in manifests {
        let text = repository.read_text(&manifest)?;
        if !pattern.is_match(&text) {
            findings.push(Finding::error(
                AREA,
                format!(
                    "{} must contain [lints] workspace = true",
                    manifest.display()
                ),
            ));
        }
    }
    Ok(findings)
}

fn list_workspace_sources(repository: &Repository) -> Result<Vec<PathBuf>, GateError> {
    let files = repository.list_files(&["rs"])?;
    Ok(files
        .into_iter()
        .filter(|path| path.starts_with("crates") || path.starts_with("xtask"))
        .collect())
}

fn list_member_manifests(repository: &Repository) -> Result<Vec<PathBuf>, GateError> {
    let mut manifests: Vec<PathBuf> = repository
        .list_crate_names()?
        .iter()
        .map(|name| Repository::crate_manifest_path(name))
        .filter(|path| repository.has_file(path))
        .collect();
    let tooling_manifest = PathBuf::from("xtask/Cargo.toml");
    if repository.has_file(&tooling_manifest) {
        manifests.push(tooling_manifest);
    }
    Ok(manifests)
}
