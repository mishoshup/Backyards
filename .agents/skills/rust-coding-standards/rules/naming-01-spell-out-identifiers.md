---
id: NAMING-01
enforcedBy: clippy::min_ident_chars (clippy.toml allows only `id`)
source: workspace AGENTS.md verbose naming; clippy::min_ident_chars
---

# Spell out identifiers

Write `manifest_path`, not `mp`; `retention_days`, not `ret`. A reader should not have to expand a word.

A spec-defined term is used verbatim (`kid`, `uid`, `gid`, field names of an external API). Put the expansion in a comment only when the term would otherwise be unclear.

Booleans start with `is_`, `has_`, `should_` or `can_`.

```rust
// Bad
fn ret(p: &Path, d: u32) -> Vec<Entry> { ... }

// Good
fn expired_entries(cache_directory: &Path, retention_days: u32) -> Vec<Entry> { ... }
```
