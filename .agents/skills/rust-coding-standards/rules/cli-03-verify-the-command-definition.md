---
id: CLI-03
enforcedBy: unit test with `CommandFactory::command().debug_assert()`
source: clap documentation (Command::debug_assert, testing)
---

# Test the command definition

A debug assertion test catches conflicting flags and bad help text at build time. Every CLI crate has one.

```rust
// Incorrect
(no test)

// Correct
#[test] fn verify_cli() { Cli::command().debug_assert(); }
```
