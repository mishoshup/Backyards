#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

main() {
  scripts/review/request-deepseek-review.sh "$1" | scripts/review/post-review-findings.sh "$1"
}

main "$@"
