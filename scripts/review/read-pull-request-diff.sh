#!/usr/bin/env bash
set -euo pipefail

maximum_diff_bytes=600000
diff_file=""

fetch_diff() {
  if [ -n "${PULL_REQUEST_NUMBER:-}" ]; then
    gh pr diff "${PULL_REQUEST_NUMBER}" --repo "${REPOSITORY}"
  else
    git diff "$(git merge-base "${1:-origin/main}" HEAD)"
    diff_untracked_files
  fi
}

diff_new_file() {
  local differences_found_status=1 status=0
  git diff --no-index -- /dev/null "$1" || status=$?
  [ "${status}" -le "${differences_found_status}" ]
}

diff_untracked_files() {
  local untracked_file
  while IFS= read -r untracked_file; do
    diff_new_file "${untracked_file}"
  done < <(git ls-files --others --exclude-standard)
}

fetch_capped_diff() {
  local sigpipe_status=141
  { fetch_diff "$@" || [ $? -eq "${sigpipe_status}" ]; } | head -c "$((maximum_diff_bytes + 1))"
}

reject_oversized_diff() {
  local diff_byte_count
  diff_byte_count=$(wc -c < "$1")
  if [ "${diff_byte_count}" -gt "$maximum_diff_bytes" ]; then
    echo "review: diff exceeds ${maximum_diff_bytes} bytes; split the pull request" >&2
    exit 1
  fi
}

remove_diff_file() {
  rm -f -- "${diff_file}"
}

main() {
  diff_file=$(mktemp)
  trap remove_diff_file EXIT
  fetch_capped_diff "$@" > "${diff_file}"
  reject_oversized_diff "${diff_file}"
  cat "${diff_file}"
}

main "$@"
