use std::path::{Path, PathBuf};

use regex::Regex;

use crate::error::GateError;
use crate::finding::Finding;
use crate::line_scan::{ScanRule, report_matching_lines};
use crate::repository::{Repository, compile_pattern};

const AREA: &str = "architecture";
const BINARY_CRATE: &str = "backyards";
const FRONT_END_CRATES: [&str; 3] = ["backyards-mcp", "backyards-desktop", "backyards-tui"];
const ADAPTER_PATTERN: &str = r"(?m)^\s*(gardens-boxlite|manifest-json)";
const ANYHOW_PATTERN: &str = r"(?m)^\s*anyhow\s*=";
const GPUI_SHELL_PATTERN: &str = r"(?m)^\s*gpui-shell\s*=";
const TOOLING_DEPENDENCY_PATTERN: &str = r"(?m)^\s*xtask(\.workspace)?\s*=";

pub fn check_manifests(
    repository: &Repository,
    crate_names: &[String],
) -> Result<Vec<Finding>, GateError> {
    let mut findings = Vec::new();
    findings.extend(find_anyhow_outside_binary(repository, crate_names)?);
    findings.extend(find_gpui_shell(repository, crate_names)?);
    findings.extend(find_front_ends_using_adapters(repository)?);
    findings.extend(find_tooling_dependencies(repository, crate_names)?);
    findings.extend(find_forbidden_feature_dependencies(
        repository,
        crate_names,
    )?);
    Ok(findings)
}

fn find_anyhow_outside_binary(
    repository: &Repository,
    crate_names: &[String],
) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(ANYHOW_PATTERN)?;
    let library_crates: Vec<&String> = crate_names
        .iter()
        .filter(|name| name.as_str() != BINARY_CRATE)
        .collect();
    let mut findings = Vec::new();
    for name in library_crates {
        findings.extend(describe_manifest_matches(
            repository,
            &Repository::crate_manifest_path(name),
            &pattern,
            "uses anyhow outside the binary crate",
        )?);
    }
    Ok(findings)
}

fn find_gpui_shell(
    repository: &Repository,
    crate_names: &[String],
) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(GPUI_SHELL_PATTERN)?;
    let mut findings = Vec::new();
    for manifest in list_all_manifests(crate_names) {
        findings.extend(describe_manifest_matches(
            repository,
            &manifest,
            &pattern,
            "gpui-shell is not allowed",
        )?);
    }
    Ok(findings)
}

fn find_front_ends_using_adapters(repository: &Repository) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(ADAPTER_PATTERN)?;
    let mut findings = Vec::new();
    for name in FRONT_END_CRATES {
        findings.extend(describe_manifest_matches(
            repository,
            &Repository::crate_manifest_path(name),
            &pattern,
            "front end depends on an adapter",
        )?);
    }
    Ok(findings)
}

fn find_tooling_dependencies(
    repository: &Repository,
    crate_names: &[String],
) -> Result<Vec<Finding>, GateError> {
    let pattern = compile_pattern(TOOLING_DEPENDENCY_PATTERN)?;
    let mut findings = Vec::new();
    for name in crate_names {
        findings.extend(describe_manifest_matches(
            repository,
            &Repository::crate_manifest_path(name),
            &pattern,
            "product crate depends on the xtask tooling crate",
        )?);
    }
    Ok(findings)
}

fn find_forbidden_feature_dependencies(
    repository: &Repository,
    crate_names: &[String],
) -> Result<Vec<Finding>, GateError> {
    let mut findings = Vec::new();
    for name in crate_names {
        let allowed = list_allowed_dependencies(name, crate_names);
        let declared = list_declared_dependencies(repository, name, crate_names)?;
        findings.extend(describe_forbidden(name, &declared, &allowed));
    }
    Ok(findings)
}

fn describe_forbidden(crate_name: &str, declared: &[String], allowed: &[String]) -> Vec<Finding> {
    declared
        .iter()
        .filter(|dependency| !allowed.contains(dependency))
        .map(|dependency| {
            Finding::error(
                AREA,
                format!(
                    "crates/{crate_name} depends on {dependency}, features may depend only on kernel and their own adapters"
                ),
            )
        })
        .collect()
}

fn list_allowed_dependencies(crate_name: &str, crate_names: &[String]) -> Vec<String> {
    match crate_name {
        "kernel" => Vec::new(),
        "manifest" => vec!["kernel".to_owned()],
        "manifest-json" => vec!["kernel".to_owned(), "manifest".to_owned()],
        name if name == BINARY_CRATE || name.starts_with("backyards-") => crate_names.to_vec(),
        name => list_feature_dependencies(name, crate_names),
    }
}

fn list_feature_dependencies(crate_name: &str, crate_names: &[String]) -> Vec<String> {
    let mut allowed = vec!["kernel".to_owned()];
    allowed.extend(
        crate_names
            .iter()
            .filter(|candidate| crate_name.starts_with(&format!("{candidate}-")))
            .cloned(),
    );
    allowed
}

fn list_declared_dependencies(
    repository: &Repository,
    crate_name: &str,
    crate_names: &[String],
) -> Result<Vec<String>, GateError> {
    let manifest = repository.read_text(&Repository::crate_manifest_path(crate_name))?;
    let mut declared = Vec::new();
    for candidate in crate_names {
        let pattern = compile_pattern(&format!(
            r"(?m)^{}(\.workspace)?\s*=",
            regex::escape(candidate)
        ))?;
        if pattern.is_match(&manifest) {
            declared.push(candidate.clone());
        }
    }
    Ok(declared)
}

fn list_all_manifests(crate_names: &[String]) -> Vec<PathBuf> {
    let mut manifests = vec![PathBuf::from("Cargo.toml")];
    manifests.extend(
        crate_names
            .iter()
            .map(|name| Repository::crate_manifest_path(name)),
    );
    manifests
}

fn describe_manifest_matches(
    repository: &Repository,
    manifest: &Path,
    pattern: &Regex,
    heading: &str,
) -> Result<Vec<Finding>, GateError> {
    if !repository.has_file(manifest) {
        return Ok(Vec::new());
    }
    report_matching_lines(
        repository,
        &ScanRule::new(AREA, heading, pattern),
        &[manifest.to_path_buf()],
    )
}
