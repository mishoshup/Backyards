---
ruleId: PRIVACY-01
enforcedBy: review; redaction unit tests; clippy::dbg_macro
source: Project convention (general practice, enforced by review); secrecy crate (Debug prints [REDACTED])
---

# Mask secrets in logs, audit entries and output

Tokens, keys, passwords and environment values never reach a log, an audit entry, a report or an error message. Wrap secrets in a type whose `Debug` and `Display` print a fixed mask.

```rust
// Incorrect
tracing::info!("token = {token}");

// Correct
tracing::info!("token = {}", Redacted(&token));
```
