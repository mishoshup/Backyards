#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

review_model="${LOCAL_REVIEW_MODEL:-opencode/big-pickle}"
work_directory=$(mktemp -d)
remove_review_processes() {
  jobs -p | xargs kill 2>/dev/null || true
}
trap 'remove_review_processes; rm -rf -- "${work_directory}"' EXIT

run_one_review() {
  local mode="$1" instructions diff_text
  local prompt_file="${work_directory}/${mode}.prompt"
  instructions=$(scripts/review/build-review-prompt.sh "$mode")
  diff_text=$(scripts/review/read-pull-request-diff.sh "${BASE_REFERENCE:-origin/main}")
  printf '%s\n\nDiff:\n%s\n' "${instructions}" "${diff_text}" > "${prompt_file}"
  opencode run -m "$review_model" -f "${prompt_file}" "Follow the instructions in the attached file." \
    > "${work_directory}/${mode}.out" 2>&1
}

print_review_output() {
  local mode="$1"
  printf '=== %s review ===\n' "$mode"
  cat "${work_directory}/${mode}.out"
}

main() {
  local inline_status=0 holistic_status=0
  run_one_review inline &
  local inline_process=$!
  run_one_review holistic &
  local holistic_process=$!
  wait "${inline_process}" || inline_status=$?
  wait "${holistic_process}" || holistic_status=$?
  print_review_output inline
  print_review_output holistic
  [[ ${inline_status} -eq 0 && ${holistic_status} -eq 0 ]]
}

main "$@"
