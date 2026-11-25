#!/usr/bin/env bash
set -euo pipefail

body="${PULL_REQUEST_BODY:-}"
failures=0

report_failure() {
  echo "pull request description: $1" >&2
  failures=$((failures + 1))
}

check_required_sections() {
  local section
  for section in "### Problem" "### Change" "### Verification"; do
    grep -qF "$section" <<<"$body" || report_failure "missing section \"$section\""
  done
}

check_checklist_complete() {
  local unchecked
  unchecked=$(grep -cE '^- \[ \]' <<<"$body" || true)
  [ "$unchecked" -eq 0 ] || report_failure "$unchecked checklist box(es) unchecked, check each or write n/a after it"
}

main() {
  check_required_sections
  check_checklist_complete
  [ "$failures" -eq 0 ] || exit 1
  echo "pull request description passed"
}

main "$@"
