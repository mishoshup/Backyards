#!/usr/bin/env bash
set -euo pipefail

readonly pull_request_number="${1:?pull request number required}"
readonly file_path="${2:?path required}"
readonly comment_body="${3:?body required}"
readonly line_number="${4:?line required}"
readonly review_mode="${5:?mode required}"
readonly rule_pattern='[A-Z]+(-[A-Z]+)*-[0-9]+'

validate_arguments() {
  [[ "${pull_request_number}" =~ ^[0-9]+$ ]] || { echo "pull request number must be an integer" >&2; exit 1; }
  [[ "${line_number}" =~ ^[0-9]+$ ]] || { echo "line must be an integer" >&2; exit 1; }
}

validate_rule_reference() {
  [[ "${review_mode}" == "inline" ]] || return 0
  local rule
  rule=$(grep -oE "${rule_pattern}" <<<"${comment_body}" | head -1 || true)
  [[ -n "${rule}" ]] || { echo "comment rejected: missing rule reference" >&2; exit 1; }
  grep -qxF "${rule}" "${ALLOWED_RULES_FILE:?allowed rules file required}" \
    || { echo "comment rejected: rule ${rule} is not in the allowed list" >&2; exit 1; }
}

build_footer() {
  local short_sha="${1:0:7}"
  printf '\n\n---\n\nReviewed at: [%s](https://github.com/%s/commit/%s) | Rate this suggestion with a thumbs up or down; reactions measure reviewer quality.' \
    "${short_sha}" "${GITHUB_REPOSITORY}" "$1"
}

resolve_head_sha() {
  if [[ -n "${REVIEW_HEAD_SHA:-}" ]]; then
    echo "${REVIEW_HEAD_SHA}"
  else
    gh api "repos/${GITHUB_REPOSITORY}/pulls/${pull_request_number}" --jq .head.sha
  fi
}

post_comment() {
  local head_sha
  head_sha=$(resolve_head_sha)
  jq -n --arg body "${comment_body}$(build_footer "${head_sha}")" --arg path "${file_path}" \
    --argjson line "${line_number}" --arg sha "${head_sha}" \
    '{body: $body, path: $path, line: $line, side: "RIGHT", commit_id: $sha}' \
    | gh api -X POST "repos/${GITHUB_REPOSITORY}/pulls/${pull_request_number}/comments" --input - > /dev/null
}

main() {
  validate_arguments
  validate_rule_reference
  post_comment
}

main
