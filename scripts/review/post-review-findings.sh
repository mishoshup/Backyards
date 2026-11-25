#!/usr/bin/env bash
set -euo pipefail

readonly review_mode="${1:?mode required}"

post_each_violation() {
  local violation failed=0
  while IFS= read -r violation; do
    .github/scripts/review/create-inline-comment.sh "${PR_NUMBER}" \
      "$(jq -r .path <<<"${violation}")" "$(jq -r .body <<<"${violation}")" \
      "$(jq -r .line <<<"${violation}")" "${review_mode}" || failed=1
  done < <(jq -c '.violations[]' <<<"$1")
  return "${failed}"
}

post_summary_comment() {
  local body
  body=$(jq -r --arg mode "${review_mode}" \
    '"Automated " + $mode + " review: some comments could not be placed inline.\n\n" + ([.violations[] | "- `" + .path + ":" + (.line | tostring) + "` " + .body] | join("\n"))' <<<"$1")
  gh pr comment "${PR_NUMBER}" --repo "${REPOSITORY}" --body "${body}"
}

main() {
  local review_json
  review_json=$(cat)
  if [[ "$(jq '.violations | length' <<<"${review_json}")" -eq 0 ]]; then
    .github/scripts/review/add-pull-request-reaction.sh "${PR_NUMBER}" "+1"
    return 0
  fi
  post_each_violation "${review_json}" || post_summary_comment "${review_json}"
}

main "$@"
