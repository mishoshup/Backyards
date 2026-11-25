---
id: GPUI-01
enforcedBy: scripts/check-architecture.sh (desktop crate depends on feature crates only); review
source: Hexagonal architecture (Cockburn); GPUI Entity model
---

# Views call the feature crates and hold no business rules

A view reads an `Entity` and sends an intent to a feature crate. Rules about retention, safety or drift live in the feature crates.

```rust
// Incorrect
impl Render for CleanupView { fn render(..) { remove_old_files(); } }

// Correct
view_model.update(cx, |model, cx| model.request_sweep(cx));
```
