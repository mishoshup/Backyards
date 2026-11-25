---
ruleId: CONSISTENCY-05
enforcedBy: clippy::allow_attributes, allow_attributes_without_reason; scripts/check-policy.sh
source: Clippy lint list (rust-lang.github.io/rust-clippy): allow_attributes, allow_attributes_without_reason; Rust reference (lint reasons, RFC 2383)
---

# Use expect with a reason, never allow

A silenced lint hides a fault. Use `#[expect(lint, reason = "...")]` so the suppression is reported when it is no longer needed. The safety lints cannot be suppressed at all.

```rust
// Incorrect
#[allow(clippy::too_many_arguments)]

// Correct
#[expect(clippy::too_many_arguments, reason = "mirrors the BoxLite option struct field for field")]
```
