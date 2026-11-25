---
id: CONSISTENCY-12
enforcedBy: review
source: Project convention (general practice, enforced by review)
---

# Name callbacks and handlers for the action

Name a handler for what it does, not for the event that triggers it.

```rust
// Incorrect
fn on_click(&mut self)

// Correct
fn start_sweep(&mut self)
```
