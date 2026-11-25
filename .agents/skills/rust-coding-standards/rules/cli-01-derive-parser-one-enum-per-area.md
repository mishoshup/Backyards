---
ruleId: CLI-01
enforcedBy: cargo xtask architecture (commands registered in one table); unit test
source: clap documentation (derive tutorial, Parser, Subcommand)
---

# Define the CLI with the clap derive API

One `Parser` struct, one `Subcommand` enum per area, one `Args` struct per command. Add `propagate_version`. Commands call the feature crates and hold no logic. (clap documentation checked.)

```rust
// Incorrect
Command::new("backyards").arg(Arg::new("path"))  // builder sprawl

// Correct
#[derive(Subcommand)] enum CleanupCommand { Plan(PlanArguments), Sweep(SweepArguments) }
```
