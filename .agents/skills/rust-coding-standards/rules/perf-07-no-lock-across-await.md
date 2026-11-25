---
id: PERF-07
enforcedBy: clippy::await_holding_lock, await_holding_refcell_ref
source: Clippy lint list (rust-lang.github.io/rust-clippy): await_holding_lock; Tokio documentation (shared state)
---

# Never hold a lock guard across an await

A guard held across an await can deadlock and blocks other tasks. Copy out what is needed, drop the guard, then await.

```rust
// Incorrect
let guard = state.lock()?;
client.send(guard.value()).await?;

// Correct
let value = state.lock()?.value();
client.send(value).await?;
```
