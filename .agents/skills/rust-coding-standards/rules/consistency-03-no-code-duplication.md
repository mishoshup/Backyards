---
ruleId: CONSISTENCY-03
enforcedBy: review; the second caller moves the helper to a shared type
source: Project convention (general practice, enforced by review)
---

# Eliminate duplication

When a helper has a second caller, move it to a shared module and reuse it. Prefer shared records, traits and mappers over copies.

```rust
// Incorrect
fn format_size_a(bytes: u64) -> String { /* ... */ }
fn format_size_b(bytes: u64) -> String { /* same */ }

// Correct
output::format_byte_size(bytes)
```
