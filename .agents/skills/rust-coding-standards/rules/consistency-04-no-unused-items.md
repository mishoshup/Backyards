---
ruleId: CONSISTENCY-04
enforcedBy: rustc dead_code, unused_variables; clippy::unused_self, used_underscore_binding
source: rustc dead_code and unused lints; Clippy lint list (rust-lang.github.io/rust-clippy): unused_self
---

# No unused parameters, fields or items

An unused parameter or field misleads readers about what matters. Remove it. Do not prefix with an underscore to silence the warning.

```rust
// Incorrect
fn plan(manifest: &Manifest, _machine: &MachineState) -> Plan

// Correct
fn plan(manifest: &Manifest) -> Plan
```
