---
id: GPUI-02
enforcedBy: review
source: GPUI documentation (Context::notify)
---

# Call cx.notify only when state changed

`cx.notify()` schedules observers and redraws. Call it after an actual change, not on every update. (GPUI documentation checked.)

```rust
// Incorrect
model.update(cx, |model, cx| { model.refresh(); cx.notify(); });  // always

// Correct
model.update(cx, |model, cx| { if model.refresh() { cx.notify(); } });
```
