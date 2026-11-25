---
id: UI-08
enforcedBy: review
source: Project convention (general practice, enforced by review)
---

# Every action has a text label and a keyboard path

A button or command shows a clear verb label. Every action in the TUI and desktop app has a key binding and appears in the command palette or help.

```rust
// Incorrect
icon_button(Icon::Trash)

// Correct
button("Remove cache").on_action(RemoveCache)
```
