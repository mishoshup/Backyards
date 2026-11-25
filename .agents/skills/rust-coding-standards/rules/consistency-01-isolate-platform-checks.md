---
ruleId: CONSISTENCY-01
enforcedBy: scripts/check-architecture.sh (`cfg(target_os` only under `platform/`)
source: Hexagonal architecture (Cockburn); platform modules as in std::sys
---

# Keep platform checks in one module

Scattered `cfg(target_os)` branches make behavior hard to follow. Put platform differences behind one trait in a `platform` module.

```rust
// Incorrect
#[cfg(target_os = "macos")] fn free_space() {}  // in five files

// Correct
platform::free_space()  // one module owns the cfg
```
