---
name: babysit-pr
description: Drive a Backyards pull request from local work to merge. Covers the local gate, local review, pushing, reading agent review comments, replying to and resolving each thread, repeating until clean, and merging. Use when asked to babysit, shepherd, loop on or merge a pull request, or when starting any change that ends in a pull request.
---

# Babysit a pull request

The loop spends the local run first and the CI review at most twice. It ends with a report to the owner, and a merge only after the owner approves. Pushes to the pull request branch are covered by the standing approval recorded in AGENTS.md section 10 item 5. Pushes to `main` are never allowed.

## Loop

1. The owner asks for the pull request to be babysat.
2. Develop on a branch named `<type>/<short-description>` created from an up-to-date `main`. Run `cargo xtask check --keep-going` and `cargo xtask review local` after each change, fix every finding that holds against the source, and repeat until both are clean and the result is trusted. The local run is the main check, because each CI review spends the owner's review credits.
3. When confident, make the last commit and push with `git push origin HEAD:refs/heads/<branch>`, then open the pull request or mark it ready for review. This starts CI review run 1. On an open, non-draft pull request a push starts no review: toggle draft (`gh pr ready <number> --undo`, then `gh pr ready <number>`) or comment `/agent-review`; the comment form runs the default-branch workflow. Commit subjects use `<type>(<scope>): <subject>`, at most 72 characters, no trailers.
4. Read the result. If the agent review status is `success` with no finding that holds, report to the owner and ask whether to merge. If findings hold, fix them with the local loop from step 2, push, and request run 2 with a comment that contains `/agent-review`.
5. CI review runs at most twice per pull request. The code refuses a third run unless the comment contains `/agent-review force`, which only the owner authorizes. After run 2, report the final state to the owner and stop.

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
- Request a CI review with a comment that contains `/agent-review`. Do not use `@` mentions, because they notify other accounts. The review does not run on every push.
- Confirm the account with `gh auth status`. It must be `mishoshup`.
- A failing check that is not caused by the change is reported, not hidden. Do not merge over a failing required check.
- When the review service returns an error such as insufficient balance, stop the loop and report it. Do not merge without a completed review.
- Wait for the review with `gh run watch <run id> --exit-status`, which blocks until the run finishes. Do not poll with sleep. The status `ai-review-completed/deepseek/pr-N` is advisory, not a required check.
- Run every wait (checks, review runs, builds) as a background command and keep working meanwhile. Open the next task, other pull requests or the next review round in parallel instead of idling on one wait.
- The pull request is updated by adding commits. Never force-push and never merge `main` into the branch.
