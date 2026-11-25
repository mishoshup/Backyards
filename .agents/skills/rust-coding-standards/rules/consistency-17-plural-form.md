---
ruleId: CONSISTENCY-17
enforcedBy: review; message tests
source: Project convention (general practice, enforced by review)
---

# Use the correct plural in messages

Messages choose singular or plural from the count through one helper.

```rust
// Incorrect
format!("{count} file(s) removed")

// Correct
pluralize(count, "file")
```
