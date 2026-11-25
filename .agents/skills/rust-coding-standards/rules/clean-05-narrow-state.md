---
id: CLEAN-05
enforcedBy: review; clippy::rc_mutex, mutex_atomic
source: Project convention (general practice, enforced by review); Clippy lint list (rust-lang.github.io/rust-clippy): rc_mutex
---

# Keep state narrow and owned in one place

One owner holds each piece of mutable state. Share by message or by a narrow handle, not by a broad `Arc<Mutex<App>>`.

```rust
// Incorrect
let app = Arc::new(Mutex::new(App::default()));

// Correct
let cleanup_state = CleanupState::new();  // owned by the cleanup task, queried by message
```
