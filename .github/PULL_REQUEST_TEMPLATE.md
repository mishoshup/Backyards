### Problem

The problem this change addresses and the current defect or gap.

### Change

The change, listed per crate.

### Verification

Mark each item as checked, or write n/a if it does not apply.

- [ ] `cargo xtask check` passes on the final commit.
- [ ] The behavior is demonstrated with command output or a real garden run on macOS.
- [ ] Destructive paths use a dry run by default, an allowlist, an audit entry and a per-step report.
- [ ] No silent drops: every skipped item is recorded and reported.
- [ ] Library behavior is verified against crate source or upstream documentation. The source is stated.
- [ ] A bug fix includes a test that fails without the change.
- [ ] Commits carry no trailers, and the title matches `<type>(<scope>): <subject>`.
