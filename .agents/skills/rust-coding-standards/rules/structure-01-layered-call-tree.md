---
id: STRUCTURE-01
enforcedBy: clippy::too_many_lines (30), clippy::cognitive_complexity (8); review for closures
source: workspace AGENTS.md code decomposition style; clippy::too_many_lines, cognitive_complexity
---

# Build a layered call tree

A top-level function delegates to a few clearly named helpers, which delegate further. The names alone describe the flow.

```rust
pub fn plan(manifest: &Manifest, machine: &MachineState) -> Plan {
    let desired = desired_state(manifest);
    let differences = compare_states(&desired, machine);
    order_steps(differences)
}
```

Closures are allowed only as short continuations that call a named function (`.map(summarize_entry)`). A business rule never lives inside a closure. Replace a flat iterator chain of more than three adapters with named helpers.

Add a doc comment to a public item only when the signature and types cannot carry the contract.
