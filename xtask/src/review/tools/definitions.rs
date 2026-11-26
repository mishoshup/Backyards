use serde_json::{Value, json};

use crate::review::tools::fetch_url::allowed_hosts;

#[must_use]
pub fn tool_definitions() -> Value {
    json!([
        function_definition(
            "read_file",
            "Read a file from the repository checkout. Returns numbered lines. Use start_line and end_line to read a range.",
            &json!({"path": {"type": "string"}, "start_line": {"type": "integer"}, "end_line": {"type": "integer"}}),
            &["path"],
        ),
        function_definition(
            "list_files",
            "List repository files whose path matches a glob such as crates/*/Cargo.toml or **/*.rs.",
            &json!({"glob": {"type": "string"}}),
            &["glob"],
        ),
        function_definition(
            "grep",
            "Search the repository for a regular expression. Returns file:line:text matches, at most 100.",
            &json!({"pattern": {"type": "string"}, "path": {"type": "string", "description": "Directory or file to search. Defaults to the repository root."}}),
            &["pattern"],
        ),
        function_definition(
            "fetch_url",
            &format!(
                "Fetch an https documentation page as plain text. Only these hosts are allowed: {}.",
                allowed_hosts().join(", ")
            ),
            &json!({"url": {"type": "string"}}),
            &["url"],
        ),
    ])
}

fn function_definition(
    name: &str,
    description: &str,
    properties: &Value,
    required: &[&str],
) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": {"type": "object", "properties": properties, "required": required},
        },
    })
}
