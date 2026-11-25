---
id: UI-02
enforcedBy: review; TUI and desktop snapshot tests at narrow sizes
source: Project convention (general practice, enforced by review)
---

# Long content scrolls and never overflows

A list, report or log view handles more rows than the window shows, in the TUI and the desktop app.

```rust
// Incorrect
render_all_rows(frame)

// Correct
render_visible_rows(frame, viewport)
```
