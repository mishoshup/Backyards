---
ruleId: CONSISTENCY-20
enforcedBy: review
source: Rust API Guidelines (rust-lang.github.io/api-guidelines) C-CASE, C-CONV, C-GETTER
---

# Name functions with a verb

A function that acts starts with a verb (`load_manifest`, `remove_entry`). A predicate starts with `is_`, `has_`, `can_` or `should_`. A conversion uses `to_`, `as_` or `into_` by convention.

```rust
// Incorrect
fn manifest_loading()

// Correct
fn load_manifest()
```
