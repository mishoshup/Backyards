---
ruleId: PERF-06
enforcedBy: clippy::disallowed_methods (std::thread::sleep, std::fs in async); review
source: Tokio documentation (bridging with sync code, spawn_blocking)
---

# Never block an async task

Blocking file or process calls stall the runtime thread and every task on it. Use the async API, or `tokio::task::spawn_blocking` for short blocking work. Long-lived blocking work gets a dedicated thread (checked against the tokio documentation).

```rust
// Incorrect
async fn read_state(path: &Path) -> Result<String> { Ok(std::fs::read_to_string(path)?) }

// Correct
async fn read_state(path: &Path) -> Result<String> { Ok(tokio::fs::read_to_string(path).await?) }
```
