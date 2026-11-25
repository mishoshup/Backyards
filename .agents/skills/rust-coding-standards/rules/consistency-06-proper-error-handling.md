---
id: CONSISTENCY-06
enforcedBy: clippy::unwrap_used, expect_used, map_err_ignore, let_underscore_must_use
source: Clippy lint list (rust-lang.github.io/rust-clippy): unwrap_used, expect_used, map_err_ignore; Rust API Guidelines (rust-lang.github.io/api-guidelines) C-GOOD-ERR
---

# Handle errors where they can be acted on

Convert errors at the boundary into typed errors with context. Do not discard the cause (`map_err(|_| ...)`). Report the failure to the user in plain words with the next step.

```rust
// Incorrect
let state = read_state().map_err(|_| StateError::Unreadable)?;

// Correct
let state = read_state().map_err(StateError::Unreadable)?;
```
