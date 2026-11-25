#!/usr/bin/env bash
set -euo pipefail

pull_request_number="$1"
content="$2"
reactor_login="$3"

reaction_id=$(gh api "repos/${GITHUB_REPOSITORY}/issues/${pull_request_number}/reactions" \
  --jq "[.[] | select(.content == \"${content}\" and .user.login == \"${reactor_login}\")] | first | .id // empty")
if [[ -n "${reaction_id}" ]]; then
  gh api -X DELETE "repos/${GITHUB_REPOSITORY}/issues/${pull_request_number}/reactions/${reaction_id}" > /dev/null
fi
