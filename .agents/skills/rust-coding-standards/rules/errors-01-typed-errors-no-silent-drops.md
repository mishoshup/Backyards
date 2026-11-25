---
ruleId: ERRORS-01
enforcedBy: clippy::unwrap_used, expect_used, panic, let_underscore_must_use; review for silent drops
source: thiserror and anyhow documentation (libraries typed, binaries anyhow); workspace AGENTS.md silent-drop rule
---

# Typed errors and no silent drops

Library crates return `Result<_, ErrorType>` with a `thiserror` enum. `anyhow` appears only in the binary entry point. No `unwrap`, `expect` or `panic!` outside tests.

A silent drop is a defect. Any path that skips an item, ignores an error or disables a feature records why, and the report shows it. Examples: a cache entry that cannot be read, a garden that refuses to stop, a channel that is off.

Do not write `let _ = fallible();`. Handle the error or record it.
