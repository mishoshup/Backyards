---
ruleId: SERDE-01
enforcedBy: unit test that an unknown key fails; review
source: serde documentation (container attribute deny_unknown_fields)
---

# Reject unknown fields in manifests and requests

A typo in the manifest must fail loudly, not be ignored. Add `#[serde(deny_unknown_fields)]` to every input type. It does not combine with `flatten` (serde documentation checked), so avoid `flatten` on input types.

```rust
// Incorrect
#[derive(Deserialize)] struct Policy { retention_days: u16 }

// Correct
#[derive(Deserialize)] #[serde(deny_unknown_fields)] struct Policy { retention_days: u16 }
```
