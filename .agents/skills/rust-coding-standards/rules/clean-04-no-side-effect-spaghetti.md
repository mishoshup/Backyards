---
id: CLEAN-04
enforcedBy: review
source: Functional core, imperative shell (Bernhardt); plan and apply as in Terraform
---

# Separate deciding from doing

Compute a plan as pure data first, then execute it. `plan` and `apply` are separate steps, so `plan` is safe to run anywhere.

```rust
// Incorrect
for entry in entries { if stale(entry) { remove(entry)?; } }

// Correct
let plan = plan_removals(&entries);
execute_removals(&plan)?;
```
