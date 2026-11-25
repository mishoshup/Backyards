#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

review_model="${LOCAL_REVIEW_MODEL:-opencode/big-pickle}"

run_one_review() {
  local mode="$1" instructions diff_text
  instructions=$(scripts/review/build-review-prompt.sh "$mode")
  diff_text=$(scripts/review/read-pull-request-diff.sh "${BASE_REFERENCE:-origin/main}")
  printf '=== %s review ===\n' "$mode"
  opencode run -m "$review_model" "${instructions}"$'\n\nDiff:\n'"${diff_text}"
}

main() {
  run_one_review inline
  run_one_review holistic
}

main "$@"
