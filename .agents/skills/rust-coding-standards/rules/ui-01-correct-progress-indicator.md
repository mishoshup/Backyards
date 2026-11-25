---
ruleId: UI-01
enforcedBy: review
source: Project convention (general practice, enforced by review); clig.dev (progress, only when a terminal)
---

# Show the right progress indicator

Use a spinner for unknown duration and a bar with a count for known work. Draw progress only on a terminal. When output is piped, print plain lines.

```rust
// Incorrect
print_spinner_always()

// Correct
if output.is_interactive() { show_progress(total) } else { log_line(step) }
```
