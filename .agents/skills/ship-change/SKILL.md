---
name: ship-change
description: Procedure for taking finished Backyards work to a reviewed pull request. Covers the gate, demonstration, independent review, branch and commit, and pull request. Use when asked to open, create, ship or submit a pull request, or when implementation is complete and review is next.
---

# Ship a change

## Completion criteria

The change is ready for handoff when all of the following hold:

1. The checks pass.
2. The change is demonstrated working.
3. An independent reviewer has read the diff.
4. Only judgment calls remain for the human.

No commit, push or post occurs without explicit approval for that batch.

## 1. Gate

```bash
scripts/check.sh
```

Correct every failure. Do not weaken a lint or add an opt-out. If a check cannot run, state which check and why.

## 2. Demonstration

Run the changed command or a real garden test and retain the output. For a destructive path, show the dry run and the audited real run against a temporary directory.

## 3. Independent review

Start one subagent that follows `.claude/agents/code-inline-reviewer.md` and one that follows `.claude/agents/holistic-reviewer.md`. Provide each with `git diff $(git merge-base origin/main HEAD)`. Verify every finding against the source and record whether it held.

## 4. Branch and commit

1. Work on a branch named `<type>/<short-description>`, created from an up-to-date `main`. Never commit on `main`.
2. Inspect `git status` for unrelated changes. Stage named files only.
3. Use the subject `<type>(<scope>): <subject>`, at most 72 characters, with no trailers.
4. Obtain approval before `git commit`.

## 5. Pull request

1. Confirm the account with `gh auth status`. It must be `mishoshup`.
2. Fetch, then push with `git push origin HEAD:refs/heads/<branch>`. Never force-push.
3. Open a draft pull request using the repository template. The title is `<type>(<scope>): <subject>` and its type matches the branch type.
4. Run the naming check locally first: `PULL_REQUEST_TITLE=... PULL_REQUEST_BRANCH=... scripts/check-pull-request-naming.sh`.
5. After review, merge by squash and delete the branch.

## Handoff report

The report lists what passed, what did not run and why, and the judgment calls that remain.
