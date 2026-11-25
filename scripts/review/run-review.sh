#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

select_requester() {
  case "$1" in
    holistic) echo scripts/review/request-deepseek-agent-review.sh ;;
    *) echo scripts/review/request-deepseek-review.sh ;;
  esac
}

main() {
  "$(select_requester "$1")" "$1" | scripts/review/post-review-findings.sh "$1"
}

main "$@"
