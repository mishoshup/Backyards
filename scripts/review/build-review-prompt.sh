#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

strip_front_matter() {
  awk 'BEGIN{fence=0} /^---$/ && fence<2 {fence++; next} fence>=2 || fence==0 {print}' "$1"
}

print_reviewer_instructions() {
  case "$1" in
    inline) strip_front_matter .claude/agents/code-inline-reviewer.md ;;
    holistic) strip_front_matter .claude/agents/holistic-reviewer.md ;;
    *) echo "review: unknown mode $1" >&2; exit 2 ;;
  esac
}

print_rule_files() {
  local rule_file
  for rule_file in .agents/skills/rust-coding-standards/rules/*.md; do
    printf '\n### %s\n' "$rule_file"
    cat "$rule_file"
  done
}

print_output_contract() {
  cat <<'CONTRACT'

Respond with JSON only, in this shape:
{"findings":[{"path":"<file path>","line":<line number in the new file>,"body":"<defect and smallest correction>"}]}
Use {"findings":[]} when there is no finding. Report only lines present in the diff.
CONTRACT
}

main() {
  print_reviewer_instructions "$1"
  if [ "$1" = "inline" ]; then
    print_rule_files
  fi
  print_output_contract
}

main "$@"
