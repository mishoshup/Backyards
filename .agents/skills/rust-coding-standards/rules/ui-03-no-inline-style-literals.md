---
ruleId: UI-03
enforcedBy: cargo xtask architecture (no color or pixel literals outside `theme`)
source: Project convention (general practice, enforced by review)
---

# No inline colors or sizes

Every color, size and font comes from the theme module of the front end. A view never writes a literal.

```rust
// Incorrect
div().bg(rgb(0x1e1e2e)).p(px(12.0))

// Correct
div().bg(theme.surface).p(theme.spacing.medium)
```
