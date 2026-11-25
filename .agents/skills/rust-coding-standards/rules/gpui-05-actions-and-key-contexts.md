---
ruleId: GPUI-05
enforcedBy: review; test that each action has a binding
source: GPUI documentation (actions!, on_action, key_context)
---

# Expose every command as an action

Define commands with `actions!`, handle them with `on_action`, and scope keys with `key_context`. This gives keyboard access and a command palette for free. (GPUI documentation checked.)

```rust
// Incorrect
div().on_click(|_, _, cx| remove_cache(cx))  // mouse only

// Correct
div().key_context("Cleanup").on_action(cx.listener(CleanupView::remove_cache))
```
