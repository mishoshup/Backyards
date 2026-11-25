---
id: UI-06
enforcedBy: scripts/check-architecture.sh (layer import table)
source: Project convention (general practice, enforced by review)
---

# Layer components and import downward only

Components are atoms, molecules or screens. A layer imports only from layers below it.

```rust
// Incorrect
an atom importing a screen

// Correct
a screen composing molecules, molecules composing atoms
```
