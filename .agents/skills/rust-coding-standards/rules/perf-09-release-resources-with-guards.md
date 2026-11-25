---
id: PERF-09
enforcedBy: review; real-garden tests assert nothing is left behind
source: Rust API Guidelines C-DTOR-FAIL and RAII guards in std (MutexGuard, tempfile)
---

# Release every resource through a guard

A garden, a process, a temporary directory or a lock that is released only on the happy path leaks on error. Wrap each in a type whose `Drop` or explicit async close releases it, and hold or join every spawned task.

```rust
// Incorrect
let box_handle = backend.start(spec)?;
run(&box_handle)?;
backend.remove(box_handle)?;

// Correct
let box_guard = BoxGuard::start(backend, spec)?;
run(box_guard.handle())?;
```
