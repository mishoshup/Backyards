#!/usr/bin/env bash
set -euo pipefail

maximum_diff_bytes=600000

fetch_diff() {
  if [ -n "${PR_NUMBER:-}" ]; then
    gh pr diff "${PR_NUMBER}" --repo "${REPOSITORY}"
  else
    git diff "${1:-origin/main}...HEAD"
  fi
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
  diff_text=$(fetch_diff "$@")
  reject_oversized_diff "$diff_text"
  printf '%s\n' "$diff_text"
}

main "$@"
