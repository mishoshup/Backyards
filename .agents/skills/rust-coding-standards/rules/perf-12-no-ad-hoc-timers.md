---
id: PERF-12
enforcedBy: clippy::disallowed_methods (std::thread::sleep, tokio::time::sleep)
source: Project convention (general practice, enforced by review)
---

# No ad hoc sleeps or timers

A sleep hides a missing signal and slows every run. Wait on the real condition (a flag, a channel, a readiness probe) with a timeout. Retry policy lives in one named helper.

```rust
// Incorrect
tokio::time::sleep(Duration::from_secs(2)).await;

// Correct
wait_until_ready(&box_handle, READINESS_TIMEOUT).await?;
```
