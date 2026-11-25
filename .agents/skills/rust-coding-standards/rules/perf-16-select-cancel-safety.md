---
ruleId: PERF-16
enforcedBy: review
source: Tokio documentation (tokio::select! cancellation safety)
---

# Use only cancel-safe futures inside select

A `select!` branch that loses the race is dropped. Do not put a future there that loses data when dropped (a partial read into a local buffer). Use a cancel-safe method or move the read into its own task.

```rust
// Incorrect
select! { _ = token.cancelled() => {}, line = read_line_into_buffer(&mut buffer) => {} }

// Correct
select! { _ = token.cancelled() => {}, line = lines.next_line() => {} }
```
