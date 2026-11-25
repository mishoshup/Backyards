---
ruleId: PERF-02
enforcedBy: review; clippy::needless_continue
source: Project convention (general practice, enforced by review)
---

# Return early before expensive work

Check the cheap rejecting conditions first so the expensive scan, read or allocation never runs for an item that will be skipped.

```rust
// Incorrect
let sizes = measure_all(&paths)?;
if paths.is_empty() { return Ok(Report::empty()); }

// Correct
if paths.is_empty() { return Ok(Report::empty()); }
let sizes = measure_all(&paths)?;
```
