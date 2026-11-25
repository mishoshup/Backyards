---
ruleId: PERF-11
enforcedBy: review
source: Tokio documentation (try_join!); Project convention (general practice, enforced by review)
---

# Batch round trips and parallelize independent reads

Independent reads run concurrently. Writes inside one operation are batched on one connection or one pass and are never run in parallel against the same target.

```rust
// Incorrect
let usage = disk_usage().await?;
let images = list_images().await?;

// Correct
let (usage, images) = tokio::try_join!(disk_usage(), list_images())?;
```
