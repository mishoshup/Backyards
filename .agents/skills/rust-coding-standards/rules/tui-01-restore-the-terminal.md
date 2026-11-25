---
id: TUI-01
enforcedBy: review; test that a panic path restores
source: ratatui documentation (ratatui::run, init, restore, panic hook)
---

# Always restore the terminal

Start with `ratatui::run` (or `init` plus `restore`), which installs a panic hook. A crash must never leave the terminal in raw mode. (ratatui documentation checked.)

```rust
// Incorrect
enable_raw_mode(); run_app();  // no restore on error

// Correct
ratatui::run(|terminal| app.run(terminal))
```
