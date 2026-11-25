---
id: CONSISTENCY-09
enforcedBy: review; scripts/check-architecture.sh (no `utils.rs`, `helpers.rs`, `misc.rs`, `common.rs`)
source: Project convention (general practice, enforced by review)
---

# Name files after the behavior they hold

A reader finds code from its name. Use `sweep.rs`, `drift.rs`, `box_lifecycle.rs`. Catch-all names are rejected.

```rust
// Incorrect
src/utils.rs

// Correct
src/retention_policy.rs
```
