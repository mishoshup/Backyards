---
ruleId: CONSISTENCY-16
enforcedBy: cargo xtask policy (banned phrases); review
source: Project convention (general practice, enforced by review)
---

# No assistant or tooling jargon in code or docs

Source and documentation describe the product. They do not mention the assistant, prompts or the tool that wrote them.

```rust
// Incorrect
// AI-generated helper

// Correct
(no comment)
```
