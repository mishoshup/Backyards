---
ruleId: NAMING-02
enforcedBy: review
source: Project convention (general practice, enforced by review)
---

# Prefix booleans with is, has, should or can

A boolean named for a state reads as a question.

```rust
// Incorrect
let dry = true;

// Correct
let is_dry_run = true;
```
