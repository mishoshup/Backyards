#!/usr/bin/env bash
set -euo pipefail

readonly rules_directory="${1:?rules directory required}"
readonly output_file="${2:?output file required}"

collect_rule_identifiers() {
  local rule_file
  for rule_file in "${rules_directory}"/[!_]*.md; do
    [[ -f "${rule_file}" ]] || continue
    grep -m1 '^ruleId:' "${rule_file}" | grep -oE '[A-Z]+(-[A-Z]+)*-[0-9]+' || true
  done
}

main() {
  [[ -d "${rules_directory}" ]] || { echo "rules directory not found: ${rules_directory}" >&2; exit 1; }
  collect_rule_identifiers | sort -u > "${output_file}"
  [[ -s "${output_file}" ]] || { echo "no rule identifiers found in ${rules_directory}" >&2; exit 1; }
  echo "ALLOWED_RULES_FILE=${output_file}" >> "${GITHUB_ENV:-/dev/null}"
}

main
