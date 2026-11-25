#!/usr/bin/env bash
set -euo pipefail

pull_request_number="$1"
status_context="$2"

head_sha=$(gh api "repos/${GITHUB_REPOSITORY}/pulls/${pull_request_number}" --jq .head.sha)
existing_state=$(gh api "repos/${GITHUB_REPOSITORY}/commits/${head_sha}/statuses" \
  --jq "[.[] | select(.context == \"${status_context}\")] | first | .state // \"\"")

{
  echo "head_sha=${head_sha}"
  echo "context=${status_context}"
  if [[ "${existing_state}" == "success" ]]; then echo "skip=true"; else echo "skip=false"; fi
} >> "${GITHUB_OUTPUT}"
