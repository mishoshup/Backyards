---
id: CONSISTENCY-11
enforcedBy: scripts/check-policy.sh; clippy::todo, unimplemented
source: Project convention (general practice, enforced by review); Clippy lint list (rust-lang.github.io/rust-clippy): todo, unimplemented
---

# No TODO, FIXME, HACK or XXX

A marker comment is unowned work. Add the behavior with a test, or open an issue.

```rust
// Incorrect
// TODO: handle symlinks

// Correct
// (open an issue and refuse symlinks that leave the root today)
```
