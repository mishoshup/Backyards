---
ruleId: ASYNC-01
enforcedBy: scripts/check-architecture.sh (no `#[tokio::main]` in library crates)
source: Tokio documentation (runtime in the binary; #[tokio::main])
---

# Only binaries build a runtime

Library crates expose async functions and take a handle or token. The binary entry point owns the runtime, so tests and other front ends choose their own.

```rust
// Incorrect
// in a feature crate
#[tokio::main] async fn run() {}

// Correct
// in backyards-cli
#[tokio::main] async fn main() -> ExitCode { /* call features */ }
```
