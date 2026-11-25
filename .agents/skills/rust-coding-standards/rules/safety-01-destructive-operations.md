---
ruleId: SAFETY-01
enforcedBy: review; integration tests for dry run and allowlist
source: Project safety policy (user requirement); dry-run default as in Terraform plan
---

# Destructive operations are allowlisted, dry run, audited

Every delete, prune or overwrite follows the same four steps:

1. Resolve the target and confirm it sits under an allowlisted root. Refuse anything else, including symlinks that leave the root.
2. Default to a dry run that prints what would change. A real run needs an explicit flag.
3. Write an audit entry before and after each step.
4. Report after each step: what was removed, how much space returned, what was skipped and why.

Retention for caches and images is 7 days unless the manifest says otherwise. Never delete a path the manifest does not declare as managed.

Tests run against a temporary directory, never the real home directory.
