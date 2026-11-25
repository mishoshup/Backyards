#!/usr/bin/env bash
set -euo pipefail

maximum_diff_bytes=600000

fetch_diff() {
  if [ -n "${PULL_REQUEST_NUMBER:-}" ]; then
    gh pr diff "${PULL_REQUEST_NUMBER}" --repo "${REPOSITORY}"
  else
    git diff "$(git merge-base "${1:-origin/main}" HEAD)"
  fi
}

fetch_capped_diff() {
  local sigpipe_status=141
  { fetch_diff "$@" || [ $? -eq "${sigpipe_status}" ]; } | head -c "$((maximum_diff_bytes + 1))"
}

reject_oversized_diff() {
  local diff_byte_count
  diff_byte_count=$(printf '%s' "$1" | wc -c)
  if [ "${diff_byte_count}" -gt "$maximum_diff_bytes" ]; then
    echo "review: diff exceeds ${maximum_diff_bytes} bytes; split the pull request" >&2
    exit 1
  fi
}

main() {
  local diff_text
  diff_text=$(fetch_capped_diff "$@")
  reject_oversized_diff "$diff_text"
  printf '%s\n' "$diff_text"
}

main "$@"
