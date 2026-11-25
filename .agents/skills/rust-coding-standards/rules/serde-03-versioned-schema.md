---
id: SERDE-03
enforcedBy: generated schema file is checked in and compared in CI
source: schemars documentation (JsonSchema derive)
---

# Version and publish the schema

The manifest carries `schema_version`. A JSON Schema is generated from the Rust types (`schemars`), so agents and editors validate before running. Unknown versions are rejected with a clear message.

```rust
// Incorrect
{ "toolchains": [] }

// Correct
{ "schema_version": 1, "toolchains": [] }
```
