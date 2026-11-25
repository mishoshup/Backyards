---
ruleId: GPUI-06
enforcedBy: cargo xtask architecture (only `gpui-kit` is listed, no direct `gpui` or `gpui-shell`); review
source: gpui-kit README (umbrella crate pins GPUI)
---

# Build on gpui-kit and its theme

Depend on the `gpui-kit` umbrella crate, which pins a matching GPUI release. Use its components and theme tokens. Do not add `gpui-shell` (the JavaScript extension layer). Verify each component API against the kit documentation before use.

```rust
// Incorrect
gpui = "..."  # a second GPUI version next to the kit

// Correct
gpui-kit = "0.7"
```
