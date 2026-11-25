#!/usr/bin/env bash
set -euo pipefail

title="${PULL_REQUEST_TITLE:-}"
branch="${PULL_REQUEST_BRANCH:-}"
change_types='feat|fix|refactor|perf|test|docs|chore|ci|build'
maximum_title_length=72
failures=0

report_failure() {
  echo "pull request naming: $1" >&2
  failures=$((failures + 1))
}

check_title_format() {
  echo "$title" | grep -qE "^(${change_types})\([a-z0-9-]+\): [a-z0-9].*[^.]$" \
    || report_failure "title \"$title\" must match <type>(<scope>): <subject> without a trailing period"
}

check_title_length() {
  [ "${#title}" -le "$maximum_title_length" ] \
    || report_failure "title has ${#title} characters, the limit is $maximum_title_length"
}

check_branch_format() {
  echo "$branch" | grep -qE "^(${change_types})/[a-z0-9]+(-[a-z0-9]+)*$" \
    || report_failure "branch \"$branch\" must match <type>/<short-description> in lowercase hyphens"
}

check_branch_type_matches_title() {
  local title_type="${title%%(*}"
  local branch_type="${branch%%/*}"
  [ "$title_type" = "$branch_type" ] \
    || report_failure "title type \"$title_type\" differs from branch type \"$branch_type\""
}

main() {
  check_title_format
  check_title_length
  check_branch_format
  check_branch_type_matches_title
  [ "$failures" -eq 0 ] || exit 1
  echo "pull request naming passed"
}

main "$@"
