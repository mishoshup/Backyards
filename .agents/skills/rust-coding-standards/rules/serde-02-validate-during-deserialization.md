---
ruleId: SERDE-02
enforcedBy: unit tests of rejected values
source: serde documentation (container attribute try_from)
---

# Validate while parsing

Use `#[serde(try_from = "Raw")]` so a value that fails a rule never becomes a typed value. Runtime validation comes first. Types alone are not acceptance.

```rust
// Incorrect
let policy: Policy = serde_json::from_str(text)?;
validate(&policy)?;

// Correct
#[serde(try_from = "RawPolicy")] struct Policy { /* validated */ }
```
