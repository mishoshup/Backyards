---
id: CLI-04
enforcedBy: review; integration tests assert stdout, stderr and the exit code
source: clig.dev (output: stdout for data, stderr for messages; exit codes)
---

# Data on stdout, messages on stderr, stable exit codes

Machine output goes to stdout. Progress and diagnostics go to stderr. Exit codes are an enum: success, usage error, partial failure, failure. Agents depend on them.

```rust
// Incorrect
println!("warning: skipped"); // mixed with data

// Correct
output.warn(Message::Skipped);  // stderr
```
