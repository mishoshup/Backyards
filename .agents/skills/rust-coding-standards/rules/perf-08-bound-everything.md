---
ruleId: PERF-08
enforcedBy: review; clippy::disallowed_methods (unbounded channels)
source: Tokio documentation (mpsc bounded channels, backpressure)
---

# Bound channels, input sizes and concurrency

Unbounded input is a memory and denial-of-service risk. Use bounded channels, cap list sizes and read limits, and limit concurrent work with a semaphore.

```rust
// Incorrect
let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

// Correct
let (sender, receiver) = tokio::sync::mpsc::channel(CHANNEL_CAPACITY);
```
