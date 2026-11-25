---
ruleId: COMMENTS-01
enforcedBy: cargo xtask policy (markers, em dashes); review for the rest
source: general practice for non-obvious comments; workspace AGENTS.md comment policy
---

# Comment non-obvious code only

Comment code whose reason cannot be recovered from the code: a platform quirk, a workaround, an ordering dependency, a deliberate trade-off. State why it exists.

Put the rule in a name, a type, a lint or a test first. Code that reads clearly gets no comment. A comment does not restate the code, record history or describe future work. No `TODO`, `FIXME`, `HACK` or `XXX`.

```rust
// Bad: restates the code
// increment the counter
counter += 1;

// Good: records a reason the code cannot show
// The audit entry must be written before the delete, so a crash leaves a record of the attempt.
record_audit_entry(&target)?;
remove_target(&target)?;
```
