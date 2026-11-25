---
id: CLEAN-03
enforcedBy: clippy::disallowed_methods (std::env::var, std::env::current_dir in feature crates); review
source: Hexagonal architecture (Cockburn): ambient state behind ports
---

# Functions take what they need as arguments

Feature crates read no ambient state (environment variables, the working directory, the clock, the home directory). A port supplies each of them, so tests run in isolation.

```rust
// Incorrect
let home = std::env::var("HOME")?;

// Correct
let home = environment.home_directory();
```
