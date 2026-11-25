#!/usr/bin/env bash
set -euo pipefail

readonly review_mode="${1:?mode required}"
readonly MAXIMUM_POSTED_FINDINGS=30

finding_noun() {
  if [ "$1" -eq 1 ]; then echo finding; else echo findings; fi
}

describe_count() {
  if [ "$1" -eq 0 ]; then echo "no findings, all ok"; else echo "$1 $(finding_noun "$1")"; fi
}

record_result() {
  echo "$1: $2" > "${RUNNER_TEMP:-/tmp}/review-result-$1"
}

has_valid_answer() {
  jq -e '.violations | arrays' <<<"$1" > /dev/null 2>&1
}

reject_missing_answer() {
  record_result "${review_mode}" "no answer"
  echo "review: ${review_mode} reviewer gave no valid answer" >&2
  exit 1
}

post_each_violation() {
  local violation has_failed=0
  while IFS= read -r violation; do
    .github/scripts/review/create-inline-comment.sh "${PULL_REQUEST_NUMBER}" \
      "$(jq -r .path <<<"${violation}")" "$(jq -r .body <<<"${violation}")" \
      "$(jq -r .line <<<"${violation}")" "${review_mode}" "$(jq -r '.ruleId // ""' <<<"${violation}")" || has_failed=1
  done < <(jq -c '.violations[]' <<<"$1")
  return "${has_failed}"
}

post_summary_comment() {
  local body
  body=$(jq -r --arg mode "${review_mode}" \
    '"Automated " + $mode + " review: some comments could not be placed inline.\n\n" + ([.violations[] | "- `" + .path + ":" + (.line | tostring) + "` " + .body] | join("\n"))' <<<"$1")
  gh pr comment "${PULL_REQUEST_NUMBER}" --repo "${REPOSITORY}" --body "${body}"
}

main() {
  local review_json
  review_json=$(cat)
  has_valid_answer "${review_json}" || reject_missing_answer
  record_result "${review_mode}" "$(describe_count "$(jq '.violations | length' <<<"${review_json}")")"
  if [[ "$(jq '.violations | length' <<<"${review_json}")" -eq 0 ]]; then
    return 0
  fi
  : "${REVIEW_HEAD_SHA:?reviewed commit required}"
  export REVIEW_HEAD_SHA
  review_json=$(jq --argjson cap "${MAXIMUM_POSTED_FINDINGS}" '.violations |= .[:$cap]' <<<"${review_json}")
  post_each_violation "${review_json}" || post_summary_comment "${review_json}"
}

main "$@"
