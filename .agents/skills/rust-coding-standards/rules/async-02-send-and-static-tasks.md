---
ruleId: ASYNC-02
enforcedBy: compiler; clippy::future_not_send
source: Tokio documentation (spawn requires Send + 'static); Clippy lint list (rust-lang.github.io/rust-clippy): future_not_send
---

# Spawned tasks own their data

A spawned future must be `Send + 'static`. Move owned values in. Do not smuggle references through unsafe or leaked statics.

```rust
// Incorrect
tokio::spawn(async { use_reference(&local).await });

// Correct
let owned = local.clone();
tokio::spawn(async move { use_owned(owned).await });
```
