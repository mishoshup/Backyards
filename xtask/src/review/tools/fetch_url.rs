use regex::Regex;
use serde::Deserialize;

use crate::review::tools::error::ToolError;
use crate::review::tools::limits::{FETCH_TIMEOUT, MAXIMUM_FETCH_BYTES, truncate_output};

const ALLOWED_HOSTS_TEXT: &str = include_str!("allowed_fetch_hosts.txt");
const HTTPS_PREFIX: &str = "https://";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchUrlArguments {
    pub url: String,
}

#[must_use]
pub fn allowed_hosts() -> Vec<&'static str> {
    ALLOWED_HOSTS_TEXT
        .lines()
        .map(str::trim)
        .filter(|host| !host.is_empty())
        .collect()
}

pub fn fetch_url(arguments: &FetchUrlArguments) -> Result<String, ToolError> {
    extract_allowed_host(&arguments.url)?;
    let page = download_page(&arguments.url)?;
    Ok(truncate_output(&reduce_to_text(&page)?))
}

fn extract_allowed_host(url: &str) -> Result<String, ToolError> {
    let remainder = url.strip_prefix(HTTPS_PREFIX).ok_or(ToolError::OnlyHttps)?;
    let host = remainder.split(['/', '?', '#']).next().unwrap_or_default();
    if allowed_hosts().contains(&host) {
        Ok(host.to_owned())
    } else {
        Err(ToolError::HostNotAllowed {
            host: host.to_owned(),
        })
    }
}

fn download_page(url: &str) -> Result<String, ToolError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .https_only(true)
        .timeout_global(Some(FETCH_TIMEOUT))
        .build()
        .into();
    let fail = |reason: String| ToolError::FetchFailed { reason };
    agent
        .get(url)
        .call()
        .map_err(|source| fail(source.to_string()))?
        .into_body()
        .with_config()
        .limit(MAXIMUM_FETCH_BYTES)
        .read_to_string()
        .map_err(|source| fail(source.to_string()))
}

fn reduce_to_text(page: &str) -> Result<String, ToolError> {
    let without_scripts = compile(r"(?s)<script[^>]*>.*?</script>")?.replace_all(page, "");
    let without_tags = compile(r"<[^>]*>")?.replace_all(&without_scripts, " ");
    let decoded = without_tags
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&amp;", "&");
    Ok(compile(r"[ \t]+")?.replace_all(&decoded, " ").into_owned())
}

fn compile(pattern: &str) -> Result<Regex, ToolError> {
    Regex::new(pattern).map_err(|source| ToolError::InvalidPattern {
        reason: source.to_string(),
    })
}
