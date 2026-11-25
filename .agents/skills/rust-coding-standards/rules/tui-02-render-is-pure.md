---
id: TUI-02
enforcedBy: snapshot tests with `TestBackend`; review
source: ratatui documentation (Widget rendering, TestBackend snapshots)
---

# Render from state and do nothing else

The draw function reads state and writes widgets. It never performs input, file or network work.

```rust
// Incorrect
fn render(frame: &mut Frame) { let usage = disk_usage().unwrap(); }

// Correct
fn render(frame: &mut Frame, state: &AppState) { draw_usage(frame, &state.usage); }
```
