---
id: CONSISTENCY-14
enforcedBy: review
source: Project convention (general practice, enforced by review)
---

# A comment states why, not what

A comment that restates the code adds noise and drifts out of date. State the constraint, bug or trade-off instead.

```rust
// Incorrect
// loop over entries
for entry in entries {}

// Correct
// Symlinks are skipped so a link cannot point the sweep outside the allowlisted root.
for entry in real_entries {}
```
