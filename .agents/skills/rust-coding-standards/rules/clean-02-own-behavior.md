---
ruleId: CLEAN-02
enforcedBy: review
source: Project convention (general practice, enforced by review)
---

# A type owns its behavior

Put the logic next to the data it needs. A free function that takes a struct and reaches into five fields belongs on the struct, or in a type that owns those fields.

```rust
// Incorrect
fn is_expired(entry: &Entry, now: Instant) -> bool { now - entry.created > entry.ttl }

// Correct
impl Entry { fn is_expired(&self, now: Instant) -> bool { now - self.created > self.ttl } }
```
