---
ruleId: CONSISTENCY-18
enforcedBy: review
source: Project convention (general practice, enforced by review)
---

# Comment code that is not self-explanatory

A non-obvious branch, workaround, magic-looking condition or ordering dependency carries a comment with the reason. Obvious code carries none.

```rust
// Incorrect
if entry.is_partial() && age < cutoff { discard(entry); }

// Correct
// A partial download older than the cutoff never completes because the registry drops the session.
if entry.is_partial() && age < cutoff { discard(entry); }
```
