---
id: CONSISTENCY-15
enforcedBy: scripts/check-policy.sh (no em dashes); review
source: Project convention (general practice, enforced by review)
---

# Write comments in plain declarative prose

Complete sentences, no slogans, no narrative, no comparisons with rejected options.

```rust
// Incorrect
// Here we cleverly dodge the classic race!

// Correct
// The lock file is created before the check so two runs cannot both pass it.
```
