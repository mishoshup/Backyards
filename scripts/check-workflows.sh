#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

readonly review_workflow=.github/workflows/agent-code-review.yml
readonly minimum_timeout_minutes=45
failures=0

report_failure() {
  echo "workflow: $1" >&2
  failures=$((failures + 1))
}

check_review_runs_are_not_cancelled() {
  ! grep -q 'cancel-in-progress: true' "${review_workflow}" || report_failure "review runs must not cancel in progress"
}

check_review_timeout_covers_worst_case() {
  local minutes
  minutes=$(grep -oE 'timeout-minutes: [0-9]+' "${review_workflow}" | awk '{print $2}' | sort -n | tail -1)
  [ "${minutes:-0}" -ge "${minimum_timeout_minutes}" ] || report_failure "review timeout must be at least ${minimum_timeout_minutes} minutes"
}

check_default_permissions_are_empty() {
  grep -q '^permissions: {}' "${review_workflow}" || report_failure "workflow default permissions must be empty"
}

check_reviewed_commit_is_pinned() {
  grep -q 'ref: refs/pull/' "${review_workflow}" || report_failure "checkout must name the pull request ref"
  grep -q 'git rev-parse HEAD\|rev-parse HEAD' .github/scripts/review/should-skip-review.sh || report_failure "head SHA must come from the checked-out commit"
}

check_fork_pull_requests_are_refused() {
  grep -q 'head.repo.full_name == github.repository' "${review_workflow}" || report_failure "fork pull requests must be refused"
}

check_fetch_allowlist_is_committed() {
  git ls-files --error-unmatch scripts/review/allowed-fetch-hosts.txt >/dev/null 2>&1 || report_failure "allowed-fetch-hosts.txt must be tracked"
}

check_local_review_passes_prompt_by_file() {
  grep -qF -e "-f \"\$" scripts/review/run-local-review.sh || report_failure "local review must attach the prompt as a file"
}

main() {
  check_review_runs_are_not_cancelled
  check_review_timeout_covers_worst_case
  check_default_permissions_are_empty
  check_reviewed_commit_is_pinned
  check_fork_pull_requests_are_refused
  check_fetch_allowlist_is_committed
  check_local_review_passes_prompt_by_file
  [ "${failures}" -eq 0 ]
}

main "$@"
