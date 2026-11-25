---
allowed-tools: Bash(git diff:*),Bash(git merge-base:*),Bash(git status:*)
description: Review the current branch diff with the rule-based and holistic reviewers.
---

Review the changes on this branch.

1. Compute the diff with `git diff $(git merge-base origin/main HEAD)`. If `origin/main` does not exist, use `git diff HEAD`.
2. Start two subagents in parallel. One follows `.claude/agents/code-inline-reviewer.md`. The other follows `.claude/agents/holistic-reviewer.md`. Provide each with the diff.
3. Verify each reported finding against the source. Record whether it held.
4. Report the confirmed findings, most severe first, and list the findings that did not hold.
