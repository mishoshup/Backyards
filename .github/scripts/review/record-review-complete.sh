#!/usr/bin/env bash
set -euo pipefail

readonly head_sha="${1:?head SHA required}"
readonly status_context="${2:?status context required}"
readonly description="${3:-Reviewed at this commit}"
readonly maximum_description_length=140
# shellcheck source=/dev/null
source "$(dirname "$0")/validate-status-context.sh"

validate_arguments() {
  [[ "${head_sha}" =~ ^[0-9a-f]{40}$ ]] || { echo "head SHA must be 40 hexadecimal characters" >&2; exit 1; }
  validate_status_context "${status_context}"
}

post_status() {
  gh api -X POST "repos/${GITHUB_REPOSITORY}/statuses/${head_sha}" \
    -f state=success \
    -f context="${status_context}" \
    -f description="${description:0:${maximum_description_length}}" \
    -f target_url="${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}" > /dev/null
}

main() {
  validate_arguments
  post_status
}

main
