---
ruleId: TUI-03
enforcedBy: review
source: ratatui documentation (event handling and async patterns)
---

# Keep slow work off the draw loop

Run feature calls on a task and send results to the loop as messages. The loop only reads input and draws, so the interface stays responsive.

```rust
// Incorrect
let report = features.sweep().await?;  // inside the draw loop

// Correct
tasks.spawn(run_sweep(features.clone(), sender.clone()));
```
