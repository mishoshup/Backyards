---
ruleId: CLEAN-01
enforcedBy: review; clippy::fn_params_excessive_bools, struct_excessive_bools
source: Clippy lint list (rust-lang.github.io/rust-clippy): fn_params_excessive_bools, struct_excessive_bools; Project convention (general practice, enforced by review)
---

# Compose small types instead of adding flags

A function or type that grows boolean switches does several jobs. Split it into named pieces and compose them.

```rust
// Incorrect
fn sweep(path: &Path, dry_run: bool, verbose: bool, force: bool)

// Correct
fn sweep(plan: SweepPlan, mode: RunMode) -> SweepReport
```
