---
ruleId: STRUCTURE-02
enforcedBy: review
source: workspace AGENTS.md file layout and routing
---

# One file per area, one table for commands

Name each file after the behavior it holds: `sweep.rs`, `drift.rs`, `box_lifecycle.rs`. A reader finds a command's code from its name.

Register commands in one place, grouped by area through small named functions (`register_cleanup_commands`). The table reads as a summary of the interface.

Every command follows the same order: authorize, decode, load, act, respond. Shared steps live in one helper that every command calls.

Split a module that grows a second responsibility. Do not raise a lint threshold to avoid the split.
