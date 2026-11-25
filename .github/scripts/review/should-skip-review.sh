#!/usr/bin/env bash
set -euo pipefail

pull_request_number="$1"
status_context="$2"
event_name="$3"

head_sha=$(gh api "repos/${GITHUB_REPOSITORY}/pulls/${pull_request_number}" --jq .head.sha)
existing_state=$(gh api "repos/${GITHUB_REPOSITORY}/commits/${head_sha}/statuses" \
  --jq "[.[] | select(.context == \"${status_context}\")] | first | .state // \"\"")

decide_skip() {
  if [[ "${event_name}" == "issue_comment" ]]; then
    echo "false"
  elif [[ "${existing_state}" == "success" ]]; then
    echo "true"
  else
    echo "false"
  fi
}

{
  echo "head_sha=${head_sha}"
  echo "context=${status_context}"
  echo "skip=$(decide_skip)"
} >> "${GITHUB_OUTPUT}"
