#!/usr/bin/env bash
set -euo pipefail

build_review_payload() {
  jq --arg sha "$HEAD_SHA" --arg mode "$1" '{
    commit_id: $sha,
    event: "COMMENT",
    body: ("Automated " + $mode + " review: " + (.findings | length | tostring) + " finding(s)."),
    comments: [.findings[] | {path, line, side: "RIGHT", body}]
  }'
}

post_inline_review() {
  gh api "repos/${REPOSITORY}/pulls/${PR_NUMBER}/reviews" --input - > /dev/null
}

post_fallback_comment() {
  local mode="$1" findings_json="$2" body
  body=$(jq -r --arg mode "$mode" '"Automated " + $mode + " review (inline placement failed):\n\n" + ([.findings[] | "- `" + .path + ":" + (.line | tostring) + "` " + .body] | join("\n"))' <<<"$findings_json")
  gh pr comment "${PR_NUMBER}" --repo "${REPOSITORY}" --body "$body"
}

main() {
  local mode="$1" findings_json
  findings_json=$(cat)
  if [ "$(jq '.findings | length' <<<"$findings_json")" -eq 0 ]; then
    echo "review: no ${mode} findings"
    return 0
  fi
  build_review_payload "$mode" <<<"$findings_json" | post_inline_review \
    || post_fallback_comment "$mode" "$findings_json"
}

main "$@"
