---
id: CLI-02
enforcedBy: review; unit tests of rejected input
source: clap documentation (value_parser, ValueEnum)
---

# Validate input in value parsers and ValueEnum

Reject a bad value while parsing, with a clear error. Use `#[arg(value_enum)]` for fixed choices and a `value_parser` for ranges and paths.

```rust
// Incorrect
retention_days: String  // parsed later

// Correct
#[arg(value_parser = clap::value_parser!(u16).range(1..=365))] retention_days: u16
```
