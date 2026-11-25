#!/usr/bin/env bash
set -euo pipefail

readonly pull_request_number="${1:?pull request number required}"
readonly reviewer_context="${2:?status context required}"
readonly context_pattern='^[a-z0-9]([a-z0-9/_-]*[a-z0-9])?$'

validate_arguments() {
  [[ "${pull_request_number}" =~ ^[0-9]+$ ]] || { echo "pull request number must be an integer" >&2; exit 1; }
  [[ "${reviewer_context}" =~ ${context_pattern} ]] || { echo "status context must be lowercase with / _ - separators" >&2; exit 1; }
}

read_head_sha() {
  gh api "repos/${GITHUB_REPOSITORY}/pulls/${pull_request_number}" --jq .head.sha
}

read_existing_state() {
  gh api --paginate "repos/${GITHUB_REPOSITORY}/commits/$1/status" \
    --jq ".statuses[] | select(.context == \"$2\") | .state"
}

decide_skip() {
  if [[ "${GITHUB_EVENT_NAME:-}" == "issue_comment" ]]; then
    echo "false"
  elif [[ "$1" == "success" ]]; then
    echo "true"
  else
    echo "false"
  fi
}

main() {
  validate_arguments
  local head_sha status_context existing_state
  head_sha=$(read_head_sha)
  [[ -n "${head_sha}" ]] || { echo "could not resolve head SHA" >&2; exit 1; }
  status_context="${reviewer_context}/pr-${pull_request_number}"
  existing_state=$(read_existing_state "${head_sha}" "${status_context}")
  {
    echo "head_sha=${head_sha}"
    echo "context=${status_context}"
    echo "skip=$(decide_skip "${existing_state}")"
  } >> "${GITHUB_OUTPUT}"
}

main
