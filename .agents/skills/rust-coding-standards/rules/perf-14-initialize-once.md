---
id: PERF-14
enforcedBy: clippy::disallowed_types (lazy statics); review
source: std::sync::OnceLock and LazyLock documentation; rustc lint static_mut_refs
---

# Initialize shared state once

Use `OnceLock` or `LazyLock` for process-wide setup, so a second call cannot rebuild it.

```rust
// Incorrect
static mut CONFIG: Option<Config> = None;

// Correct
static CONFIG: OnceLock<Config> = OnceLock::new();
```
