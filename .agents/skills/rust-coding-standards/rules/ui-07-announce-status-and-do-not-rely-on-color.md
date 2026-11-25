---
id: UI-07
enforcedBy: review; output tests with NO_COLOR set
source: no-color.org; clig.dev (color); Project convention (general practice, enforced by review)
---

# State changes are visible without color

Honor `NO_COLOR`. Every status has a word or symbol as well as a color. Desktop controls carry accessible labels and are reachable by keyboard.

```rust
// Incorrect
print_in_red("failed")

// Correct
print_status(Status::Failed)  // "failed" text, red only when color is on
```
