---
ruleId: GPUI-03
enforcedBy: must_use on `Task`; review
source: GPUI documentation (Task is cancelled on drop; Subscription; detach)
---

# Hold or detach every Task and Subscription

`cx.spawn` returns a `Task` that cancels when dropped. Store it in the owner for as long as the work should run, or call `detach()` on purpose. The same applies to `Subscription` from `cx.observe` and `cx.subscribe`.

```rust
// Incorrect
cx.spawn(async move |this, cx| { /* ... */ });  // dropped at once

// Correct
self.sweep_task = Some(cx.spawn(async move |this, cx| { /* ... */ }));
```
