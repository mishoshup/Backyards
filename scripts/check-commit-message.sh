#!/usr/bin/env bash
set -euo pipefail

message_file="$1"
subject=$(head -n 1 "$message_file")

reject() {
  echo "commit message: $1" >&2
  exit 1
}

check_no_forbidden_trailers() {
  if grep -qiE '^(Co-Authored-By|Claude-Session):' "$message_file"; then
    reject "Co-Authored-By and Claude-Session trailers are not allowed"
  fi
}

check_subject_length() {
  [ "${#subject}" -le 72 ] || reject "subject is longer than 72 characters"
}

check_subject_format() {
  echo "$subject" | grep -qE '^(feat|fix|docs|refactor|test|chore|build|ci|perf)\([a-z0-9-]+\): [a-z].*[^.]$' \
    || reject "subject must match <type>(<scope>): <lowercase imperative subject> with no trailing period"
}

main() {
  check_no_forbidden_trailers
  check_subject_length
  check_subject_format
}

main "$@"
