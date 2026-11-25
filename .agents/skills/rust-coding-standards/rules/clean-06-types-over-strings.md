---
ruleId: CLEAN-06
enforcedBy: review; clippy::wildcard_enum_match_arm
source: Rust API Guidelines (rust-lang.github.io/api-guidelines) C-NEWTYPE; Clippy lint list (rust-lang.github.io/rust-clippy): wildcard_enum_match_arm
---

# Use newtypes and enums instead of strings and flags

A `String` that holds a garden name, a path or a state is a bug waiting. Use `GardenName`, `ManagedPath` and an enum, and match every variant explicitly.

```rust
// Incorrect
fn stop(name: &str, state: &str)

// Correct
fn stop(name: &BoxName, state: BoxState)
```
