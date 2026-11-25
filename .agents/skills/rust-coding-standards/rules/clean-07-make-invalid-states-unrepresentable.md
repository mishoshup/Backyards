---
id: CLEAN-07
enforcedBy: review; clippy::option_option, struct_field_names
source: Parse, don't validate (King); Rust API Guidelines (rust-lang.github.io/api-guidelines) C-VALIDATE
---

# Make invalid states unrepresentable

Validate once at the boundary and carry a type that cannot hold a bad value, so inner code needs no recheck.

```rust
// Incorrect
struct Retention { days: i64 }

// Correct
struct RetentionDays(NonZeroU16);
```
