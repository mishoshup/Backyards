#!/usr/bin/env bash
set -euo pipefail

readonly comment_id="${1:?comment id required}"
readonly reaction_content="${2:?reaction content required}"

gh api "repos/${GITHUB_REPOSITORY}/issues/comments/${comment_id}/reactions" -f content="${reaction_content}" > /dev/null
