---
id: CONSISTENCY-10
enforcedBy: clippy::missing_docs_in_private_items is off; review
source: Rust API Guidelines (rust-lang.github.io/api-guidelines) C-FAILURE (document errors and panics); rustdoc book
---

# Document a public item only when the signature cannot carry the contract

Add rustdoc to a public item when callers need information the types do not show (units, ordering, side effects). Use `# Errors` and `# Panics` sections in that order. Do not repeat the name.

```rust
// Incorrect
/// Gets the name.
pub fn name(&self) -> &str

// Correct
/// Removes entries older than the retention period and returns the bytes freed.
///
/// # Errors
/// Returns an error when an entry sits outside an allowlisted root.
pub fn sweep(&self) -> Result<u64, SweepError>
```
