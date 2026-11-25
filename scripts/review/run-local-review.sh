#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

review_model="${LOCAL_REVIEW_MODEL:-opencode/big-pickle}"
prompt_file=$(mktemp)
trap 'rm -f -- "${prompt_file}"' EXIT

run_one_review() {
  local mode="$1" instructions diff_text
  instructions=$(scripts/review/build-review-prompt.sh "$mode")
  diff_text=$(scripts/review/read-pull-request-diff.sh "${BASE_REFERENCE:-origin/main}")
  printf '=== %s review ===\n' "$mode"
  printf '%s\n\nDiff:\n%s\n' "${instructions}" "${diff_text}" > "${prompt_file}"
  opencode run -m "$review_model" -f "${prompt_file}" "Follow the instructions in the attached file."
}

main() {
  run_one_review inline
  run_one_review holistic
}

main "$@"
