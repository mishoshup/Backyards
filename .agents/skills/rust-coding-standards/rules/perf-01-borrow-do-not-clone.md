---
id: PERF-01
enforcedBy: clippy::redundant_clone, needless_pass_by_value, clone_on_ref_ptr
source: Clippy lint list (rust-lang.github.io/rust-clippy): redundant_clone, needless_pass_by_value; The Rust Performance Book (heap allocations)
---

# Borrow instead of cloning in loops and hot paths

A clone inside a loop or a per-item function allocates for every item. Pass references and clone once at the boundary that needs ownership.

```rust
// Incorrect
for entry in entries { process(entry.clone()); }

// Correct
for entry in &entries { process(entry); }
```
