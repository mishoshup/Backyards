---
ruleId: CLI-05
enforcedBy: integration test per command; JSON schema generated from the types
source: clig.dev (--json output); gh and kubectl output flags
---

# Every command can emit JSON

A `--json` flag prints one typed document with a `schema_version`. The shape comes from the same types as the human output and has a generated schema.

```rust
// Incorrect
format!("{report:?}")

// Correct
serde_json::to_writer(stdout, &ReportDocument::from(report))
```
