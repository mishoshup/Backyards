#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=/dev/null
source "$(dirname "$0")/review-constants.sh"

readonly comment_id="${1:?comment id required}"
readonly job_status="${2:?job status required}"
readonly head_sha="${3:?head SHA required}"

read_result_line() {
  local result_file
  result_file=$(resolve_review_result_path "$1")
  if [[ -f "${result_file}" ]]; then cat "${result_file}"; else echo "$1: did not run"; fi
}

count_findings() {
  cat "$(resolve_review_result_path inline)" "$(resolve_review_result_path holistic)" 2>/dev/null | awk '{ total += $2 } END { print total + 0 }'
}

build_body() {
  printf 'Agent review of %s: %s\n\n- %s\n- %s\n' "${head_sha:0:SHORT_SHA_LENGTH}" "$1" "$(read_result_line inline)" "$(read_result_line holistic)"
}

update_comment() {
  gh api -X PATCH "repos/${GITHUB_REPOSITORY}/issues/comments/${comment_id}" -f body="$1" > /dev/null
}

react_to_outcome() {
  .github/scripts/review/remove-comment-reaction.sh "${comment_id}" "eyes" "github-actions[bot]" || echo "warning: could not remove the in-progress reaction" >&2
  if [[ "${job_status}" == "success" && -f "$(resolve_review_result_path inline)" && -f "$(resolve_review_result_path holistic)" && "$(count_findings)" -eq 0 ]]; then
    .github/scripts/review/add-comment-reaction.sh "${comment_id}" "+1" || echo "warning: could not add the approval reaction" >&2
  fi
}

main() {
  local outcome="completed"
  [[ "${job_status}" == "success" ]] || outcome="did not complete (${job_status})"
  update_comment "$(build_body "${outcome}")"
  react_to_outcome
}

main
