use std::path::{Path, PathBuf};

use regex::Regex;

use crate::report::Finding;
use crate::report::GateError;
use crate::report::{ScanRule, describe_matches};
use crate::workspace::{Repository, compile_pattern, has_extension};

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
    findings.extend(check_text_rules(repository)?);
    findings.extend(check_crates_inherit_workspace_lints(repository)?);
    Ok(findings)
}

struct TextRule {
    heading: &'static str,
    pattern: Regex,
    extensions: &'static [&'static str],
}

fn check_text_rules(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let rules = compile_text_rules()?;
    let mut findings = Vec::new();
    for file in repository.list_files(&TEXT_EXTENSIONS)? {
        let text = repository.read_text(&file)?;
        findings.extend(apply_text_rules(&rules, &file, &text));
    }
    Ok(findings)
}

fn compile_text_rules() -> Result<Vec<TextRule>, GateError> {
    Ok(vec![
        TextRule {
            heading: "marker comment",
            pattern: compile_pattern(MARKER_PATTERN)?,
            extensions: &MARKER_EXTENSIONS,
        },
        TextRule {
            heading: "em dash",
            pattern: compile_pattern(EM_DASH_PATTERN)?,
            extensions: &TEXT_EXTENSIONS,
        },
        TextRule {
            heading: "use cargo binstall instead",
            pattern: compile_pattern(CARGO_INSTALL_PATTERN)?,
            extensions: &TEXT_EXTENSIONS,
        },
        TextRule {
            heading: "lint opt-out",
            pattern: compile_pattern(LINT_OPT_OUT_PATTERN)?,
            extensions: &["rs"],
        },
    ])
}

fn apply_text_rules(rules: &[TextRule], file: &Path, text: &str) -> Vec<Finding> {
    rules
        .iter()
        .filter(|rule| has_extension(file, rule.extensions))
        .flat_map(|rule| {
            describe_matches(
                &ScanRule::new(AREA, rule.heading, &rule.pattern),
                file,
                text,
            )
        })
        .collect()
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
