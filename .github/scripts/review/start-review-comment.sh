#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=/dev/null
source "$(dirname "$0")/review-constants.sh"

readonly pull_request_number="${1:?pull request number required}"
readonly head_sha="${2:?head SHA required}"

create_comment() {
  gh api "repos/${GITHUB_REPOSITORY}/issues/${pull_request_number}/comments" \
    -f body="Agent review started for \`${head_sha:0:SHORT_SHA_LENGTH}\`. Results will replace this text." --jq .id
}

main() {
  local comment_id
  comment_id=$(create_comment)
  echo "comment_id=${comment_id}" >> "${GITHUB_OUTPUT}"
  .github/scripts/review/add-comment-reaction.sh "${comment_id}" "eyes"
}

main
