---
ruleId: UI-05
enforcedBy: review
source: Project convention (general practice, enforced by review)
---

# Reuse a shared component before writing one

Look in the shared view crate and in `gpui-kit` before building a new component. Do not hand-build a control the kit provides.

```rust
// Incorrect
a custom checkbox built from divs

// Correct
gpui_kit::component checkbox with the app theme
```
