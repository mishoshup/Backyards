#!/usr/bin/env bash
set -euo pipefail

head_sha="$1"
status_context="$2"
description="$3"

gh api "repos/${GITHUB_REPOSITORY}/statuses/${head_sha}" \
  -f state=success -f context="${status_context}" -f description="${description}" > /dev/null
