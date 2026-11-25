---
ruleId: PERF-10
enforcedBy: review; clippy::needless_collect
source: Project convention (general practice, enforced by review); The Rust Performance Book
---

# Hoist invariant work out of loops

Compile a regex, build a lookup map or resolve a path once before the loop.

```rust
// Incorrect
for line in lines { let pattern = Regex::new(PATTERN)?; pattern.is_match(line); }

// Correct
let pattern = Regex::new(PATTERN)?;
for line in lines { pattern.is_match(line); }
```
