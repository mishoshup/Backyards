---
ruleId: GPUI-04
enforcedBy: clippy::disallowed_methods; review
source: GPUI documentation (background_spawn, async executor)
---

# Never block the main thread

File, process and feature calls run on a background task. The main thread draws and handles input only.

```rust
// Incorrect
let report = std::fs::read_to_string(path)?;  // in render or an action handler

// Correct
cx.background_spawn(async move { features.read_report().await })
```
