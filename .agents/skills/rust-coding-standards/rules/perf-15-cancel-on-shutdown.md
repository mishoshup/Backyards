---
ruleId: PERF-15
enforcedBy: review; shutdown integration test
source: Tokio documentation (graceful shutdown: CancellationToken, JoinSet)
---

# Tie background tasks to a cancellation token

A task without a cancellation path outlives its owner. Pass a `CancellationToken`, select on it, and join the `JoinSet` during shutdown.

```rust
// Incorrect
tokio::spawn(async move { loop { poll().await; } });

// Correct
tasks.spawn(async move { while !token.is_cancelled() { poll().await; } });
```
