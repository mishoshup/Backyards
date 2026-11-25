---
id: NAMING-03
enforcedBy: review
source: Project convention (general practice, enforced by review); Rust API Guidelines (rust-lang.github.io/api-guidelines) C-CASE
---

# Use spec-defined terms verbatim

A name defined by a protocol, file format or external API (`kid`, `uid`, a BoxLite field) is used exactly. The human expansion goes in a comment only if it is unclear.

```rust
// Incorrect
struct Header { key_identifier: String }  // wire name is kid

// Correct
#[serde(rename = "kid")]
struct Header { key_identifier: String }
```
