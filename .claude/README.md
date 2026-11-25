# Agent configuration

## Purpose

This directory configures the coding agent for this repository. Project rules are in `AGENTS.md`. Deterministic checks are in `Cargo.toml` (workspace lints), `clippy.toml`, `deny.toml` and `scripts/`.

## Contents

| Path | Purpose |
| --- | --- |
| `settings.json` | Formatting hook for Rust files, allowed commands, denied force pushes. |
| `skills/rust-coding-standards` | Rule files. Each names the lint or script that enforces it. |
| `skills/boxlite-adapter` | Verified BoxLite facts and the procedure for checking the pinned crate. |
| `skills/real-box-testing` | Procedure for tests that start a real garden, including cleanup and measurement. |
| `skills/ship-change` | Procedure from a passing gate to a reviewed pull request. |
| `agents/code-inline-reviewer.md` | Reviews a diff against the rule files. |
| `agents/holistic-reviewer.md` | Reviews architecture, safety and correctness beyond the rule files. |
| `commands/review-changes.md` | Runs both reviewers on the branch diff. |

`skills` is a symbolic link to `.agents/skills`.
