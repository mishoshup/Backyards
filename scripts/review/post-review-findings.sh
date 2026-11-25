#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
# shellcheck source=/dev/null
source .github/scripts/review/review-constants.sh

readonly review_mode="${1:?mode required}"
readonly MAXIMUM_POSTED_FINDINGS=30
readonly MAXIMUM_SUMMARY_FINDINGS=100

select_finding_noun() {
  if [ "$1" -eq 1 ]; then echo finding; else echo findings; fi
}

describe_count() {
  if [ "$1" -eq 0 ]; then echo "no findings, all ok"; else echo "$1 $(select_finding_noun "$1")"; fi
}

record_result() {
  echo "$1: $2" > "$(resolve_review_result_path "$1")"
}

describe_truncation() {
  if [ "$1" -gt "${MAXIMUM_POSTED_FINDINGS}" ]; then echo " (first ${MAXIMUM_POSTED_FINDINGS} posted)"; fi
}

has_valid_answer() {
  jq -e '.violations | arrays' <<<"$1" > /dev/null
}

reject_missing_answer() {
  record_result "${review_mode}" "no answer"
  echo "review: ${review_mode} reviewer gave no valid answer" >&2
  exit 1
}

failed_violations=()

post_each_violation() {
  local violation
  while IFS= read -r violation; do
    .github/scripts/review/create-inline-comment.sh "${PULL_REQUEST_NUMBER}" \
      "$(jq -r .path <<<"${violation}")" "$(jq -r .body <<<"${violation}")" \
      "$(jq -r .line <<<"${violation}")" "${review_mode}" "$(jq -r '.ruleId // ""' <<<"${violation}")" \
      || failed_violations+=("${violation}")
  done < <(jq -c '.violations[]' <<<"$1")
}

post_summary_comment() {
  local body
  local omitted_count
  omitted_count=$(jq --argjson cap "${MAXIMUM_SUMMARY_FINDINGS}" '[(.violations | length) - $cap, 0] | max' <<<"$1")
  body=$(jq -r --arg mode "${review_mode}" --argjson cap "${MAXIMUM_SUMMARY_FINDINGS}" --arg omittedNoun "$(select_finding_noun "${omitted_count}")" \
    '"Automated " + $mode + " review: some findings could not be placed inline.\n\n" + ([.violations[:$cap][] | "- `" + .path + ":" + (.line | tostring) + "` " + .body] | join("\n")) + (if (.violations | length) > $cap then "\n\n" + ((.violations | length) - $cap | tostring) + " further " + $omittedNoun + " omitted." else "" end)' <<<"$1")
  gh pr comment "${PULL_REQUEST_NUMBER}" --repo "${REPOSITORY}" --body "${body}"
}

select_first_violations() {
  jq --argjson cap "${MAXIMUM_POSTED_FINDINGS}" '{violations: .violations[:$cap]}' <<<"$1"
}

select_remaining_violations() {
  jq --argjson cap "${MAXIMUM_POSTED_FINDINGS}" '{violations: .violations[$cap:]}' <<<"$1"
}

summarize_unposted_violations() {
  local failed_json
  failed_json=$(printf '%s\n' "${failed_violations[@]:-}" | jq -s --argjson overflow "$1" 'map(select(. != null)) as $failed | {violations: ($failed + $overflow.violations)}')
  if [[ "$(jq '.violations | length' <<<"${failed_json}")" -gt 0 ]]; then
    post_summary_comment "${failed_json}"
  fi
}

post_findings() {
  post_each_violation "$(select_first_violations "$1")"
  summarize_unposted_violations "$(select_remaining_violations "$1")"
}

main() {
  local review_json
  review_json=$(cat)
  has_valid_answer "${review_json}" || reject_missing_answer
  local finding_count
  finding_count=$(jq '.violations | length' <<<"${review_json}")
  record_result "${review_mode}" "$(describe_count "${finding_count}")$(describe_truncation "${finding_count}")"
  if [[ "${finding_count}" -eq 0 ]]; then
    return 0
  fi
  : "${REVIEW_HEAD_SHA:?reviewed commit required}"
  export REVIEW_HEAD_SHA
  post_findings "${review_json}"
}

main "$@"
