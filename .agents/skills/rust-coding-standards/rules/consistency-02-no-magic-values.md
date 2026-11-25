---
ruleId: CONSISTENCY-02
enforcedBy: clippy::unreadable_literal; review
source: Project convention (general practice, enforced by review)
---

# Name every magic value

Retention days, timeouts, capacities and thresholds are named constants or manifest settings, defined once.

```rust
// Incorrect
if age_days > 7 { remove(entry); }

// Correct
if age_days > DEFAULT_RETENTION_DAYS { remove(entry); }
```
