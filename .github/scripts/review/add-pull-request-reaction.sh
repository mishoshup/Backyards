#!/usr/bin/env bash
set -euo pipefail

gh api "repos/${GITHUB_REPOSITORY}/issues/$1/reactions" -f content="$2" > /dev/null
