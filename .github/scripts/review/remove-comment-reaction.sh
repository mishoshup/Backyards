#!/usr/bin/env bash
set -euo pipefail

readonly comment_id="${1:?comment id required}"
readonly reaction_content="${2:?reaction content required}"
readonly reactor_login="${3:?reactor login required}"

find_reaction_id() {
  gh api "repos/${GITHUB_REPOSITORY}/issues/comments/${comment_id}/reactions" \
    --jq "[.[] | select(.content == \"${reaction_content}\" and .user.login == \"${reactor_login}\")] | first | .id // empty"
}

main() {
  local reaction_id
  reaction_id=$(find_reaction_id)
  if [[ -n "${reaction_id}" ]]; then
    gh api -X DELETE "repos/${GITHUB_REPOSITORY}/issues/comments/${comment_id}/reactions/${reaction_id}" > /dev/null
  fi
}

main
