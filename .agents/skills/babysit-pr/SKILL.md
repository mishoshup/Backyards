---
name: babysit-pr
description: Drive a Backyards pull request from local work to merge. Covers the local gate, local review, pushing, reading agent review comments, replying to and resolving each thread, repeating until clean, and merging. Use when asked to babysit, shepherd, loop on or merge a pull request, or when starting any change that ends in a pull request.
---

# Babysit a pull request

The loop runs until the pull request is clean, then merges it. Pushes to the pull request branch and the final squash merge are pre-approved. Pushes to `main` are never allowed.

## Loop

1. Develop on a branch named `<type>/<short-description>` created from an up-to-date `main`.
2. Run `scripts/check.sh` and the local review (`scripts/review/run-local-review.sh`). Fix every finding that holds against the source.
3. Commit with the subject `<type>(<scope>): <subject>`, at most 72 characters, no trailers. Push with `git push origin HEAD:refs/heads/<branch>`.
4. Wait for the agent review of the pushed commit. Read the checks with `gh pr checks <number>` and the review threads with the query below.
5. For every unresolved thread, verify the claim against the source, then:
   - If it holds, fix it in a new commit and reply with the commit and what changed.
   - If it does not hold, reply with the evidence and the source checked.
6. Resolve each thread after replying. Push the fixes and return to step 4.
7. Stop looping when all of these hold for the head commit:
   - `scripts/check.sh` passes locally and every required check passes on the pull request.
   - The agent review status for the head commit is `success` and posted no new finding.
   - No review thread is unresolved.
   - The pull request is not a draft and its title matches `scripts/check-pull-request-naming.sh`.
8. Re-read the final diff once, confirm the description matches it, then merge with `gh pr merge <number> --squash --delete-branch`. Report what merged and what was verified.

## Review threads

List unresolved threads:

```bash
gh api graphql -f query='{repository(owner:"mishoshup",name:"backyards"){pullRequest(number:N){reviewThreads(first:100){nodes{id isResolved comments(first:5){nodes{databaseId path line body}}}}}}}'
```

Reply to a comment:

```bash
gh api -X POST repos/mishoshup/backyards/pulls/N/comments/COMMENT_ID/replies -f body="..."
```

Resolve a thread:

```bash
gh api graphql -f query='mutation{resolveReviewThread(input:{threadId:"THREAD_ID"}){thread{isResolved}}}'
```

## Rules

- Reply to a thread before resolving it. A resolved thread without a reply is incomplete.
- A reply states the commit that fixed the finding or the evidence that it does not hold. It is plain prose.
- Re-run the agent review with a comment that contains `/agent-review`. Do not use `@` mentions, because they notify other accounts.
- Confirm the account with `gh auth status`. It must be `mishoshup`.
- A failing check that is not caused by the change is reported, not hidden. Do not merge over a failing required check.
- When the review service returns an error such as insufficient balance, stop the loop and report it. Do not merge without a completed review.
- Wait for the review with `gh run watch <run id> --exit-status`, which blocks until the run finishes. Do not poll with sleep.
- The pull request is updated by adding commits. Never force-push and never merge `main` into the branch.
